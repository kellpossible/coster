//! A library with primatives representing money/commodities
//! ([Commodity](Commodity)), and their associated types
//! ([Currency](Currency)).

extern crate arrayvec;
extern crate chrono;
extern crate iso4217;
extern crate rust_decimal;
extern crate serde;

use arrayvec::ArrayString;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

pub const CURRENCY_CODE_LENGTH: usize = 8;
type CurrencyCodeArray = ArrayString<[u8; CURRENCY_CODE_LENGTH]>;

#[derive(Error, Debug, PartialEq)]
pub enum CurrencyError {
    #[error("This commodity {this_commodity:?} is incompatible with {other_commodity:?} because {reason:?}")]
    IncompatableCommodity {
        this_commodity: Commodity,
        other_commodity: Commodity,
        reason: String,
    },
    #[error(
        "The currency code {0} is too long. Maximum of {} characters allowed.",
        CURRENCY_CODE_LENGTH
    )]
    TooLongCurrencyCode(String),
    #[error("The provided alpha3 code {0} doesn't match any in the iso4217 database")]
    InvalidISO4217Alpha3(String),
    #[error("The provided string {0} is invalid, it should be a decimal followed by a currency. e.g. 1.234 USD")]
    InvalidCommodityString(String),
}

/// Represents a the type of currency held in a
/// [Commodity](Commodity). See [CurrencyCode](CurrencyCode) for the
/// primative which is genarally stored and used to refer to a given
/// [Currency](Currency).
#[derive(Debug, Clone)]
pub struct Currency {
    /// Stores the code/id of this currency in a fixed length
    /// [ArrayString](ArrayString), with a maximum length of
    /// [CURRENCY_CODE_LENGTH](CURRENCY_CODE_LENGTH).
    pub code: CurrencyCode,
    /// The human readable name of this currency.
    pub name: Option<String>,
}

impl Currency {
    /// Create a new [Currency](Currency)
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Currency, CurrencyCode};
    ///
    /// let code = CurrencyCode::from_str("AUD").unwrap();
    /// let currency = Currency::new(
    ///     code,
    ///     Some(String::from("Australian Dollar"))
    /// );
    ///
    /// assert_eq!(code, currency.code);
    /// assert_eq!(Some(String::from("Australian Dollar")), currency.name);
    /// ```
    pub fn new(code: CurrencyCode, name: Option<String>) -> Currency {
        Currency { code, name }
    }

    /// Create a [Currency](Currency) from strings, usually for debugging,
    /// or unit testing purposes.
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Currency, CurrencyCode};
    /// let currency = Currency::from_str("AUD", "Australian dollar").unwrap();
    ///
    /// assert_eq!(CurrencyCode::from_str("AUD").unwrap(), currency.code);
    /// assert_eq!("Australian dollar", currency.name.unwrap());
    /// ```
    pub fn from_str(code: &str, name: &str) -> Result<Currency, CurrencyError> {
        let code = CurrencyCode::from_str(code)?;

        let name = if name.len() == 0 {
            None
        } else {
            Some(String::from(name))
        };

        Ok(Currency::new(code, name))
    }

    /// Construct a [Currency](Currency) by looking it up in the iso4217
    /// currency database.
    ///
    /// # Example
    /// ```
    /// # use coster::currency::Currency;
    ///
    /// let currency = Currency::from_alpha3("AUD").unwrap();
    /// assert_eq!("AUD", currency.code);
    /// assert_eq!(Some(String::from("Australian dollar")), currency.name);
    /// ```
    pub fn from_alpha3(alpha3: &str) -> Result<Currency, CurrencyError> {
        match iso4217::alpha3(alpha3) {
            Some(code) => Currency::from_str(alpha3, code.name),
            None => Err(CurrencyError::InvalidISO4217Alpha3(String::from(alpha3))),
        }
    }
}

/// Return a vector of all iso4217 currencies
pub fn all_iso4217_currencies() -> Vec<Currency> {
    let mut currencies = Vec::new();
    for iso_currency in iso4217::all() {
        currencies.push(Currency::from_str(iso_currency.alpha3, iso_currency.name).unwrap());
    }

    return currencies;
}

/// The code/id of a [Currency](Currency).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CurrencyCode {
    /// This is a fixed length array of characters of length [CURRENCY_CODE_LENGTH](CURRENCY_CODE_LENGTH),
    /// with a backing implementation based on [ArrayString](ArrayString).
    code_array: CurrencyCodeArray,
}

impl CurrencyCode {
    pub fn new(code_array: CurrencyCodeArray) -> CurrencyCode {
        CurrencyCode { code_array }
    }

