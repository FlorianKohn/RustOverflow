mod backend;
mod db;
mod frontend;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

use crate::db::DbConn;
use chrono::NaiveDateTime;
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext,
};
use rocket_dyn_templates::Template;
use rocket_sass_fairing::SassSheet;

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
            routes![
                frontend::index,
                frontend::tagged_question,
                frontend::thread,
                backend::login,
                backend::register,
                backend::logout,
                backend::ask_question,
                style
            ],
        )
        .attach(DbConn::fairing())
        .attach(Template::custom(|engines| {
            engines
                .handlebars
                .register_helper("to_duration", Box::new(datetime_helper));
        }))
        .attach(SassSheet::fairing())
}
