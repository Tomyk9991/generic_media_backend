use std::fmt::{Display, Formatter};
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use mongodb::error::Error;

pub mod states;
pub mod user;
pub mod friend;
pub mod friendship;


#[derive(Debug)]
pub enum UpdateDatabaseError {
    DatabaseError(mongodb::error::Error)
}

#[derive(Debug)]
pub enum SelectDatabaseError {
    DatabaseError(mongodb::error::Error)
}

#[derive(Debug)]
pub enum DeleteDatabaseError {
    DatabaseError(mongodb::error::Error)
}

#[derive(Debug)]
pub enum InsertDatabaseError {
    DatabaseError(mongodb::error::Error)
}


impl ResponseError for DeleteDatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteDatabaseError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ResponseError for InsertDatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            InsertDatabaseError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl From<InsertDatabaseError> for SelectDatabaseError {
    fn from(v: InsertDatabaseError) -> Self {
        match v {
            InsertDatabaseError::DatabaseError(e) => SelectDatabaseError::DatabaseError(e)
        }
    }
}

impl From<mongodb::error::Error> for DeleteDatabaseError {
    fn from(value: Error) -> Self {
        DeleteDatabaseError::DatabaseError(value)
    }
}

impl From<mongodb::error::Error> for InsertDatabaseError {
    fn from(value: Error) -> Self {
        InsertDatabaseError::DatabaseError(value)
    }
}

impl Display for InsertDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            InsertDatabaseError::DatabaseError(d) => d
        })
    }
}

impl Display for DeleteDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DeleteDatabaseError::DatabaseError(d) => d
        })
    }
}

impl Display for SelectDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SelectDatabaseError::DatabaseError(_) => "Internal selection error"
        })
    }
}

impl ResponseError for SelectDatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            SelectDatabaseError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl From<mongodb::error::Error> for UpdateDatabaseError {
    fn from(value: mongodb::error::Error) -> Self {
        UpdateDatabaseError::DatabaseError(value)
    }
}

impl Display for UpdateDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            UpdateDatabaseError::DatabaseError(_) => "Internal",
        })
    }
}

impl ResponseError for UpdateDatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            UpdateDatabaseError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}