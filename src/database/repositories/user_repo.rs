use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::Collection;
use crate::database::repositories::{SelectRepository, InsertRepository, UpdateRepository};
use crate::model::user::{CreateUser, CreateUserError, FetchUserError, SelectUserById, SelectUserByName, UpdateUser, User};

pub struct UserRepository {
    context: Collection<User>,
}


impl UserRepository {
    pub fn new(context: Collection<User>) -> Self {
        Self {
            context,
        }
    }
}

impl UserRepository {
    pub fn get_context(&self) -> &Collection<User> {
        &self.context
    }
}

#[async_trait]
impl SelectRepository<SelectUserByName<'_>, User, FetchUserError> for UserRepository {
    async fn select(&self, data: &SelectUserByName) -> Result<User, FetchUserError> {
        if let Ok(Some(user)) = self.context.find_one(doc! { "name": &data.username }, None).await {
            return Ok(user);
        }

        Err(FetchUserError::UserNotFound)
    }
}

#[async_trait]
impl SelectRepository<SelectUserById, User, FetchUserError> for UserRepository {
    async fn select(&self, data: &SelectUserById) -> Result<User, FetchUserError> {
        if let Ok(Some(user)) = self.context.find_one(doc! { "_id": &data.id }, None).await {
            return Ok(user);
        }

        Err(FetchUserError::UserNotFound)
    }
}

#[async_trait]
impl InsertRepository<CreateUser, User, CreateUserError> for UserRepository {
    async fn insert(&self, create_user: CreateUser) -> Result<User, CreateUserError> {
        if let Ok(Some(_)) = self.context.find_one(doc! { "name": &create_user.username }, None).await {
            return Err(CreateUserError::UserNameTaken);
        }

        let user = User::try_from(create_user)?;
        self.context.insert_one(&user, None).await?;

        Ok(user)
    }
}

#[async_trait]
impl UpdateRepository<UpdateUser, User, FetchUserError> for UserRepository {
    async fn update(&self, data: &UpdateUser) -> Result<User, FetchUserError> {
        if let Ok(Some(user)) = self.context.find_one_and_update( doc! { "_id": &data.target_id }, doc! { "$set": { "description": &data.new_description }}, None).await {
            return Ok(user);
        };

        Err(FetchUserError::UserNotFound)
    }
}
