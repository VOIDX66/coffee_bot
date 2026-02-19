use anyhow::{anyhow, Result};
use chrono::Local;
use scraper::{Html, Selector};
use reqwest::Client;
use crate::domain::entities::coffee_price::CoffeePrice;
use crate::domain::traits::coffee_price_provider::CoffeePriceProvider;
use async_trait::async_trait;

pub struct ScraperCoffeeProvider {
    client: Client,
}

impl ScraperCoffeeProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl CoffeePriceProvider for ScraperCoffeeProvider {
    async fn get_price(&self) -> Result<CoffeePrice> {
        let url = "https://federaciondecafeteros.org/wp/"; // Ajusta si cambia la URL

        let response = self.client.get(url)
            .send()
            .await?
            .text()
            .await?;

        let document = Html::parse_document(&response);

        // Selector para el <li tabindex="1">
        let li_selector = Selector::parse(r#"li[tabindex="1"] strong"#)
            .map_err(|_| anyhow!("Selector inválido"))?;

        let element = document
            .select(&li_selector)
            .next()
            .ok_or_else(|| anyhow!("No se encontró el precio en la página"))?;

        let text = element.text().collect::<Vec<_>>().join("").trim().to_string();

        // Limpiar el texto para convertirlo a f64
        let cleaned = text
            .replace("$", "")
            .replace(".", "")
            .replace(",", "")
            .trim()
            .to_string();

        let value = cleaned
            .parse::<f64>()
            .map_err(|_| anyhow!("No se pudo parsear el precio"))?;

        let today = Local::now().date_naive();

        Ok(CoffeePrice::new(value, today))
    }
}