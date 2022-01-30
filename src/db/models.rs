
#[derive(Queryable, Debug, Clone)]
pub(crate) struct User {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) password: String,
}