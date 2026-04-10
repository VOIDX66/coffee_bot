use anyhow::Result;
use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

use crate::domain::traits::cache_repository::CacheRepository;

pub struct RedisCache {
    connection: Mutex<MultiplexedConnection>,
    available: AtomicBool,
}

impl RedisCache {
    // Crear conexión async al iniciar
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let connection = client.get_multiplexed_async_connection().await?;
        Ok(Self {
            connection: Mutex::new(connection),
            available: AtomicBool::new(true),
        })
    }

    fn disable(&self) {
        self.available.store(false, Ordering::Relaxed);
    }

    fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl<T> CacheRepository<T> for RedisCache
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
{
    async fn get(&self, key: &str) -> Result<Option<T>> {
        if !self.is_available() {
            // Redis previamente marcado como no disponible
            return Ok(None);
        }

        let mut conn = self.connection.lock().await;

        let value: Option<String> = match conn.get(key).await {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Redis get failed: {}. Disabling cache.", e);
                self.disable();
                return Ok(None);
            }
        };

        if let Some(json) = value {
            let deserialized = serde_json::from_str::<T>(&json)?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<()> {
        if !self.is_available() {
            return Ok(()); // Redis deshabilitado → ignoramos
        }

        let mut conn = self.connection.lock().await;
        let json = serde_json::to_string(value)?;

        if let Err(e) = conn
            .set_ex::<&str, String, u64>(key, json, ttl_seconds)
            .await
        {
            log::warn!("Redis set failed: {}. Disabling cache.", e);
            self.disable();
        }

        Ok(())
    }
}
