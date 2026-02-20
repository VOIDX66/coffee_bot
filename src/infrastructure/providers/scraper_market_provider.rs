// scraper_market_provider.rs
/*
  Scraper para obtener los indicadores del mercado del café
*/

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::domain::entities::coffee_market_indicators::CoffeeMarketIndicators;
use crate::domain::traits::coffee_market_provider::CoffeeMarketProvider;

pub struct ScraperCoffeeMarketProvider {
    client: Client,
}

impl ScraperCoffeeMarketProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn parse_money(value: &str) -> Result<f64> {
        // Elimina símbolo $
        let cleaned = value.trim().replace("$", "");

        // Quita separadores de miles (.)
        let no_thousands = cleaned.replace(".", "");

        // Reemplaza coma decimal por punto
        let normalized = no_thousands.replace(",", ".");

        Ok(normalized.parse::<f64>()?)
    }
}

#[async_trait]
impl CoffeeMarketProvider for ScraperCoffeeMarketProvider {
    async fn get_market_indicators(&self) -> Result<CoffeeMarketIndicators> {
        let url = "https://federaciondecafeteros.org/wp/";

        let response = self.client
            .get(url)
            .send()
            .await?
            .text()
            .await?;

        let document = Html::parse_document(&response);

        let li_selector =
            Selector::parse("#modal-indicadores .lista li")
                .map_err(|_| anyhow!("Selector inválido"))?;

        let name_selector =
            Selector::parse(".name")
                .map_err(|_| anyhow!("Selector inválido"))?;

        let value_selector =
            Selector::parse("strong")
                .map_err(|_| anyhow!("Selector inválido"))?;

        let mut publication_date: Option<NaiveDate> = None;
        let mut internal_price: Option<f64> = None;
        let mut pasilla: Option<f64> = None;
        let mut ny_price: Option<f64> = None;
        let mut exchange_rate: Option<f64> = None;
        let mut mecic: Option<f64> = None;

        for li in document.select(&li_selector) {
            let name = li
                .select(&name_selector)
                .next()
                .map(|n| n.text().collect::<Vec<_>>().join("").trim().to_string());

            let value = li
                .select(&value_selector)
                .next()
                .map(|v| v.text().collect::<Vec<_>>().join("").trim().to_string());

            if let (Some(name), Some(value)) = (name, value) {
                match name.as_str() {
                    "Fecha:" => {
                        publication_date =
                            Some(NaiveDate::parse_from_str(&value, "%Y-%m-%d")?);
                    }
                    "Precio interno de referencia:" => {
                        internal_price = Some(Self::parse_money(&value)?);
                    }
                    "Pasilla de finca:" => {
                        pasilla = Some(Self::parse_money(&value)?);
                    }
                    "Bolsa de NY:" => {
                        ny_price = Some(Self::parse_money(&value)?);
                    }
                    "Tasa de cambio:" => {
                        exchange_rate = Some(Self::parse_money(&value)?);
                    }
                    "MeCIC:" => {
                        mecic = Some(Self::parse_money(&value)?);
                    }
                    _ => {}
                }
            }
        }

        Ok(CoffeeMarketIndicators::new(
            publication_date.ok_or_else(|| anyhow!("Falta Fecha"))?,
            internal_price.ok_or_else(|| anyhow!("Falta Precio interno"))?,
            pasilla.unwrap_or(0.0),
            ny_price.unwrap_or(0.0),
            exchange_rate.unwrap_or(0.0),
            mecic.unwrap_or(0.0),
        ))
    }
}