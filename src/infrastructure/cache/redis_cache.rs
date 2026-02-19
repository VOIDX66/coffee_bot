use anyhow::Result;
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::Mutex;

use crate::domain::traits::cache_repository::CacheRepository;

pub struct RedisCache {
    connection: Mutex<MultiplexedConnection>,
}

impl RedisCache {
    // Ahora async porque abrir la conexión es async
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let connection = client.get_multiplexed_async_connection().await?;
        Ok(Self { connection: Mutex::new(connection) })
    }
}

#[async_trait]
impl<T> CacheRepository<T> for RedisCache
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
{
    async fn get(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.connection.lock().await;
        let value: Option<String> = conn.get(key).await?;

        if let Some(json) = value {
            let deserialized = serde_json::from_str::<T>(&json)?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<()> {
        let json = serde_json::to_string(value)?;
        let mut conn = self.connection.lock().await;

        let _: () = conn.set_ex(key, json, ttl_seconds).await?;

        Ok(())
    }
}