use async_trait::async_trait;
use futures_util::StreamExt;
use mongodb::bson::doc;
use mongodb::Collection;
use crate::database::repositories::{InsertRepository, SelectRepository};
use crate::model::friend::{CreateFriendshipError, FetchFriendshipError};
use crate::model::friendship::{Friendship};
use crate::model::user::SelectUserById;

pub struct FriendshipRepository {
    context: Collection<Friendship>
}

impl FriendshipRepository {
    pub fn new(context: Collection<Friendship>) -> Self {
        Self {
            context
        }
    }
}

#[async_trait]
impl SelectRepository<SelectUserById, Vec<Friendship>, FetchFriendshipError> for FriendshipRepository {
    async fn select(&self, data: &SelectUserById) -> Result<Vec<Friendship>, FetchFriendshipError> {
        let mut friendships: Vec<Friendship> = vec![];

        let query = doc! {
            "$or": [
                { "_id": &data.id },
                { "friend_b": &data.id }
            ]
        };

        if let Ok(mut cursor) = self.context.find(query, None).await {
            while let Some(document) = cursor.next().await {
                let friendship: Friendship = document?;
                friendships.push(friendship);
            }


            return Ok(friendships)
        }


        return Err(FetchFriendshipError::UserNotFound)
    }
}

#[async_trait]
impl InsertRepository<Friendship, Friendship, CreateFriendshipError> for FriendshipRepository {
    async fn insert(&self, data: Friendship) -> Result<Friendship, CreateFriendshipError> {

        let query = doc! {
            "$or": [
                { "_id": &data.friend_a, "friend_b": &data.friend_b },
                { "_id": &data.friend_b, "friend_b": &data.friend_a }
            ]
        };

        if let Ok(Some(_)) = self.context.find_one(query, None).await {
            return Err(CreateFriendshipError::AlreadyFriends);
        }

        self.context.insert_one(&data, None).await?;

        Ok(data)
    }
}