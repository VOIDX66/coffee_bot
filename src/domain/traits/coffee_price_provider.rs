// coffe_price_providers.rs
use async_trait::async_trait;
use anyhow::Result;

use crate::domain::entities::coffee_price::CoffeePrice;

#[async_trait]
pub trait CoffeePriceProvider: Send + Sync {
  async fn get_price(&self) -> Result<CoffeePrice>;
}