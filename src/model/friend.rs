use std::fmt::{Display, Formatter};
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use serde::{Serialize};

#[derive(Serialize)]
pub struct Friend {
    pub user_name: String,
    pub profile_image: String,
    pub description: String,
    pub is_bot: bool,
    pub amount_posts: usize
}

#[derive(Debug)]
pub enum FetchFriendshipError {
    UserNotFound,
    DocumentNotParsable
}

#[derive(Debug)]
pub enum CreateFriendshipError {
    DatabaseError(mongodb::error::Error),
    AlreadyFriends
}


impl From<mongodb::error::Error> for CreateFriendshipError {
    fn from(value: mongodb::error::Error) -> Self {
        CreateFriendshipError::DatabaseError(value)
    }
}


impl Display for CreateFriendshipError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            CreateFriendshipError::DatabaseError(_) => "Internal",
            CreateFriendshipError::AlreadyFriends => "Conflict"
        })
    }
}

impl ResponseError for CreateFriendshipError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateFriendshipError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateFriendshipError::AlreadyFriends => StatusCode::CONFLICT
        }
    }
}

impl Display for FetchFriendshipError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            FetchFriendshipError::UserNotFound => "User not found",
            FetchFriendshipError::DocumentNotParsable => "Internal"
        })
    }
}


impl From<mongodb::error::Error> for FetchFriendshipError {
    fn from(_: mongodb::error::Error) -> Self {
        FetchFriendshipError::DocumentNotParsable
    }
}

impl ResponseError for FetchFriendshipError {
    fn status_code(&self) -> StatusCode {
        match self {
            FetchFriendshipError::UserNotFound => StatusCode::NOT_FOUND,
            FetchFriendshipError::DocumentNotParsable => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}