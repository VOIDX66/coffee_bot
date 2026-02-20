mod application;
mod domain;
mod infrastructure;
use std::sync::Arc;

use infrastructure::providers::scraper_market_provider::ScraperCoffeeMarketProvider;
use infrastructure::cache::redis_cache::RedisCache;
use infrastructure::time::system_clock::SystemClock;

use application::use_cases::get_coffee_market_indicators::GetCoffeeMarketIndicatorsUseCase;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let provider = Arc::new(ScraperCoffeeMarketProvider::new());
    let cache = Arc::new(RedisCache::new("redis://127.0.0.1/").await?);
    let clock = Arc::new(SystemClock);

    let use_case = GetCoffeeMarketIndicatorsUseCase::new(
        provider,
        cache,
        clock,
        3600, // TTL 1 hora
    );

    let report1 = use_case.execute().await?;
    println!("Primera llamada:\n{:#?}", report1);

    let report2 = use_case.execute().await?;
    println!("Segunda llamada (cache):\n{:#?}", report2);

    Ok(())
}