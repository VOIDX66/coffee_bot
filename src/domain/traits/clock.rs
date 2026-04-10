// clock.rs
use chrono::NaiveDate;

pub trait Clock {
    fn today(&self) -> NaiveDate;
    fn is_market_day(&self) -> bool;
}
