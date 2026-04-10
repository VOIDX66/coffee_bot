mod application;
mod domain;
mod infrastructure;
use std::sync::Arc;

use infrastructure::cache::noop_cache::NoopCache;
use infrastructure::cache::redis_cache::RedisCache;
use infrastructure::providers::scraper_market_provider::ScraperCoffeeMarketProvider;
use infrastructure::time::system_clock::SystemClock;

use application::use_cases::get_coffee_market_indicators::GetCoffeeMarketIndicatorsUseCase;

use crate::domain::traits::cache_repository::CacheRepository;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = Arc::new(ScraperCoffeeMarketProvider::new());

    //let cache = Arc::new(RedisCache::new("redis://:Secure_Void_Redis_Pass@192.168.20.4").await?);
    let cache: Arc<dyn CacheRepository<_>> =
        match RedisCache::new("redis://:Secure_Void_Redis_Pass@192.168.20.4").await {
            Ok(redis) => {
                print!("Redis Connection OK\n");
                Arc::new(redis)
            }
            Err(e) => {
                print!("Redis Connection Error: {}\n", e);
                Arc::new(NoopCache)
            }
        };

    let clock = Arc::new(SystemClock);

    let use_case = GetCoffeeMarketIndicatorsUseCase::new(provider, cache, clock, 3600);

    // --- First Call (Provider/Scraper) ---
    let start1 = tokio::time::Instant::now();
    let report1 = use_case.execute().await?;
    let duration1 = start1.elapsed();

    println!("Primera llamada (Scraper): {:?}\n{:#?}", duration1, report1);
    println!("------------------------------------------");

    // --- Second Call (Redis Cache) ---
    let start2 = tokio::time::Instant::now();
    let report2 = use_case.execute().await?;
    let duration2 = start2.elapsed();

    println!("Segunda llamada (Cache): {:?}\n{:#?}", duration2, report2);

    // Simple math to show the improvement
    if duration1 > duration2 {
        println!(
            "\nEfficiency gain: {:.2}x faster",
            duration1.as_secs_f64() / duration2.as_secs_f64()
        );
    }

    Ok(())
}
