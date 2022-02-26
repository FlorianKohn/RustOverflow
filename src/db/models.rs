use crate::db::schema::{answers, questions, users};
use bcrypt::hash;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Clone)]
pub(crate) struct User {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) password: String,
}

/// A logged in user
#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Login {
    pub(crate) id: i32,
    pub(crate) username: String,
}
impl From<User> for Login {
    fn from(u: User) -> Self {
        Login {
            id: u.id,
            username: u.username,
        }
    }
}

/// Represents a Tag in the Database
#[derive(Queryable, Serialize, Debug, Clone)]
pub(crate) struct Tag {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) description: String,
}

/// Represents a Question in the Database
/// The author is replaced with the username of the author
#[derive(Queryable, Serialize, Debug, Clone)]
pub(crate) struct Question {
    pub(crate) id: i32,
    pub(crate) author: String,
    pub(crate) time: NaiveDateTime,
    pub(crate) score: i32,
    pub(crate) title: String,
    pub(crate) text: String,
}

/// A collection of data concerning a question.
/// Suitable to generate HTMl for a question
#[derive(Serialize, Debug, Clone)]
pub(crate) struct DisplayQuestion {
    pub(crate) id: i32,
    pub(crate) author: String,
    pub(crate) time: NaiveDateTime,
    pub(crate) score: i32,
    pub(crate) title: String,
    pub(crate) text: String,
    pub(crate) tags: Vec<Tag>,
    pub(crate) num_answers: i64,
    pub(crate) answered: bool,
}

/// Represents an Answer in the Database
/// The author is replaced with the username of the author
#[derive(Queryable, Serialize, Debug, Clone)]
pub(crate) struct Answer {
    pub(crate) id: i32,
    pub(crate) author: String,
    pub(crate) question: i32,
    pub(crate) time: NaiveDateTime,
    pub(crate) score: i32,
    pub(crate) accepted: bool,
    pub(crate) text: String,
}

/// Represents the data needed to create a new User
/// I.e. it omits all fields of the `ùsers` table that are filled in with defaults.
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

/// Represents the data needed to create a new Question
/// I.e. it omits all fields of the `questions` table that are filled in with defaults.
#[derive(Insertable, Debug, Clone)]
#[table_name = "questions"]
pub(crate) struct NewQuestion {
    pub(crate) author: i32,
    pub(crate) title: String,
    pub(crate) text: String,
}

/// Represents the data needed to create a new Answer
/// I.e. it omits all fields of the `answers` table that are filled in with defaults.
#[derive(Insertable, Debug, Clone)]
#[table_name = "answers"]
pub(crate) struct NewAnswer {
    pub(crate) author: i32,
    pub(crate) question: i32,
    pub(crate) text: String,
}
