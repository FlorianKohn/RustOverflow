use crate::db::models::{Login, NewUser, User, Tag, DisplayQuestion, Question};
use crate::db::DbConn;
use bcrypt::verify;
use diesel::result::{DatabaseErrorKind, Error};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, BoolExpressionMethods};
use rocket::http::Status;
use chrono::NaiveDateTime;
use diesel::NullableExpressionMethods;
use diesel::expression::count::count_star;
use std::future::Future;

fn internal_error<E>(_: E) -> (Status, String) {
    (Status::InternalServerError, "Database error".into())
}

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
        let verified = verify(login_pw, &db_user.password)
            .map_err(internal_error)?;

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

    pub(crate) async fn num_answers(&self, q_id: i32) ->  Result<i64, (Status, String)> {
        use crate::db::schema::answers::dsl::*;
        self.run(move |c| answers.filter(question.eq(q_id)).select(count_star()).first::<i64>(c)).await.map_err(internal_error)
    }

    pub(crate) async fn answered(&self, q_id: i32) ->  Result<bool, (Status, String)> {
        use crate::db::schema::answers::dsl::*;
        let res = self.run(move |c|
            answers.filter(question.eq(q_id).and(accepted.eq(true))).select(id).first::<i32>(c)
        ).await;

        match res {
            Ok(_) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(e) => Err(internal_error(e))
        }
    }

    pub(crate) async fn tags(&self, q_id: i32) ->  Result<Vec<Tag>, (Status, String)> {
        use crate::db::schema::tags::dsl::{name, description, tags, id};
        use crate::db::schema::chosen_tags::dsl::{chosen_tags, question};
        self.run(move |c|
            chosen_tags.filter(question.eq(q_id)).inner_join(tags).select((id, name, description)).load::<Tag>(c)
        ).await.map_err(internal_error)
    }

    pub(crate) async fn newest_questions(&self) -> Result<Vec<DisplayQuestion>, (Status, String)>
    {
        use crate::db::schema::questions::dsl::*;
        use crate::db::schema::users::dsl::{users, username};

        let new_questions: Vec<Question> = self.run(|c|
            questions
                .inner_join(users)
                .order_by(time.desc())
                .select((id, username, time, score, title, text))
                .load::<Question>(c)
        ).await.map_err(internal_error)?;

        let mut res: Vec<DisplayQuestion> = Vec::with_capacity(new_questions.len());
        for q in new_questions.into_iter() {
            let dq = DisplayQuestion{
                id: q.id,
                author: q.author,
                time: q.time,
                score: q.score,
                title: q.title,
                text: q.text,
                tags: self.tags(q.id).await?,
                num_answers: self.num_answers(q.id).await?,
                answered: self.answered(q.id).await?,
            };
            res.push(dq);
        }

        Ok(res)
    }
}
