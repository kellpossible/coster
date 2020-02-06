use crate::currency::{CurrencyCode};
use chrono::{DateTime, Utc};

pub enum ExchangeRateSource {
    /// A local source
    Local,
    /// From the internet (string indicating the source)
    Internet(String),
}

#[derive(Debug, Clone)]
pub struct ExchangeRate {
    /// The datetime that this exchange rate represents
    pub datetime: Option<DateTime<Utc>>,
    /// The datetime that this exchange rate was obtained.
    pub obtained_datetime: Option<DateTime<Utc>>,
    /// The base currency for the exchange rate
    pub base: CurrencyCode,
    pub rates: Vec<(CurrencyCode, f32)>,
}
