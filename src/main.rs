mod db;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

use rocket::request::FromRequest;
use rocket::Request;
use rocket::request::Outcome;
use rocket::outcome::IntoOutcome;
use rocket::http::{CookieJar, Cookie, Status};
use rocket_dyn_templates::Template;
use rocket::fs::{relative, FileServer};
use rocket_sass_fairing::SassSheet;
use crate::db::DbConn;
use crate::db::models::Login;
use serde::Serialize;
use rocket::response::Redirect;
use rocket::form::Form;

#[derive(Debug, Clone, Serialize)]
struct IndexCtx {
    user: Option<String>,
}
#[get("/")]
fn index(user: Option<Login>) -> Template {
    Template::render(
        "index",
        IndexCtx{
            user: user.map(|u| u.username)
        },
    )
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Login {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        request
            .cookies()
            .get_private("User")
            .and_then(|c| serde_json::from_str(c.value()).ok())
            .or_forward(())
    }
}


#[derive(FromForm)]
struct LoginForm<'r>{
    username: &'r str,
    password: &'r str,
}

#[post("/login", data = "<login>")]
async fn login(conn: DbConn, cookies: &CookieJar<'_>, login: Form<LoginForm<'_>>) -> Result<Redirect, (Status, String)> {
    let login = conn.login(login.username.to_string(), login.password.to_string()).await?;
    cookies.add_private(Cookie::new("User", serde_json::to_string(&login).unwrap()));
    Ok(Redirect::to("/"))
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.get_private("User").map(|c| cookies.remove_private(c));
    Redirect::to("/")
}

#[derive(FromForm)]
struct RegisterForm<'r>{
    username: &'r str,
    password: &'r str,
    password_repeat: &'r str,
}
#[post("/register", data = "<register>")]
async fn register(conn: DbConn, cookies: &CookieJar<'_>, register: Form<RegisterForm<'_>>) -> Result<Redirect, (Status, String)> {
    if register.password != register.password_repeat {
        return Err((Status::BadRequest, "Passwords do not match!".into()));
    }
    let login = conn.register(register.username.to_string(), register.password.to_string()).await?;
    cookies.add_private(Cookie::new("User", serde_json::to_string(&login).unwrap()));
    Ok(Redirect::to("/"))
}

#[get("/restricted")]
async fn restricted(login: Login) -> Result<String, String> {
    Ok(format!("Logged in as: {:?}", login.id))
}

#[get("/bootstrap.css")]
async fn style(sheet: &SassSheet) -> &SassSheet { sheet }

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/static", FileServer::from(relative!("static")).rank(3))
        .mount("/", routes![index, login, register, restricted, logout, style])
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .attach(SassSheet::fairing())
}
