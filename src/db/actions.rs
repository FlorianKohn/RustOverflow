use crate::db::models::{
    Answer, DisplayQuestion, Login, NewQuestion, NewUser, Question, Tag, User,
};
use crate::db::DbConn;
use bcrypt::verify;
use diesel::expression::count::count_star;
use diesel::result::{DatabaseErrorKind, Error};
use diesel::{BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::http::Status;

fn internal_error<E>(_: E) -> (Status, String) {
    (Status::InternalServerError, "Database error".into())
}

// Helper functions
impl DbConn {
    async fn to_display_questions(
        &self,
        questions: Vec<Question>,
    ) -> Result<Vec<DisplayQuestion>, (Status, String)> {
        let mut res: Vec<DisplayQuestion> = Vec::with_capacity(questions.len());
        for q in questions.into_iter() {
            let dq = self.to_display_question(q).await?;
            res.push(dq);
        }
        Ok(res)
    }

    async fn to_display_question(&self, q: Question) -> Result<DisplayQuestion, (Status, String)> {
        Ok(DisplayQuestion {
            id: q.id,
            author: q.author,
            time: q.time,
            score: q.score,
            title: q.title,
            text: q.text,
            tags: self.tags(q.id).await?,
            num_answers: self.num_answers(q.id).await?,
            answered: self.answered(q.id).await?,
        })
    }
}

// pub(crate) interface
impl DbConn {
    pub(crate) async fn login(
        &self,
        login_name: String,
        login_pw: String,
    ) -> Result<Login, (Status, String)> {
        use crate::db::schema::users::dsl::*;
        let db_user: User = self
            .run(|c| users.filter(username.eq(login_name)).first::<User>(c))
            .await
            .map_err(|e| (Status::BadRequest, e.to_string()))?;
        let verified = verify(login_pw, &db_user.password).map_err(internal_error)?;

        verified
            .then(|| db_user.into())
            .ok_or((Status::Unauthorized, "wrong password".into()))
    }

    pub(crate) async fn register(
        &self,
        username: String,
        password: String,
    ) -> Result<Login, (Status, String)> {
        use crate::db::schema::users::dsl::users;
        use diesel::insert_into;

        let user = NewUser::create(username.clone(), password.clone())
            .map_err(|reason| (Status::BadRequest, reason))?;

        // Insert User into db
        self.run(move |c| insert_into(users).values(&user).execute(c))
            .await
            .map_err(|e: Error| match e {
                Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    (Status::BadRequest, "Username already exists".into())
                }
                _ => internal_error(e),
            })?;

        // log user in after registration
        self.login(username, password).await
    }

    pub(crate) async fn num_answers(&self, q_id: i32) -> Result<i64, (Status, String)> {
        use crate::db::schema::answers::dsl::*;
        self.run(move |c| {
            answers
                .filter(question.eq(q_id))
                .select(count_star())
                .first::<i64>(c)
        })
        .await
        .map_err(internal_error)
    }

    pub(crate) async fn answered(&self, q_id: i32) -> Result<bool, (Status, String)> {
        use crate::db::schema::answers::dsl::*;
        let res = self
            .run(move |c| {
                answers
                    .filter(question.eq(q_id).and(accepted.eq(true)))
                    .select(id)
                    .first::<i32>(c)
            })
            .await;

        match res {
            Ok(_) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(e) => Err(internal_error(e)),
        }
    }

    pub(crate) async fn tags(&self, q_id: i32) -> Result<Vec<Tag>, (Status, String)> {
        use crate::db::schema::chosen_tags::dsl::{chosen_tags, question};
        use crate::db::schema::tags::dsl::{description, id, name, tags};
        self.run(move |c| {
            chosen_tags
                .filter(question.eq(q_id))
                .inner_join(tags)
                .select((id, name, description))
                .load::<Tag>(c)
        })
        .await
        .map_err(internal_error)
    }

    pub(crate) async fn all_tags(&self) -> Result<Vec<Tag>, (Status, String)> {
        use crate::db::schema::tags::dsl::tags;
        self.run(move |c| tags.load(c))
            .await
            .map_err(internal_error)
    }

    pub(crate) async fn tags_with_names(
        &self,
        targets: Vec<String>,
    ) -> Result<Vec<Tag>, (Status, String)> {
        use crate::db::schema::tags::dsl::{name, tags};
        let target_len = targets.len();
        let res = self
            .run(move |c| tags.filter(name.eq_any(targets)).load::<Tag>(c))
            .await
            .map_err(internal_error)?;
        if res.len() != target_len {
            Err((Status::BadRequest, "Invalid Tag".into()))
        } else {
            Ok(res)
        }
    }

    pub(crate) async fn newest_questions(&self) -> Result<Vec<DisplayQuestion>, (Status, String)> {
        use crate::db::schema::questions::dsl::*;
        use crate::db::schema::users::dsl::{username, users};

        let new_questions: Vec<Question> = self
            .run(move |c| {
                questions
                    .inner_join(users)
                    .order_by(time.desc())
                    .select((id, username, time, score, title, text))
                    .load::<Question>(c)
            })
            .await
            .map_err(internal_error)?;

        self.to_display_questions(new_questions).await
    }

    pub(crate) async fn questions_with_tag(
        &self,
        target_tags: Vec<String>,
    ) -> Result<Vec<DisplayQuestion>, (Status, String)> {
        use crate::db::schema::chosen_tags::dsl::chosen_tags;
        use crate::db::schema::questions::dsl::*;
        use crate::db::schema::tags::dsl::{name, tags};
        use crate::db::schema::users::dsl::{username, users};

        let tagged_questions: Vec<Question> = self
            .run(move |c| {
                questions
                    .inner_join(users)
                    .inner_join(chosen_tags.inner_join(tags))
                    .filter(name.eq_any(target_tags))
                    .order_by(time.desc())
                    .select((id, username, time, score, title, text))
                    .distinct()
                    .load::<Question>(c)
            })
            .await
            .map_err(internal_error)?;

        self.to_display_questions(tagged_questions).await
    }

    pub(crate) async fn new_question(
        &self,
        author: i32,
        title: String,
        text: String,
        tags: Vec<i32>,
    ) -> Result<i32, (Status, String)> {
        use crate::db::schema::chosen_tags::dsl::{chosen_tags, question, tag};
        use crate::db::schema::questions::dsl::{id, questions};
        use diesel::insert_into;

        let new_question = NewQuestion {
            author,
            title,
            text,
        };
        // Insert question into db and retrieve id
        self.run(move |c| {
            c.transaction::<_, Error, _>(|| {
                insert_into(questions).values(&new_question).execute(c)?;
                let new_id = questions.order_by(id.desc()).select(id).first(c)?;
                for t in tags.iter() {
                    insert_into(chosen_tags)
                        .values((question.eq(new_id), tag.eq(t)))
                        .execute(c)?;
                }
                Ok(new_id)
            })
        })
        .await
        .map_err(|e: Error| match e {
            Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                (Status::BadRequest, "Invalid tag id supplied".into())
            }
            e => internal_error(e),
        })
    }

    pub(crate) async fn question(&self, qid: i32) -> Result<DisplayQuestion, (Status, String)> {
        use crate::db::schema::questions::dsl::*;
        use crate::db::schema::users::dsl::{username, users};
        let question = self
            .run(move |c| {
                questions
                    .inner_join(users)
                    .filter(id.eq(qid))
                    .select((id, username, time, score, title, text))
                    .first(c)
            })
            .await
            .map_err(|e: Error| match e {
                Error::NotFound => (Status::NotFound, "This question does not exist".into()),
                e => internal_error(e),
            })?;
        self.to_display_question(question).await
    }

    pub(crate) async fn answers(&self, qid: i32) -> Result<Vec<Answer>, (Status, String)> {
        use crate::db::schema::answers::dsl::*;
        use crate::db::schema::users::dsl::{username, users};
        self.run(move |c| {
            answers
                .inner_join(users)
                .filter(question.eq(qid))
                .select((id, username, question, time, score, accepted, text))
                .load(c)
        })
        .await
        .map_err(internal_error)
    }
}
