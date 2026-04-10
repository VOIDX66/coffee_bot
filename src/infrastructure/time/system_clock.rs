use chrono::Datelike;
// system_clock.rs
use chrono::Local;
use chrono::NaiveDate;
use chrono::Weekday;

use crate::domain::traits::clock::Clock;

pub struct SystemClock;

impl Clock for SystemClock {
    fn today(&self) -> NaiveDate {
        Local::now().date_naive()
    }

    fn is_market_day(&self) -> bool {
        let now = Local::now();
        match now.weekday() {
            Weekday::Sat | Weekday::Sun => false,
            _ => true,
        }
    }
}
