use rocket_dyn_templates::Template;
use serde::Serialize;
use crate::db::models::{Tag, DisplayQuestion, Login};
use crate::db::DbConn;
use rocket::http::Status;

#[derive(Debug, Clone, Serialize)]
struct QuestionsCtx {
    user: Option<String>,

    title: String,
    description: String,

    all_tags: Vec<Tag>,
    selected_tags: Vec<Tag>,

    num_questions: usize,
    questions: Vec<DisplayQuestion>,
}

#[get("/")]
pub(crate) async fn index(user: Option<Login>, conn: DbConn) -> Result<Template, (Status, String)> {
    let questions = conn.newest_questions().await?;
    Ok(Template::render(
        "questions",
        QuestionsCtx {
            user: user.map(|u| u.username),

            title: "New Questions".into(),
            description: "The latest questions on this board.".into(),

            all_tags: conn.all_tags().await?,
            selected_tags: vec![],

            num_questions: questions.len(),
            questions: questions,
        },
    ))
}

#[get("/t/<tags>")]
pub(crate) async fn tagged_question(user: Option<Login>, conn: DbConn, tags: String) -> Result<Template, (Status, String)> {
    let tag_names:Vec<String> = tags.split("+").map(String::from).collect();
    let tags= conn.tags_with_names(tag_names.clone()).await?;
    let questions = conn.questions_with_tag(tag_names.clone()).await?;
    Ok(Template::render(
        "questions",
        QuestionsCtx {
            user: user.map(|u| u.username),

            title: tag_names.join(", "),
            description: tags[0].description.clone(),

            all_tags: conn.all_tags().await?,
            selected_tags: tags,

            num_questions: questions.len(),
            questions: questions,
        },
    ))
}