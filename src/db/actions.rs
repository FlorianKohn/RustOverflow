use crate::db::models::{NewUser, User};
use crate::DbConn;
use bcrypt::{verify};
use diesel::result::{DatabaseErrorKind, Error};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

impl DbConn {
    pub(crate) async fn login(self, login_name: String, login_pw: String) -> Result<i32, String> {
        use crate::db::schema::users::dsl::*;
        let db_user: User = self
            .run(|c| users.filter(username.eq(login_name)).first::<User>(c))
            .await
            .map_err(|e| e.to_string())?;
        let verified =
            verify(login_pw, &db_user.password).map_err(|_| "Database error".to_string())?;

        if verified {
            Ok(db_user.id)
        } else {
            Err("wrong password".into())
        }
    }

    pub(crate) async fn register(self, username: String, password: String) -> Result<i32, String> {
        use crate::db::schema::users::dsl::users;
        use diesel::insert_into;

        let user = NewUser::create(username.clone(), password.clone())?;

        // Insert User into db
        self.run(move |c| insert_into(users).values(&user).execute(c))
            .await
            .map_err(|e: Error| match e {
                Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    "Username already exists"
                }
                _ => "Database Error",
            })?;

        // log user in after registration
        self.login(username, password).await
    }
}
