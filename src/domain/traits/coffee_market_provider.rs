// coffee_market_providers.rs
use async_trait::async_trait;
use anyhow::Result;
use crate::domain::entities::coffee_market_indicators::CoffeeMarketIndicators;

#[async_trait]
pub trait CoffeeMarketProvider: Send + Sync {
  async fn get_market_indicators(&self) -> Result<CoffeeMarketIndicators>;
}