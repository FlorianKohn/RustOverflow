use crate::db::models::{
    Answer, DisplayQuestion, Login, NewAnswer, NewQuestion, NewUser, Question, Tag, User,
};
use crate::db::DbConn;
use bcrypt::verify;
use diesel::expression::count::count_star;
use diesel::result::{DatabaseErrorKind, Error};
use diesel::{
    insert_into, update, BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use rocket::http::Status;

fn internal_error<E>(_: E) -> (Status, String) {
    (Status::InternalServerError, "Database error".into())
}

// Helper functions
impl DbConn {
    /// Converts multiple questions into a DisplayQuestions
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

    /// Annotate a Question with the data needed for displaying it, transforming it into a DisplayQuestion.
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
    /// Verify the credentials of the given user and return a logged in user on success.
    pub(crate) async fn login(
        &self,
        login_name: String,
        login_pw: String,
    ) -> Result<Login, (Status, String)> {
        use crate::db::schema::users::dsl::*;
        let db_user: User = self
            .run(|connection| {
                users
                    .filter(username.eq(login_name))
                    .first::<User>(connection)
            })
            .await
            .map_err(|e| (Status::BadRequest, e.to_string()))?;
        let verified = verify(login_pw, &db_user.password).map_err(internal_error)?;

        verified
            .then(|| db_user.into())
            .ok_or((Status::Unauthorized, "wrong password".into()))
    }

    /// Create a new user and return a logged in user.
    pub(crate) async fn register(
        &self,
        username: String,
        password: String,
    ) -> Result<Login, (Status, String)> {
        use crate::db::schema::users::dsl::users;

        let user = NewUser::create(username.clone(), password.clone())
            .map_err(|reason| (Status::BadRequest, reason))?;

        // Insert User into db
        self.run(move |connection| insert_into(users).values(&user).execute(connection))
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

    /// Return the number of answers a question has
    pub(crate) async fn num_answers(&self, q_id: i32) -> Result<i64, (Status, String)> {
        use crate::db::schema::answers::dsl::*;
        self.run(move |connection| {
            answers
                .filter(question.eq(q_id))
                .select(count_star())
                .first::<i64>(connection)
        })
        .await
        .map_err(internal_error)
    }

    /// Return whether a question was answered
    pub(crate) async fn answered(&self, q_id: i32) -> Result<bool, (Status, String)> {
        use crate::db::schema::answers::dsl::*;
        let res = self
            .run(move |connection| {
                answers
                    .filter(question.eq(q_id).and(accepted.eq(true)))
                    .select(id)
                    .first::<i32>(connection)
            })
            .await;

        match res {
            Ok(_) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(e) => Err(internal_error(e)),
        }
    }

    /// Return all tags of a question
    pub(crate) async fn tags(&self, q_id: i32) -> Result<Vec<Tag>, (Status, String)> {
        use crate::db::schema::chosen_tags::dsl::{chosen_tags, question};
        use crate::db::schema::tags::dsl::{description, id, name, tags};
        self.run(move |connection| {
            chosen_tags
                .filter(question.eq(q_id))
                .inner_join(tags)
                .select((id, name, description))
                .load::<Tag>(connection)
        })
        .await
        .map_err(internal_error)
    }

    /// Return all tags in the database
    pub(crate) async fn all_tags(&self) -> Result<Vec<Tag>, (Status, String)> {
        use crate::db::schema::tags::dsl::tags;
        self.run(move |connection| tags.load(connection))
            .await
            .map_err(internal_error)
    }

    /// Return all tags with a name in the given vector of names.
    pub(crate) async fn tags_with_names(
        &self,
        targets: Vec<String>,
    ) -> Result<Vec<Tag>, (Status, String)> {
        use crate::db::schema::tags::dsl::{name, tags};
        let target_len = targets.len();
        let res = self
            .run(move |connection| tags.filter(name.eq_any(targets)).load::<Tag>(connection))
            .await
            .map_err(internal_error)?;
        if res.len() != target_len {
            Err((Status::BadRequest, "Invalid Tag".into()))
        } else {
            Ok(res)
        }
    }

    /// Select all questions in the database and oder them by newest first
    pub(crate) async fn newest_questions(&self) -> Result<Vec<DisplayQuestion>, (Status, String)> {
        use crate::db::schema::questions::dsl::*;
        use crate::db::schema::users::dsl::{username, users};

        let new_questions: Vec<Question> = self
            .run(move |connection| {
                questions
                    .inner_join(users)
                    .order_by(time.desc())
                    .select((id, username, time, score, title, text))
                    .load::<Question>(connection)
            })
            .await
            .map_err(internal_error)?;

        self.to_display_questions(new_questions).await
    }

    /// Select all question that have a tag in the specified target vector.
    pub(crate) async fn questions_with_tag(
        &self,
        target_tags: Vec<String>,
    ) -> Result<Vec<DisplayQuestion>, (Status, String)> {
        use crate::db::schema::chosen_tags::dsl::chosen_tags;
        use crate::db::schema::questions::dsl::*;
        use crate::db::schema::tags::dsl::{name, tags};
        use crate::db::schema::users::dsl::{username, users};

        let tagged_questions: Vec<Question> = self
            .run(move |connection| {
                questions
                    .inner_join(users)
                    .inner_join(chosen_tags.inner_join(tags))
                    .filter(name.eq_any(target_tags))
                    .order_by(time.desc())
                    .select((id, username, time, score, title, text))
                    .distinct()
                    .load::<Question>(connection)
            })
            .await
            .map_err(internal_error)?;

        self.to_display_questions(tagged_questions).await
    }

    /// Add a new question to the database and return the id of it.
    pub(crate) async fn new_question(
        &self,
        author: i32,
        title: String,
        text: String,
        tags: Vec<i32>,
    ) -> Result<i32, (Status, String)> {
        use crate::db::schema::chosen_tags::dsl::{chosen_tags, question, tag};
        use crate::db::schema::questions::dsl::{id, questions};

        let new_question = NewQuestion {
            author,
            title,
            text,
        };
        // Insert question into db and retrieve id
        // A transaction is used to guarantee atomicity of the operations.
        self.run(move |connection| {
            connection.transaction::<_, Error, _>(|| {
                insert_into(questions)
                    .values(&new_question)
                    .execute(connection)?;
                let new_id = questions.order_by(id.desc()).select(id).first(connection)?;
                for t in tags.iter() {
                    insert_into(chosen_tags)
                        .values((question.eq(new_id), tag.eq(t)))
                        .execute(connection)?;
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

    /// Select a question by id
    pub(crate) async fn question(&self, qid: i32) -> Result<DisplayQuestion, (Status, String)> {
        use crate::db::schema::questions::dsl::*;
        use crate::db::schema::users::dsl::{username, users};
        let question = self
            .run(move |connection| {
                questions
                    .inner_join(users)
                    .filter(id.eq(qid))
                    .select((id, username, time, score, title, text))
                    .first(connection)
            })
            .await
            .map_err(|e: Error| match e {
                Error::NotFound => (Status::NotFound, "This question does not exist".into()),
                e => internal_error(e),
            })?;
        self.to_display_question(question).await
    }

    /// Select all answers of a given question
    pub(crate) async fn answers(&self, qid: i32) -> Result<Vec<Answer>, (Status, String)> {
        use crate::db::schema::answers::dsl::*;
        use crate::db::schema::users::dsl::{username, users};
        self.run(move |connection| {
            answers
                .inner_join(users)
                .filter(question.eq(qid))
                .order_by(score.desc())
                .select((id, username, question, time, score, accepted, text))
                .load(connection)
        })
        .await
        .map_err(internal_error)
    }

    /// Add a new answer to the database.
    pub(crate) async fn new_answer(
        &self,
        author: i32,
        question: i32,
        text: String,
    ) -> Result<(), (Status, String)> {
        use crate::db::schema::answers::dsl::answers;

        let new = NewAnswer {
            author,
            question,
            text,
        };
        self.run(move |connection| insert_into(answers).values(new).execute(connection))
            .await
            .map_err(|e: Error| match e {
                Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                    (Status::BadRequest, "Invalid question id supplied".into())
                }
                e => internal_error(e),
            })?;
        Ok(())
    }

    /// Update the score of the question by the given difference.
    pub(crate) async fn update_question_score(
        &self,
        q_id: i32,
        diff: i32,
    ) -> Result<(), (Status, String)> {
        use crate::db::schema::questions::dsl::{id, questions, score};

        self.run(move |connection| {
            update(questions.filter(id.eq(q_id)))
                .set(score.eq(score + diff))
                .execute(connection)
        })
        .await
        .map_err(|e: Error| match e {
            Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                (Status::BadRequest, "Invalid question id supplied".into())
            }
            e => internal_error(e),
        })?;
        Ok(())
    }

    /// Update the score of the answer by the given difference.
    pub(crate) async fn update_answer_score(
        &self,
        a_id: i32,
        diff: i32,
    ) -> Result<(), (Status, String)> {
        use crate::db::schema::answers::dsl::{answers, id, score};

        self.run(move |connection| {
            update(answers.filter(id.eq(a_id)))
                .set(score.eq(score + diff))
                .execute(connection)
        })
        .await
        .map_err(|e: Error| match e {
            Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                (Status::BadRequest, "Invalid answer id supplied".into())
            }
            e => internal_error(e),
        })?;
        Ok(())
    }

    /// Mark an answer as solved.
    pub(crate) async fn mark_solved(&self, a_id: i32) -> Result<(), (Status, String)> {
        use crate::db::schema::answers::dsl::{accepted, answers, id};

        self.run(move |connection| {
            update(answers.filter(id.eq(a_id)))
                .set(accepted.eq(true))
                .execute(connection)
        })
        .await
        .map_err(|e: Error| match e {
            Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                (Status::BadRequest, "Invalid answer id supplied".into())
            }
            e => internal_error(e),
        })?;
        Ok(())
    }
}
