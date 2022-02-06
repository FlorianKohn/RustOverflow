use crate::db::schema::{answers, questions, users};
use bcrypt::hash;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Debug, Clone)]
pub(crate) struct User {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) password: String,
}

// A logged in user
#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Login {
    pub(crate) id: i32,
    pub(crate) username: String,
}
 impl From<User> for Login {
     fn from(u: User) -> Self {
         Login{
             id: u.id,
             username: u.username
         }
     }
 }

#[derive(Queryable, Debug, Clone)]
pub(crate) struct Tag {
    pub(crate) id: i32,
    pub(crate) name: String,
}

#[derive(Queryable, Debug, Clone)]
pub(crate) struct Question {
    pub(crate) id: i32,
    pub(crate) author: i32,
    pub(crate) time: SystemTime,
    pub(crate) score: i32,
    pub(crate) title: String,
    pub(crate) text: String,
}

#[derive(Queryable, Debug, Clone)]
pub(crate) struct Answer {
    pub(crate) id: i32,
    pub(crate) author: i32,
    pub(crate) question: i32,
    pub(crate) time: SystemTime,
    pub(crate) score: i32,
    pub(crate) accepted: bool,
    pub(crate) text: String,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "users"]
pub(crate) struct NewUser {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl NewUser {
    /// Creates a new user including hashing the password
    pub(crate) fn create(username: String, password: String) -> Result<Self, String> {
        // Hash the password using bcrypt
        let hash = hash(password, 12).map_err(|_| "Invalid Password".to_string())?;

        // Crate a new User
        Ok(NewUser {
            username,
            password: hash,
        })
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "questions"]
pub(crate) struct NewQuestion {
    pub(crate) author: i32,
    pub(crate) title: String,
    pub(crate) text: String,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "answers"]
pub(crate) struct NewAnswer {
    pub(crate) author: i32,
    pub(crate) question: i32,
    pub(crate) text: String,
}
