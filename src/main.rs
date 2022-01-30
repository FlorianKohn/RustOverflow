mod db;
#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
use rocket_sync_db_pools::{database};
use db::schema;
use crate::db::models::User;
use diesel::RunQueryDsl;

#[database("rust_overflow")]
struct DbConn(diesel::SqliteConnection);

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/users")]
async fn get_users(conn: DbConn) -> String {
    use crate::db::schema::users::dsl::*;
    let db_users = conn.run(|c| users.load::<User>(c)).await.unwrap();
    format!("{:?}", db_users)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, get_users]).attach(DbConn::fairing())
}
