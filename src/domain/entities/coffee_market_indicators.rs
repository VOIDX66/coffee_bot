// coffer_market_indicators.rs
/*
Estructura para representar los indicadores del mercado del café
*/
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CoffeeMarketIndicators {
    pub publication_date: NaiveDate,
    pub internal_price_cop: f64,
    pub pasilla_cop: f64,
    pub ny_price_usd: f64,
    pub exchange_rate_cop_usd: f64,
    pub mecic_cop: f64,
}

impl CoffeeMarketIndicators {
    pub fn new(
        publication_date: NaiveDate,
        internal_price_cop: f64,
        pasilla_cop: f64,
        ny_price_usd: f64,
        exchange_rate_cop_usd: f64,
        mecic_cop: f64,
    ) -> Self {
        Self {
            publication_date,
            internal_price_cop,
            pasilla_cop,
            ny_price_usd,
            exchange_rate_cop_usd,
            mecic_cop,
        }
    }
}