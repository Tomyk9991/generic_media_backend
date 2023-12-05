use std::fmt::{Display, Formatter};
use std::io::Error;
use std::num::ParseIntError;
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use crate::model::friend::{CreateFriendshipError, FetchFriendshipError};
use crate::model::{DeleteDatabaseError, InsertDatabaseError, SelectDatabaseError, UpdateDatabaseError};
use crate::model::user::FetchUserError;

pub mod list;

#[derive(Debug)]
pub enum GETError {
    CantRead,
    UserNotFound,
    Unauthorized,
    DatabaseError(mongodb::error::Error)
}

#[derive(Debug)]
pub enum UploadError {
    FileSizeTooBig,
    UserNotFound,
    IllegalContentType,
    CorruptedHeaderLength(ParseIntError),
    IOError(Error),
    WritingError,
    Unauthorized
}

#[derive(Debug)]
pub enum DeleteError {
    UserNotFound,
    DatabaseError(mongodb::error::Error),
    ContentNotFound(String),
}


impl From<DeleteDatabaseError> for DeleteError {
    fn from(value: DeleteDatabaseError) -> Self {
        match value {
            DeleteDatabaseError::DatabaseError(value) => DeleteError::DatabaseError(value)
        }
    }
}

impl From<InsertDatabaseError> for UploadError {
    fn from(value: InsertDatabaseError) -> Self {
        match value {
            InsertDatabaseError::DatabaseError(_) => UploadError::WritingError
        }
    }
}

impl From<InsertDatabaseError> for GETError {
    fn from(value: InsertDatabaseError) -> Self {
        match value {
            InsertDatabaseError::DatabaseError(e) => GETError::DatabaseError(e)
        }
    }
}

impl From<SelectDatabaseError> for GETError {
    fn from(value: SelectDatabaseError) -> Self {
        match value {
            SelectDatabaseError::DatabaseError(e) => GETError::DatabaseError(e)
        }
    }
}

impl From<UpdateDatabaseError> for DeleteError {
    fn from(value: UpdateDatabaseError) -> Self {
        match value {
            UpdateDatabaseError::DatabaseError(e) => DeleteError::ContentNotFound(e.to_string())
        }
    }
}

impl From<SelectDatabaseError> for DeleteError {
    fn from(value: SelectDatabaseError) -> Self {
        match value {
            SelectDatabaseError::DatabaseError(e) => DeleteError::DatabaseError(e)
        }
    }
}

impl From<FetchUserError> for DeleteError {
    fn from(value: FetchUserError) -> Self {
        match value {
            FetchUserError::UserNotFound => DeleteError::UserNotFound,
        }
    }
}


impl Display for UploadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            UploadError::CorruptedHeaderLength(_) => "The provided Content length header was not parsable to an integer".to_string(),
            UploadError::IOError(_) => "Internal".to_string(),
            UploadError::WritingError => "Internal".to_string(),
            UploadError::FileSizeTooBig => "File size is too big.".to_string(),
            UploadError::UserNotFound => "User not found".to_string(),
            UploadError::Unauthorized => "Unauthorized".to_string(),
            UploadError::IllegalContentType => "Illegal content type".to_string(),
        })
    }
}

impl Display for DeleteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DeleteError::UserNotFound => "User not found".to_string(),
            DeleteError::ContentNotFound(content) => format!("Content not found: {content}"),
            DeleteError::DatabaseError(_) => format!("Internal delete error")
        })
    }
}

impl ResponseError for DeleteError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteError::UserNotFound => StatusCode::NOT_FOUND,
            DeleteError::ContentNotFound(_) => StatusCode::NOT_FOUND,
            DeleteError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ResponseError for UploadError {
    fn status_code(&self) -> StatusCode {
        match self {
            UploadError::FileSizeTooBig => StatusCode::PAYLOAD_TOO_LARGE,
            UploadError::Unauthorized => StatusCode::UNAUTHORIZED,
            UploadError::UserNotFound => StatusCode::NOT_FOUND,
            UploadError::IllegalContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl From<UpdateDatabaseError> for UploadError {
    fn from(value: UpdateDatabaseError) -> Self {
        match value {
            UpdateDatabaseError::DatabaseError(_) => UploadError::WritingError,
        }
    }
}

impl From<SelectDatabaseError> for UploadError {
    fn from(value: SelectDatabaseError) -> Self {
        match value {
            SelectDatabaseError::DatabaseError(_) => UploadError::WritingError
        }
    }
}

impl From<CreateFriendshipError> for UploadError {
    fn from(value: CreateFriendshipError) -> Self {
        match value {
            CreateFriendshipError::DatabaseError(_) => UploadError::WritingError,
            CreateFriendshipError::AlreadyFriends => UploadError::WritingError
        }
    }
}

impl From<FetchFriendshipError> for UploadError {
    fn from(value: FetchFriendshipError) -> Self {
        match value {
            FetchFriendshipError::UserNotFound => Self::UserNotFound,
            FetchFriendshipError::DocumentNotParsable => Self::WritingError
        }
    }
}

impl From<FetchFriendshipError> for GETError {
    fn from(value: FetchFriendshipError) -> Self {
        match value {
            FetchFriendshipError::UserNotFound => Self::UserNotFound,
            FetchFriendshipError::DocumentNotParsable => Self::CantRead
        }
    }
}

impl From<GETError> for UploadError {
    fn from(value: GETError) -> Self {
        match value {
            GETError::CantRead => UploadError::WritingError,
            GETError::UserNotFound => UploadError::UserNotFound,
            GETError::Unauthorized => UploadError::Unauthorized,
            GETError::DatabaseError(_) => UploadError::WritingError
        }
    }
}

impl From<ParseIntError> for UploadError {
    fn from(value: ParseIntError) -> Self {
        UploadError::CorruptedHeaderLength(value)
    }
}

impl From<std::io::Error> for UploadError {
    fn from(value: std::io::Error) -> Self {
        UploadError::IOError(value)
    }
}

impl From<FetchUserError> for UploadError {
    fn from(_: FetchUserError) -> Self {
        UploadError::UserNotFound
    }
}

impl Display for GETError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            GETError::CantRead => "Not found",
            GETError::UserNotFound => "User not found",
            GETError::Unauthorized => "Unauthorized",
            GETError::DatabaseError(_) =>"Internal"
        })
    }
}

impl ResponseError for GETError {
    fn status_code(&self) -> StatusCode {
        match self {
            GETError::CantRead => StatusCode::NOT_FOUND,
            GETError::UserNotFound => StatusCode::NOT_FOUND,
            GETError::Unauthorized => StatusCode::UNAUTHORIZED,
            GETError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl From<std::io::Error> for GETError {
    fn from(_: Error) -> Self {
        GETError::CantRead
    }
}

impl From<FetchUserError> for GETError {
    fn from(_: FetchUserError) -> Self {
        GETError::UserNotFound
    }
}