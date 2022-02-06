use crate::db::models::{NewUser, User, Login};
use bcrypt::{verify};
use diesel::result::{DatabaseErrorKind, Error};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use crate::db::DbConn;
use rocket::http::Status;

impl DbConn {
    pub(crate) async fn login(self, login_name: String, login_pw: String) -> Result<Login, (Status, String)> {
        use crate::db::schema::users::dsl::*;
        let db_user: User = self
            .run(|c| users.filter(username.eq(login_name)).first::<User>(c))
            .await
            .map_err(|e| (Status::BadRequest, e.to_string()))?;
        let verified =
            verify(login_pw, &db_user.password).map_err(|_| (Status::InternalServerError, "Database error".into()))?;

        verified.then(|| db_user.into()).ok_or((Status::Unauthorized, "wrong password".into()))
    }

    pub(crate) async fn register(self, username: String, password: String) -> Result<Login, (Status, String)> {
        use crate::db::schema::users::dsl::users;
        use diesel::insert_into;

        let user = NewUser::create(username.clone(), password.clone()).map_err(|reason| (Status::BadRequest, reason))?;

        // Insert User into db
        self.run(move |c| insert_into(users).values(&user).execute(c))
            .await
            .map_err(|e: Error| match e {
                Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    (Status::BadRequest, "Username already exists".into())
                }
                _ => (Status::InternalServerError, "Database error".into()),
            })?;

        // log user in after registration
        self.login(username, password).await
    }
}
