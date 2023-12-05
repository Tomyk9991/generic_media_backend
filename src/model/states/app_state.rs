use std::env::VarError;
use std::num::ParseIntError;
use crate::database::database_context::DatabaseContext;
use crate::database::database_context::Error as DBError;
#[derive(Clone)]
pub struct AppState {
    pub ip_port_tuple: (String, u16),
    pub jwt_secret: String,
    pub data_directory: String,
    pub db: DatabaseContext
}

impl AppState {
    pub async fn from_env() -> Result<Self, AppStateError> {
        let server_ip = std::env::var("SERVERIP")?;
        let server_port: u16 = std::env::var("SERVERPORT")?.parse()?;

        let database = DatabaseContext::new().await?;

        Ok(AppState {
            ip_port_tuple: (server_ip, server_port),
            jwt_secret: std::env::var("JWT_SECRET")?,
            data_directory: std::env::var("DATADIRECTORY")?,
            db: database,
        })
    }
}


#[derive(Debug)]
pub enum AppStateError {
    Var(VarError),
    ParseInt(ParseIntError),
    IO(std::io::Error),
    Database(DBError)
}


impl From<DBError> for AppStateError { fn from(value: DBError) -> Self { AppStateError::Database(value) } }

impl From<std::io::Error> for AppStateError { fn from(value: std::io::Error) -> Self { AppStateError::IO(value) } }

impl From<ParseIntError> for AppStateError { fn from(value: ParseIntError) -> Self { AppStateError::ParseInt(value) } }

impl From<VarError> for AppStateError { fn from(value: VarError) -> Self { AppStateError::Var(value) } }
