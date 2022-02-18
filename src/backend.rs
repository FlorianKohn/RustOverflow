use crate::db::models::Login;
use crate::db::DbConn;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Outcome};
use rocket::response::Redirect;
use rocket::Request;

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

#[derive(Debug, FromForm)]
pub(crate) struct LoginForm<'r> {
    username: &'r str,
    password: &'r str,
}

#[post("/login", data = "<login>")]
pub(crate) async fn login(
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
pub(crate) async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies
        .get_private("User")
        .map(|c| cookies.remove_private(c));
    Redirect::to("/")
}

#[derive(FromForm)]
pub(crate) struct RegisterForm<'r> {
    username: &'r str,
    password: &'r str,
    password_repeat: &'r str,
}
#[post("/register", data = "<register>")]
pub(crate) async fn register(
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

#[derive(Debug, FromForm)]
pub(crate) struct AskForm {
    title: String,
    question: String,
    tags: Vec<i32>,
}

#[post("/ask", data = "<question>")]
pub(crate) async fn ask_question(
    conn: DbConn,
    question: Form<AskForm>,
    user: Login,
) -> Result<Redirect, (Status, String)> {
    use crate::frontend::rocket_uri_macro_thread;
    let AskForm {
        title,
        question,
        tags,
    } = question.into_inner();
    let new_id = conn.new_question(user.id, title, question, tags).await?;
    Ok(Redirect::to(uri!(thread(id = new_id))))
}

#[derive(Debug, FromForm)]
pub(crate) struct AnswerForm {
    question: i32,
    text: String,
}

#[post("/answer", data = "<answer>")]
pub(crate) async fn answer_question(
    conn: DbConn,
    answer: Form<AnswerForm>,
    user: Login,
) -> Result<Redirect, (Status, String)> {
    use crate::frontend::rocket_uri_macro_thread;
    let AnswerForm {
        question,
        text,
    } = answer.into_inner();
    conn.new_answer(user.id, question, text).await?;
    Ok(Redirect::to(uri!(thread(id = question))))
}
