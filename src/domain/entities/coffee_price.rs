// coffee_price.rs
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// Estructura para representar el precio del café
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CoffeePrice {
  pub value: f64,
  pub currency: Currency,
  pub date: NaiveDate
}

/*
  Estructura para representar la moneda del precio del café
  Posteriormente se podrán agregar otras monedas (USD, EUR, etc.)
*/
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Currency {
  COP,
}

impl CoffeePrice {
  pub fn new(value: f64, date: NaiveDate) -> Self {
    Self { value, currency: Currency::COP, date }
  }
}