// get_coffee_market_indicator.rs
use std::sync::Arc;
use anyhow::Result;

use crate::domain::traits::coffee_market_provider::CoffeeMarketProvider;
use crate::domain::entities::coffee_market_indicators::CoffeeMarketIndicators;
use crate::domain::traits::cache_repository::CacheRepository;
use crate::domain::traits::clock::Clock;

/*
  Use Case para obtener los indicadores del mercado del café
*/
pub struct GetCoffeeMarketIndicatorsUseCase<P, C, T>
where
  P: CoffeeMarketProvider,
  C: CacheRepository<CoffeeMarketIndicators>,
  T: Clock
{
  provider: Arc<P>,
  cache: Arc<C>,
  clock: Arc<T>,
  ttl_seconds: u64
}

// Pasamos a la implementación
impl<P, C, T> GetCoffeeMarketIndicatorsUseCase<P, C, T>
where
  P: CoffeeMarketProvider,
  C: CacheRepository<CoffeeMarketIndicators>,
  T: Clock,
{
  pub fn new(provider: Arc<P>, cache: Arc<C>, clock: Arc<T>, ttl_seconds: u64) -> Self {
    Self { provider, cache, clock, ttl_seconds,  }
  }

  pub async fn execute(&self) -> Result<CoffeeMarketIndicators> {
    let cache_key = "coffee:market:indicators";

    // Intentamos obtener el precio del café de la caché
    if let Some(cached) = self.cache.get(cache_key).await? {
      let today = self.clock.today();

      /*
        Si el precio del café de la caché es de hoy, lo devolvemos
        Si es de ayer, lo obtenemos del proveedor
       */
      if cached.publication_date == today {
        return Ok(cached);
      }
    }
    let report = self.provider.get_market_indicators().await?;

    // Guardamos el precio en la caché
    self.cache
      .set(cache_key, &report, self.ttl_seconds)
      .await?;

    Ok(report)
  }
}

