// system_clock.rs
use chrono::Local;
use chrono::NaiveDate;

use crate::domain::traits::clock::Clock;

pub struct SystemClock;

impl Clock for SystemClock {
    fn today(&self) -> NaiveDate {
        Local::now().date_naive()
    }
}