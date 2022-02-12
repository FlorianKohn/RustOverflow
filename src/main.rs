mod db;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

use crate::db::models::{Login, DisplayQuestion};
use crate::db::DbConn;
use chrono::NaiveDateTime;
use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::IntoOutcome;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::response::Redirect;
use rocket::Request;
use rocket_dyn_templates::handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext,
};
use rocket_dyn_templates::Template;
use rocket_sass_fairing::SassSheet;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct IndexCtx {
    user: Option<String>,
    questions: Vec<DisplayQuestion>,
}
#[get("/")]
async fn index(user: Option<Login>, conn: DbConn) -> Result<Template, (Status, String)> {
    let questions = conn.newest_questions().await?;
    Ok(Template::render(
        "index",
        IndexCtx {
            user: user.map(|u| u.username),
            questions: questions,
        },
    ))
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
struct LoginForm<'r> {
    username: &'r str,
    password: &'r str,
}

#[post("/login", data = "<login>")]
async fn login(
    conn: DbConn,
    cookies: &CookieJar<'_>,
    login: Form<LoginForm<'_>>,
) -> Result<Redirect, (Status, String)> {
    let login = conn
        .login(login.username.to_string(), login.password.to_string())
        .await?;
    cookies.add_private(Cookie::new("User", serde_json::to_string(&login).unwrap()));
    Ok(Redirect::to("/"))
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies
        .get_private("User")
        .map(|c| cookies.remove_private(c));
    Redirect::to("/")
}

#[derive(FromForm)]
struct RegisterForm<'r> {
    username: &'r str,
    password: &'r str,
    password_repeat: &'r str,
}
#[post("/register", data = "<register>")]
async fn register(
    conn: DbConn,
    cookies: &CookieJar<'_>,
    register: Form<RegisterForm<'_>>,
) -> Result<Redirect, (Status, String)> {
    if register.password != register.password_repeat {
        return Err((Status::BadRequest, "Passwords do not match!".into()));
    }
    let login = conn
        .register(register.username.to_string(), register.password.to_string())
        .await?;
    cookies.add_private(Cookie::new("User", serde_json::to_string(&login).unwrap()));
    Ok(Redirect::to("/"))
}

#[get("/restricted")]
async fn restricted(login: Login) -> Result<String, String> {
    Ok(format!("Logged in as: {:?}", login.id))
}

#[get("/bootstrap.css")]
async fn style(sheet: &SassSheet) -> &SassSheet {
    sheet
}

fn datetime_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    use chrono_humanize::HumanTime;

    let raw_value = h.param(0).unwrap().value();
    let created: NaiveDateTime = serde_json::from_str(&raw_value.to_string()).unwrap();
    let now = chrono::offset::Local::now().naive_local();

    let passed = created.signed_duration_since(now);

    out.write(&HumanTime::from(passed).to_string())?;
    Ok(())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/static", FileServer::from(relative!("static")).rank(3))
        .mount(
            "/",
            routes![index, login, register, restricted, logout, style],
        )
        .attach(DbConn::fairing())
        .attach(Template::custom(|engines| {
            engines
                .handlebars
                .register_helper("to_duration", Box::new(datetime_helper));
        }))
        .attach(SassSheet::fairing())
}
