pub(crate) mod actions;
pub(crate) mod models;
pub(crate) mod schema;

use rocket_sync_db_pools::database;
#[database("rust_overflow")]
pub(crate) struct DbConn(diesel::SqliteConnection);