    /// Create a new [Currency](Currency).
    ///
    /// # Example
    /// ```
    /// # use coster::currency::CurrencyCode;
    ///
    /// let currency_code = CurrencyCode::from_str("AUD").unwrap();
    /// assert_eq!("AUD", currency_code);
    /// ```
    pub fn from_str(code: &str) -> Result<CurrencyCode, CurrencyError> {
        if code.len() > CURRENCY_CODE_LENGTH {
            return Err(CurrencyError::TooLongCurrencyCode(String::from(code)));
        }

        return Ok(CurrencyCode::new(CurrencyCodeArray::from(code).unwrap()));
    }
}

// TODO: make serde a feature flag
impl<'de> Deserialize<'de> for CurrencyCode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<CurrencyCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct CurrencyCodeVisitor;

        impl<'de> Visitor<'de> for CurrencyCodeVisitor {
            type Value = CurrencyCode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    format!(
                        "a string with a maximum of {} characters",
                        CURRENCY_CODE_LENGTH
                    )
                    .as_ref(),
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                CurrencyCode::from_str(v).map_err(|e| {
                    E::custom(format!(
                        "there was an error ({}) parsing the currency code string",
                        e
                    ))
                })
            }
        }

        deserializer.deserialize_str(CurrencyCodeVisitor)
    }
}

impl PartialEq<CurrencyCode> for &str {
    fn eq(&self, other: &CurrencyCode) -> bool {
        match CurrencyCodeArray::from_str(self) {
            Ok(self_as_code) => self_as_code == other.code_array,
            Err(_) => false,
        }
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code_array)
    }
}

/// A commodity, which holds a value of a type of [Currrency](Currency)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Commodity {
    pub value: Decimal,
    pub currency_code: CurrencyCode,
}

/// Check whether the currencies of two commodities are compatible (the same),
/// if they aren't then return a [IncompatableCommodity](CurrencyError::IncompatableCommodity) error in the `Result`.
fn check_currency_compatible(
    this_commodity: &Commodity,
    other_commodity: &Commodity,
    reason: String,
) -> Result<(), CurrencyError> {
    if !this_commodity.compatible_with(other_commodity) {
        return Err(CurrencyError::IncompatableCommodity {
            this_commodity: this_commodity.clone(),
            other_commodity: other_commodity.clone(),
            reason,
        });
    }

    return Ok(());
}

