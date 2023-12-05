use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friendship {
    #[serde(rename="_id")]
    pub friend_a: ObjectId,
    pub friend_b: ObjectId
}