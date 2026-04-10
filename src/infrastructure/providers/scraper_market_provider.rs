use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::{Client, header};
use scraper::{Html, Selector};
use std::sync::Arc;

use crate::domain::entities::coffee_market_indicators::CoffeeMarketIndicators;
use crate::domain::traits::coffee_market_provider::CoffeeMarketProvider;

// Actualizamos los selectores para la nueva estructura de Elementor
struct ScraperSelectors {
    menu_item: Selector,
    title: Selector,
    content: Selector,
}

pub struct ScraperCoffeeMarketProvider {
    client: Client,
    selectors: Arc<ScraperSelectors>,
}

impl ScraperCoffeeMarketProvider {
    pub fn new() -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            ),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| Client::new());

        let selectors = ScraperSelectors {
            // Cada bloque de indicador en el nuevo diseño
            menu_item: Selector::parse(".e-n-menu-item").expect("Invalid menu item selector"),
            // El span que tiene el texto ej: "Precio interno de referencia: $2.220.000"
            title: Selector::parse(".e-n-menu-title-text").expect("Invalid title selector"),
            // El div desplegable que contiene la fecha
            content: Selector::parse(".e-n-menu-content").expect("Invalid content selector"),
        };

        Self {
            client,
            selectors: Arc::new(selectors),
        }
    }

    fn parse_money(value: &str) -> Result<f64> {
        // La lógica se mantiene igual de robusta:
        // Solo conservamos números y la coma (para decimales)
        let cleaned: String = value
            .chars()
            .filter(|c| c.is_digit(10) || *c == ',')
            .collect();

        let normalized = cleaned.replace(",", ".");

        normalized
            .parse::<f64>()
            .with_context(|| format!("No se pudo parsear el valor numérico: {}", value))
    }
}

#[async_trait]
impl CoffeeMarketProvider for ScraperCoffeeMarketProvider {
    async fn get_market_indicators(&self) -> Result<CoffeeMarketIndicators> {
        // Nota: Según el HTML que pasaste, la URL correcta ahora podría ser esta:
        let url = "https://federaciondecafeteros.org/publicaciones/";

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Error al conectar con la web de la Federación")?
            .text()
            .await?;

        let document = Html::parse_document(&response);

        let mut publication_date = None;
        let mut internal_price = None;
        let mut pasilla = None;
        let mut ny_price = None;
        let mut exchange_rate = None;
        let mut mecic = None;

        for item in document.select(&self.selectors.menu_item) {
            // 1. Parsear el Nombre y el Valor (Vienen juntos en el title)
            if let Some(title_elem) = item.select(&self.selectors.title).next() {
                let full_title = title_elem.inner_html().trim().to_string();

                // Dividimos el texto por los dos puntos ":"
                let parts: Vec<&str> = full_title.split(':').collect();
                if parts.len() >= 2 {
                    let name = parts[0].trim();
                    let value = parts[1..].join(":").trim().to_string(); // En caso de que haya más ":"

                    match name {
                        n if n.contains("Precio interno") => {
                            internal_price = Some(Self::parse_money(&value)?)
                        }
                        n if n.contains("Bolsa de NY") => {
                            ny_price = Some(Self::parse_money(&value)?)
                        }
                        n if n.contains("Tasa de cambio") => {
                            exchange_rate = Some(Self::parse_money(&value)?)
                        }
                        n if n.contains("MeCIC") => mecic = Some(Self::parse_money(&value)?),
                        n if n.contains("Pasilla") => pasilla = Some(Self::parse_money(&value)?),
                        _ => {}
                    }
                }
            }

            // 2. Extraer la fecha del contenido interno (Buscamos la palabra "Fecha:")
            // Solo lo hacemos si no la hemos encontrado todavía
            if publication_date.is_none() {
                if let Some(content_elem) = item.select(&self.selectors.content).next() {
                    let content_text = content_elem.text().collect::<Vec<_>>().join(" ");

                    if let Some(idx) = content_text.find("Fecha:") {
                        // Tomamos lo que sigue de "Fecha:", quitamos espacios y extraemos la primera palabra (la fecha en sí)
                        let date_str = content_text[idx + 6..]
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("");
                        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                            publication_date = Some(date);
                        }
                    }
                }
            }
        }

        // Validación de datos mínimos requeridos
        let date =
            publication_date.ok_or_else(|| anyhow!("No se encontró la fecha de publicación"))?;
        let price = internal_price.ok_or_else(|| anyhow!("No se encontró el precio interno"))?;

        Ok(CoffeeMarketIndicators::new(
            date,
            price,
            pasilla.unwrap_or(0.0),
            ny_price.unwrap_or(0.0),
            exchange_rate.unwrap_or(0.0),
            mecic.unwrap_or(0.0),
        ))
    }
}
