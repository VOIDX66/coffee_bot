use anyhow::Result;
use async_trait::async_trait;
use chrono::Local;

use crate::domain::entities::coffee_price::CoffeePrice;
use crate::domain::traits::coffee_price_provider::CoffeePriceProvider;

pub struct MockCoffeePriceProvider;

#[async_trait]
impl CoffeePriceProvider for MockCoffeePriceProvider {
    async fn get_price(&self) -> Result<CoffeePrice> {
        let today = Local::now().date_naive();

        // Precio simulado
        Ok(CoffeePrice::new(1_500_000.0, today))
    }
}