impl Commodity {
    /// Create a new [Commodity](Commodity).
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity, CurrencyCode};
    /// # use std::str::FromStr;
    /// use rust_decimal::Decimal;
    /// use std::rc::Rc;
    ///
    /// let currency_code = CurrencyCode::from_str("USD").unwrap();
    /// let commodity = Commodity::new(Decimal::new(202, 2), currency_code);
    ///
    /// assert_eq!(Decimal::from_str("2.02").unwrap(), commodity.value);
    /// assert_eq!(currency_code, commodity.currency_code)
    /// ```
    pub fn new(value: Decimal, currency_code: CurrencyCode) -> Commodity {
        Commodity {
            currency_code,
            value,
        }
    }

    /// Create a commodity with a value of zero
    pub fn zero(currency_code: CurrencyCode) -> Commodity {
        Commodity::new(Decimal::zero(), currency_code)
    }

    /// Construct a [Commodity](Commodity) from a string
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity, CurrencyCode};
    /// use std::str::FromStr;
    /// use rust_decimal::Decimal;
    ///
    /// let commodity = Commodity::from_str("1.234 USD").unwrap();
    ///
    /// assert_eq!(Decimal::from_str("1.234").unwrap(), commodity.value);
    /// assert_eq!(CurrencyCode::from_str("USD").unwrap(), commodity.currency_code);
    /// ```
    pub fn from_str(commodity_string: &str) -> Result<Commodity, CurrencyError> {
        let elements: Vec<&str> = commodity_string.split_whitespace().collect();

        if elements.len() != 2 {
            return Err(CurrencyError::InvalidCommodityString(String::from(
                commodity_string,
            )));
        }

        Ok(Commodity::new(
            Decimal::from_str(elements.get(0).unwrap()).unwrap(),
            CurrencyCode::from_str(elements.get(1).unwrap())?,
        ))
    }

    /// Add the value of commodity `other` to `self`
    /// such that `result = self + other`.
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity, CurrencyCode};
    /// use rust_decimal::Decimal;
    /// use std::rc::Rc;
    ///
    /// let currency_code = CurrencyCode::from_str("USD").unwrap();
    /// let commodity1 = Commodity::new(Decimal::new(400, 2), currency_code);
    /// let commodity2 = Commodity::new(Decimal::new(250, 2), currency_code);
    ///
    /// // perform the add
    /// let result = commodity1.add(&commodity2).unwrap();
    ///
    /// assert_eq!(Decimal::new(650, 2), result.value);
    /// assert_eq!(currency_code, result.currency_code);
    /// ```
    pub fn add(&self, other: &Commodity) -> Result<Commodity, CurrencyError> {
        check_currency_compatible(
            self,
            other,
            String::from("cannot add commodities with different currencies"),
        )?;

        return Ok(Commodity::new(self.value + other.value, self.currency_code));
    }

    /// Subtract the value of commodity `other` from `self`
    /// such that `result = self - other`.
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity, CurrencyCode};
    /// use rust_decimal::Decimal;
    /// use std::rc::Rc;
    ///
    /// let currency_code = CurrencyCode::from_str("USD").unwrap();
    /// let commodity1 = Commodity::new(Decimal::new(400, 2), currency_code);
    /// let commodity2 = Commodity::new(Decimal::new(250, 2), currency_code);
    ///
    /// // perform the subtraction
    /// let result = commodity1.subtract(&commodity2).unwrap();
    ///
    /// assert_eq!(Decimal::new(150, 2), result.value);
    /// assert_eq!(currency_code, result.currency_code);
    /// ```
    pub fn subtract(&self, other: &Commodity) -> Result<Commodity, CurrencyError> {
        check_currency_compatible(
            self,
            other,
            String::from("cannot subtract commodities with different currencies"),
        )?;

        return Ok(Commodity::new(self.value - other.value, self.currency_code));
    }

    /// Negate the value of this commodity such that `result = -self`
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity, CurrencyCode};
    /// # use std::str::FromStr;
    /// use rust_decimal::Decimal;
    /// use std::rc::Rc;
    ///
    /// let currency_code = CurrencyCode::from_str("USD").unwrap();
    /// let commodity = Commodity::new(Decimal::new(202, 2), currency_code);
    ///
    /// // perform the negation
    /// let result = commodity.negate();
    ///
    /// assert_eq!(Decimal::from_str("-2.02").unwrap(), result.value);
    /// assert_eq!(currency_code, result.currency_code)
    /// ```
    pub fn negate(&self) -> Commodity {
        Commodity::new(-self.value, self.currency_code)
    }

    /// Divide this commodity by the specified integer value
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity};
    /// use rust_decimal::{Decimal};
    ///
    /// let commodity = Commodity::from_str("4.03 AUD").unwrap();
    /// let result = commodity.divide(4);
    /// assert_eq!(Decimal::new(10075, 4), result.value);
    /// ```
    pub fn divide(&self, i: i64) -> Commodity {
        let decimal = Decimal::new(i * 100, 2);
        Commodity::new(self.value / decimal, self.currency_code)
    }

    /// Divide this commodity by the specified integer value
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity};
    /// use rust_decimal::{Decimal};
    ///
    /// let commodity = Commodity::from_str("4.03 AUD").unwrap();
    /// let results = commodity.divide_share(4, 2);
    ///
    /// assert_eq!(Decimal::new(101, 2), results.get(0).unwrap().value);
    /// assert_eq!(Decimal::new(101, 2), results.get(1).unwrap().value);
    /// assert_eq!(Decimal::new(101, 2), results.get(2).unwrap().value);
    /// assert_eq!(Decimal::new(100, 2), results.get(3).unwrap().value);
    /// ```
    pub fn divide_share(&self, i: i64, dp: u32) -> Vec<Commodity> {
        let mut commodities: Vec<Commodity> = Vec::new();
        let divisor = Decimal::new(i * 10_i64.pow(dp), dp);
        let remainder = self.value % divisor;
        // = 0.03

        let mut divided = self.value / divisor;
        // 4.03 / 0.04 = 100.75
        // divided.set_scale(dp * 2).unwrap();
        // = 1.0075
        let truncated = divided.trunc();
        // = 1.00

        let dp_divisor = Decimal::new(1, dp);

        let remainder_bits = (remainder / dp_divisor).to_i64().unwrap();
        let remainder_bits_abs = remainder_bits.abs();
        let i_abs = i.abs();

        // dbg!(self.value);
        // dbg!(i);
        // dbg!(divided);
        // dbg!(truncated);
        // dbg!(remainder_bits);
        // dbg!(remainder);

        let sign = Decimal::new(remainder_bits.signum() * i.signum(), 0);

        for commodity_index in 1..=i_abs {
            let value = if commodity_index <= remainder_bits_abs {
                truncated + dp_divisor * sign
            } else {
                truncated
            };

            commodities.push(Commodity::new(value, self.currency_code))
        }

        dbg!(commodities.clone());

        return commodities;
    }

    /// Convert this commodity to a different currency using a conversion rate.
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity, CurrencyCode};
    /// use rust_decimal::Decimal;
    /// use std::str::FromStr;
    ///
    /// let aud = Commodity::from_str("100.00 AUD").unwrap();
    /// let usd = aud.convert(CurrencyCode::from_str("USD").unwrap(), Decimal::from_str("0.01").unwrap());
    ///
    /// assert_eq!(Decimal::from_str("1.00").unwrap(), usd.value);
    /// assert_eq!("USD", usd.currency_code);
    /// ```
    pub fn convert(&self, currency_code: CurrencyCode, rate: Decimal) -> Commodity {
        Commodity::new(self.value * rate, currency_code)
    }

    /// Returns true if the currencies of both this commodity, and
    /// the `other` commodity are compatible for numeric operations.
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity};
    /// let aud1 = Commodity::from_str("1.0 AUD").unwrap();
    /// let aud2 = Commodity::from_str("2.0 AUD").unwrap();
    /// let nzd = Commodity::from_str("1.0 NZD").unwrap();
    ///
    /// assert!(aud1.compatible_with(&aud2));
    /// assert!(!aud1.compatible_with(&nzd));
    /// ```
    pub fn compatible_with(&self, other: &Commodity) -> bool {
        return self.currency_code == other.currency_code;
    }
}

