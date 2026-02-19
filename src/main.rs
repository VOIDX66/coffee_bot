mod domain;
mod infrastructure;
mod application;

#[allow(unused_imports)]
use infrastructure::{
    providers::mock_provider::MockCoffeePriceProvider,
    providers::scraper_provider::ScraperCoffeeProvider,
    cache::in_memory_cache::InMemoryCache,
    cache::redis_cache::RedisCache
};
#[allow(unused_imports)]
use domain::traits::coffee_price_provider::CoffeePriceProvider;
use application::use_cases::get_coffee_price::GetCoffeePriceUseCase;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //let provider = Arc::new(MockCoffeePriceProvider);
    let provider = Arc::new(ScraperCoffeeProvider::new());
    //let cache = Arc::new(InMemoryCache::new());
    let cache = Arc::new(RedisCache::new("redis://127.0.0.1/").await?);

    let use_case = GetCoffeePriceUseCase::new(provider, cache, 600);

    let price1 = use_case.execute().await?;
    println!("Primera llamada: {:?}", price1);

    let price2 = use_case.execute().await?;
    println!("Segunda llamada (cache): {:?}", price2);

    Ok(())
}