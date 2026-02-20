// clock.rs
use chrono::NaiveDate;

pub trait Clock {
  fn today(&self) -> NaiveDate;
}