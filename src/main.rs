mod db;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
use rocket_sync_db_pools::database;
use rocket::request::FromRequest;
use rocket::Request;
use rocket::request::Outcome;
use rocket::outcome::IntoOutcome;
use rocket::http::{CookieJar, Cookie};

#[database("rust_overflow")]
struct DbConn(diesel::SqliteConnection);

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

pub(crate) struct LoggedIn {
    id: i32
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LoggedIn {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        request
            .cookies()
            .get_private("User")
            .map(|c| LoggedIn { id: c.value().parse().unwrap() })
            .or_forward(())
    }
}

#[get("/login/<name>/<password>")]
async fn login(conn: DbConn, name: String, password: String, cookies: &CookieJar<'_>) -> Result<String, String> {
    let id = conn.login(name, password).await?;
    cookies.add_private(Cookie::new("User", id.to_string()));
    Ok(format!("{:?}", id))
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> &'static str {
    cookies.get_private("User").map(|c| cookies.remove_private(c));
    "Logged out"
}

#[get("/register/<name>/<password>")]
async fn register(conn: DbConn, name: String, password: String, cookies: &CookieJar<'_>) -> Result<String, String> {
    let id = conn.register(name, password).await?;
    cookies.add_private(Cookie::new("User", id.to_string()));
    Ok(format!("{:?}", id))
}

#[get("/restricted")]
async fn restricted(login: LoggedIn) -> Result<String, String> {
    Ok(format!("Logged in as: {:?}", login.id))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, login, register, restricted, logout])
        .attach(DbConn::fairing())
}
