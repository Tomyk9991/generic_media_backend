use std::env::VarError;
use std::fmt::{Display, Formatter};
use mongodb::{Client, Collection, Database};
use mongodb::bson::doc;
use mongodb::options::ClientOptions;
use crate::database::repositories::friendship_repo::FriendshipRepository;
use crate::database::repositories::user_repo::UserRepository;
use crate::model::friendship::Friendship;
use crate::model::user::User;

#[derive(Clone)]
pub struct DatabaseContext {
    _db: Database,
    users: Collection<User>,
    friendships: Collection<Friendship>
}

#[derive(Debug)]
pub enum Error {
    VarError(std::env::VarError),
    MongoDBError(mongodb::error::Error)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Error::VarError(v) => v.to_string(),
            Error::MongoDBError(v) => v.to_string()
        })
    }
}

impl From<std::env::VarError> for Error {
    fn from(value: VarError) -> Self {
        Error::VarError(value)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        Error::MongoDBError(value)
    }
}

impl std::error::Error for Error { }

impl DatabaseContext {
    pub async fn new() -> Result<Self, Error> {
        let client_options = ClientOptions::parse(
            std::env::var("MONGO_URL")?
        ).await?;

        let client = Client::with_options(client_options)?;

        let mongodb_name = std::env::var("MONGO_DATABASE")?;
        let db = client.database(&mongodb_name);
        db.run_command(doc! { "ping": 1 }, None)
            .await?;

        log::info!("Established connection the database");

        Ok(Self {
            _db: db.clone(),
            users: db.collection("users"),
            friendships: db.collection("friendships")
        })
    }

    pub fn friendship(&self) -> FriendshipRepository { 
        FriendshipRepository::new(self.friendships.clone()) 
    }
    
    pub fn user(&self) -> UserRepository {
        UserRepository::new(self.users.clone())
    }
}