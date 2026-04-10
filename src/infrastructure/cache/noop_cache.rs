use anyhow::Result;
use async_trait::async_trait;

use crate::domain::entities::coffee_market_indicators::CoffeeMarketIndicators;
use crate::domain::traits::cache_repository::CacheRepository;

pub struct NoopCache;

#[async_trait]
impl CacheRepository<CoffeeMarketIndicators> for NoopCache {
    async fn get(&self, _key: &str) -> Result<Option<CoffeeMarketIndicators>> {
        Ok(None)
    }

    async fn set(&self, _key: &str, _value: &CoffeeMarketIndicators, _ttl: u64) -> Result<()> {
        Ok(())
    }
}
