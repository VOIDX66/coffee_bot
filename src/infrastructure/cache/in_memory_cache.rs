use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use async_trait::async_trait;

use crate::domain::traits::cache_repository::CacheRepository;

pub struct InMemoryCache<T> {
    store: Arc<RwLock<HashMap<String, T>>>,
}
#[allow(dead_code)]
impl<T> InMemoryCache<T> {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl<T> CacheRepository<T> for InMemoryCache<T>
where
    T: Clone + Send + Sync + 'static,
{
    async fn get(&self, key: &str) -> Result<Option<T>> {
        let store = self.store.read().await;
        Ok(store.get(key).cloned())
    }

    async fn set(&self, key: &str, value: &T, _ttl_seconds: u64) -> Result<()> {
        let mut store = self.store.write().await;
        store.insert(key.to_string(), value.clone());
        Ok(())
    }
}