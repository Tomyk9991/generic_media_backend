use async_trait::async_trait;

pub mod user_repo;
pub mod friendship_repo;

#[async_trait]
pub trait InsertRepository<T, K, E>: Sized {
    async fn insert(&self, data: T) -> Result<K, E>;
}

#[async_trait]
pub trait SelectRepository<T, K, E>: Sized {
    async fn select(&self, data: &T) -> Result<K, E>;
}

#[async_trait]
pub trait UpdateRepository<T, K, E>: Sized {
    async fn update(&self, data: &T) -> Result<K, E>;
}

#[async_trait]
pub trait DeleteRepository<T, K, E>: Sized {
    async fn delete(&self, data: &T) -> Result<K, E>;
}