impl fmt::Display for Commodity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.currency_code)
    }
}

#[cfg(test)]
mod tests {
    use super::{Commodity, CurrencyCode, CurrencyError};
    use rust_decimal::Decimal;

    // #[test]
    // fn divide_larger() {
    //     let commodity = Commodity::from_str("4.25 AUD").unwrap();
    //     let results = commodity.divide_share(4, 2);

    //     assert_eq!(4, results.len());
    //     assert_eq!(Decimal::new(107, 2), results.get(0).unwrap().value);
    //     assert_eq!(Decimal::new(106, 2), results.get(1).unwrap().value);
    //     assert_eq!(Decimal::new(106, 2), results.get(2).unwrap().value);
    //     assert_eq!(Decimal::new(106, 2), results.get(3).unwrap().value);
    // }

    // #[test]
    // fn divide_share_negative_dividend() {
    //     let commodity = Commodity::from_str("-4.03 AUD").unwrap();
    //     let results = commodity.divide_share(4, 2);

    //     assert_eq!(4, results.len());
    //     assert_eq!(Decimal::new(-101, 2), results.get(0).unwrap().value);
    //     assert_eq!(Decimal::new(-101, 2), results.get(1).unwrap().value);
    //     assert_eq!(Decimal::new(-101, 2), results.get(2).unwrap().value);
    //     assert_eq!(Decimal::new(-100, 2), results.get(3).unwrap().value);
    // }

    // #[test]
    // fn divide_share_negative_divisor() {
    //     let commodity = Commodity::from_str("4.03 AUD").unwrap();
    //     let results = commodity.divide_share(-4, 2);

    //     assert_eq!(4, results.len());
    //     assert_eq!(Decimal::new(-101, 2), results.get(0).unwrap().value);
    //     assert_eq!(Decimal::new(-101, 2), results.get(1).unwrap().value);
    //     assert_eq!(Decimal::new(-101, 2), results.get(2).unwrap().value);
    //     assert_eq!(Decimal::new(-100, 2), results.get(3).unwrap().value);
    // }

    #[test]
    fn commodity_incompatible_currency() {
        let currency1 = CurrencyCode::from_str("USD").unwrap();
        let currency2 = CurrencyCode::from_str("AUD").unwrap();

        let commodity1 = Commodity::new(Decimal::new(400, 2), currency1);
        let commodity2 = Commodity::new(Decimal::new(250, 2), currency2);

        let error1 = commodity1.add(&commodity2).expect_err("expected an error");

        assert_eq!(
            CurrencyError::IncompatableCommodity {
                this_commodity: commodity1.clone(),
                other_commodity: commodity2.clone(),
                reason: String::from("cannot add commodities with different currencies"),
            },
            error1
        );

        let error2 = commodity1
            .subtract(&commodity2)
            .expect_err("expected an error");

        assert_eq!(
            CurrencyError::IncompatableCommodity {
                this_commodity: commodity1,
                other_commodity: commodity2,
                reason: String::from("cannot subtract commodities with different currencies"),
            },
            error2
        );
    }
}
