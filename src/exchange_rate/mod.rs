use chrono::{DateTime, Utc};

pub enum ExchangeRateSource {
    /// Manually entered
    Manual,
    /// From the internet (string indicating the source)
    Internet(String),
}

#[derive(Debug, Clone)]
pub struct ExchangeRate {
    /// The datetime that this exchange rate represents
    datetime: DateTime<Utc>,
    /// The datetime that this exchange rate was obtained.
    obtained_datetime: Option<DateTime<Utc>>,
}
