// get_coffee_price.rs
use std::sync::Arc;
use anyhow::Result;

use crate::domain::traits::{
  coffee_price_provider::CoffeePriceProvider,
  cache_repository::CacheRepository};
use crate::domain::entities::coffee_price::CoffeePrice;

pub struct GetCoffeePriceUseCase {
  provider : Arc<dyn CoffeePriceProvider>,
  cache: Arc<dyn CacheRepository<CoffeePrice>>,
  ttl_seconds: u64,
}

impl GetCoffeePriceUseCase {
  pub fn new(
    provider: Arc<dyn CoffeePriceProvider>,
    cache: Arc<dyn CacheRepository<CoffeePrice>>,
    ttl_seconds: u64
  ) -> Self {
    Self { provider, cache, ttl_seconds }
  }

  pub async fn execute(&self) -> Result<CoffeePrice> {
    let cache_key = "coffee:price:current";

    // Intentamos obtener el precio del café de la caché
    if let Some(cached) = self.cache.get(cache_key).await? {
      return Ok(cached);
    }

    // Si no se encuentra en la caché, lo obtenemos del proveedor
    let price = self.provider.get_price().await?;

    // Guardamos el precio en la caché
    self.cache
      .set(cache_key, &price, self.ttl_seconds)
      .await?;

    Ok(price)
  }
}