// Tests de la implementación
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::sync::{Arc, Mutex};

    // ---------- MOCK PROVIDER ----------

    struct MockProvider {
        pub response: CoffeeMarketIndicators,
        pub called: Arc<Mutex<bool>>,
    }

    #[async_trait::async_trait]
    impl CoffeeMarketProvider for MockProvider {
        async fn get_market_indicators(&self) -> Result<CoffeeMarketIndicators> {
            *self.called.lock().unwrap() = true;
            Ok(self.response.clone())
        }
    }

    // ---------- MOCK CACHE ----------

    struct MockCache {
        pub stored: Arc<Mutex<Option<CoffeeMarketIndicators>>>,
    }

    #[async_trait::async_trait]
    impl CacheRepository<CoffeeMarketIndicators> for MockCache {
        async fn get(&self, _key: &str) -> Result<Option<CoffeeMarketIndicators>> {
            Ok(self.stored.lock().unwrap().clone())
        }

        async fn set(
            &self,
            _key: &str,
            value: &CoffeeMarketIndicators,
            _ttl: u64,
        ) -> Result<()> {
            *self.stored.lock().unwrap() = Some(value.clone());
            Ok(())
        }
    }

    // ---------- MOCK CLOCK ----------

    struct FixedClock {
        pub date: NaiveDate,
    }

    impl Clock for FixedClock {
        fn today(&self) -> NaiveDate {
            self.date
        }
    }

    // ---------- MOCK ERROR PROVIDER ----------
    
    struct FailingProvider;

    #[async_trait::async_trait]
    impl CoffeeMarketProvider for FailingProvider {
        async fn get_market_indicators(&self) -> Result<CoffeeMarketIndicators> {
            Err(anyhow::anyhow!("provider error"))
        }
    }

    // ---------- MOCK ERROR CACHE GET ----------

    struct FailingGetCache;

    #[async_trait::async_trait]
    impl CacheRepository<CoffeeMarketIndicators> for FailingGetCache {
        async fn get(&self, _key: &str) -> Result<Option<CoffeeMarketIndicators>> {
            Err(anyhow::anyhow!("cache get error"))
        }

        async fn set(
            &self,
            _key: &str,
            _value: &CoffeeMarketIndicators,
            _ttl: u64,
        ) -> Result<()> {
            Ok(())
        }
    }

    // ---------- MOCK ERROR CACHE SET ----------

    struct FailingSetCache {
        pub stored: Arc<Mutex<Option<CoffeeMarketIndicators>>>,
    }

    #[async_trait::async_trait]
    impl CacheRepository<CoffeeMarketIndicators> for FailingSetCache {
        async fn get(&self, _key: &str) -> Result<Option<CoffeeMarketIndicators>> {
            Ok(self.stored.lock().unwrap().clone())
        }

        async fn set(
            &self,
            _key: &str,
            _value: &CoffeeMarketIndicators,
            _ttl: u64,
        ) -> Result<()> {
            Err(anyhow::anyhow!("cache set error"))
        }
    }

    // ---------- TEST 1 ----------
    // Cache vacío → debe llamar al provider

    #[tokio::test]
    async fn should_call_provider_when_cache_is_empty() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();
        let provider_called = Arc::new(Mutex::new(false));

        let provider = Arc::new(MockProvider {
            response: CoffeeMarketIndicators::new(
                today,
                1_500_000.0,
                120_000.0,
                2.15,
                3900.0,
                1_480_000.0,
            ),
            called: provider_called.clone(),
        });

        let cache = Arc::new(MockCache {
            stored: Arc::new(Mutex::new(None)),
        });

        let clock = Arc::new(FixedClock { date: today });

        let use_case =
            GetCoffeeMarketIndicatorsUseCase::new(provider, cache, clock, 3600);

        let result = use_case.execute().await.unwrap();

        assert_eq!(result.publication_date, today);
        assert_eq!(*provider_called.lock().unwrap(), true);
    }

    // ---------- TEST 2 ----------
    // Cache válido (fecha de hoy) → NO debe llamar al provider

    #[tokio::test]
    async fn should_return_cached_value_when_publication_date_is_today() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();
        let provider_called = Arc::new(Mutex::new(false));

        let provider = Arc::new(MockProvider {
            response: CoffeeMarketIndicators::new(
                today,
                9_999_999.0, // si lo llama, lo notamos
                9_999_999.0,
                9.99,
                9999.0,
                9_999_999.0,
            ),
            called: provider_called.clone(),
        });

        let cached_value = CoffeeMarketIndicators::new(
            today,
            1_500_000.0,
            120_000.0,
            2.15,
            3900.0,
            1_480_000.0,
        );

        let cache = Arc::new(MockCache {
            stored: Arc::new(Mutex::new(Some(cached_value.clone()))),
        });

        let clock = Arc::new(FixedClock { date: today });
        let use_case =
            GetCoffeeMarketIndicatorsUseCase::new(provider, cache, clock, 3600);

        let result = use_case.execute().await.unwrap();

        assert_eq!(result.internal_price_cop, 1_500_000.0);
        assert_eq!(*provider_called.lock().unwrap(), false);
    }

    // ---------- TEST 3 ----------
    // Cache con fecha vieja → debe llamar al provider y actualizar cache

    #[tokio::test]
    async fn should_refresh_when_cached_publication_date_is_old() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();
        let yesterday = NaiveDate::from_ymd_opt(2025, 1, 9).unwrap();

        let provider_called = Arc::new(Mutex::new(false));

        let new_report = CoffeeMarketIndicators::new(
            today,
            1_700_000.0,
            130_000.0,
            2.30,
            4000.0,
            1_650_000.0,
        );

        let provider = Arc::new(MockProvider {
            response: new_report.clone(),
            called: provider_called.clone(),
        });

        // Cache tiene datos de ayer
        let old_cached_value = CoffeeMarketIndicators::new(
            yesterday,
            1_500_000.0,
            120_000.0,
            2.10,
            3900.0,
            1_480_000.0,
        );

        let cache_storage = Arc::new(Mutex::new(Some(old_cached_value)));

        let cache = Arc::new(MockCache {
            stored: cache_storage.clone(),
        });

        let clock = Arc::new(FixedClock { date: today });
        let use_case =
            GetCoffeeMarketIndicatorsUseCase::new(provider, cache, clock, 3600);

        let result = use_case.execute().await.unwrap();

        // Debe haber llamado al provider
        assert_eq!(*provider_called.lock().unwrap(), true);

        // Debe devolver el nuevo reporte
        assert_eq!(result.publication_date, today);
        assert_eq!(result.internal_price_cop, 1_700_000.0);

        // Debe haber actualizado el cache
        let cached_after = cache_storage.lock().unwrap().clone().unwrap();
        assert_eq!(cached_after.publication_date, today);
    }

    // ---------- TEST 4 ----------
    // Provider falla → debe devolver error
    #[tokio::test]
    async fn should_propagate_error_when_provider_fails() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();

        let provider = Arc::new(FailingProvider);
        let cache = Arc::new(MockCache {
            stored: Arc::new(Mutex::new(None)),
        });
        let clock = Arc::new(FixedClock { date: today });

        let use_case =
            GetCoffeeMarketIndicatorsUseCase::new(provider, cache, clock, 3600);

        let result = use_case.execute().await;

        assert!(result.is_err());
    }

    // ---------- TEST 5 ----------
    // Cache GET falla → debe devolver error
    #[tokio::test]
    async fn should_propagate_error_when_cache_get_fails() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();

        let provider = Arc::new(MockProvider {
            response: CoffeeMarketIndicators::new(
                today, 1.0, 1.0, 1.0, 1.0, 1.0
            ),
            called: Arc::new(Mutex::new(false)),
        });

        let cache = Arc::new(FailingGetCache);
        let clock = Arc::new(FixedClock { date: today });

        let use_case =
            GetCoffeeMarketIndicatorsUseCase::new(provider, cache, clock, 3600);

        let result = use_case.execute().await;

        assert!(result.is_err());
    }

    // ---------- TEST 6 ----------
    // Cache SET falla → debe devolver error
    #[tokio::test]
    async fn should_propagate_error_when_cache_set_fails() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();

        let provider = Arc::new(MockProvider {
            response: CoffeeMarketIndicators::new(
                today, 1.0, 1.0, 1.0, 1.0, 1.0
            ),
            called: Arc::new(Mutex::new(false)),
        });

        let cache = Arc::new(FailingSetCache {
            stored: Arc::new(Mutex::new(None)),
        });

        let clock = Arc::new(FixedClock { date: today });

        let use_case =
            GetCoffeeMarketIndicatorsUseCase::new(provider, cache, clock, 3600);

        let result = use_case.execute().await;

        assert!(result.is_err());
    }
}