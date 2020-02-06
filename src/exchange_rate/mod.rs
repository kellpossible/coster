extern crate chrono;
extern crate rust_decimal;
extern crate serde;
extern crate serde_json;
extern crate thiserror;

use crate::currency::{Commodity, CurrencyCode};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};
use thiserror::Error;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum ExchangeRateError {
    #[error("the currency {0} is not present in the exchange rate")]
    CurrencyNotPresent(CurrencyCode),
}

pub enum ExchangeRateSource {
    /// A local source
    Local,
    /// From the internet (string indicating the source)
    Internet(String),
}

// TODO: make serde a feature flag
#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeRate {
    /// The datetime that this exchange rate represents
    pub datetime: Option<DateTime<Utc>>,
    /// The datetime that this exchange rate was obtained.
    pub obtained_datetime: Option<DateTime<Utc>>,
    /// The base currency for the exchange rate
    pub base: CurrencyCode,
    rates: HashMap<CurrencyCode, Decimal>,
}

impl ExchangeRate {
    pub fn get_rate(&self, currency_code: &CurrencyCode) -> Option<&Decimal> {
        self.rates.get(currency_code)
    }

    pub fn convert(
        &self,
        commodity: Commodity,
        target_currency: CurrencyCode,
    ) -> Result<Commodity, ExchangeRateError> {
        if commodity.currency_code == self.base {
            match self.get_rate(&target_currency) {
                Some(rate) => return Ok(Commodity::new(rate * commodity.value, target_currency)),
                None => {}
            };
        }

        if target_currency == self.base {
            match self.get_rate(&commodity.currency_code) {
                Some(rate) => return Ok(Commodity::new(rate / commodity.value, target_currency)),
                None => {}
            };
        }

        let commodity_rate = match self.get_rate(&commodity.currency_code) {
            Some(rate) => rate,
            None => {
                return Err(ExchangeRateError::CurrencyNotPresent(
                    commodity.currency_code,
                ))
            }
        };

        let target_rate = match self.get_rate(&target_currency) {
            Some(rate) => rate,
            None => return Err(ExchangeRateError::CurrencyNotPresent(target_currency)),
        };

        let value = (commodity.value / commodity_rate) * target_rate;
        return Ok(Commodity::new(value, target_currency));
    }
}

#[cfg(test)]
mod tests {
    use super::{ExchangeRate, CurrencyCode};
    use serde_json;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_deserialize() {
        let data = r#"
            {
                "base": "AUD",
                "rates": {
                    "USD": 2.542,
                    "EU": "1.234"
                }
            }
            "#;

        let exchange_rate: ExchangeRate = serde_json::from_str(data).unwrap();
        let usd = CurrencyCode::from_str("USD").unwrap();
        let eu = CurrencyCode::from_str("EU").unwrap();

        assert_eq!("AUD", exchange_rate.base);
        assert_eq!(Decimal::from_str("2.542").unwrap(), *exchange_rate.get_rate(&usd).unwrap());
        assert_eq!(Decimal::from_str("1.234").unwrap(), *exchange_rate.get_rate(&eu).unwrap());
    }
}
