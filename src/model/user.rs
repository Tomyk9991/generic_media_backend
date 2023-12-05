use std::fmt::{Display, Formatter};
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{Error, SaltString};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub name: String,
    pub password_hash: String,
    pub is_bot: bool,
    pub description: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.name == "admin"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub is_bot: bool,
    pub description: String,
}

pub struct UpdateUser {
    pub target_id: ObjectId,
    pub new_description: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectUserByName<'a> {
    pub username: &'a str
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectUserById {
    pub id: ObjectId
}

#[derive(Debug)]
pub enum FetchUserError {
    UserNotFound
}

#[derive(Debug)]
pub enum CreateUserError {
    NotHashable(Error),
    DatabaseError(mongodb::error::Error),
    UserNameTaken
}

impl TryFrom<CreateUser> for User {
    type Error = CreateUserError;

    fn try_from(create_user: CreateUser) -> Result<Self, Self::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2.hash_password(create_user.password.as_bytes(), &salt)?
            .to_string();

        let now = Utc::now();

        Ok(Self {
            id: ObjectId::new(),
            name: create_user.username,
            password_hash,
            is_bot: create_user.is_bot,
            description: create_user.description,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }
}

impl Display for CreateUserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            CreateUserError::NotHashable(_) => "Something went wrong hashing",
            CreateUserError::DatabaseError(_) => "Internal",
            CreateUserError::UserNameTaken => "Username taken"
        })
    }
}

impl ResponseError for CreateUserError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateUserError::UserNameTaken => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<mongodb::error::Error> for CreateUserError {
    fn from(value: mongodb::error::Error) -> Self {
        CreateUserError::DatabaseError(value)
    }
}

impl From<Error> for CreateUserError {
    fn from(value: Error) -> Self {
        CreateUserError::NotHashable(value)
    }
}

impl Display for FetchUserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            FetchUserError::UserNotFound => "User not found",
        })
    }
}


impl ResponseError for FetchUserError {
    fn status_code(&self) -> StatusCode {
        match self {
            FetchUserError::UserNotFound => StatusCode::NOT_FOUND,
        }
    }
}

