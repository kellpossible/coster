extern crate chrono;
extern crate iso4217;
extern crate rust_decimal;
extern crate arrayvec;

use rust_decimal::Decimal;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use arrayvec::ArrayString;

pub const CURRENCY_CODE_LENGTH: usize = 8;
type CurrencyCodeArray = ArrayString::<[u8;CURRENCY_CODE_LENGTH]>;

#[derive(Error, Debug, PartialEq)]
pub enum CurrencyError {
    // #[error("unable to convert commodity {commodity:?} to the currency {currency:?} because {reason:?}")]
    // UnableToConvert {
    //     commodity: Commodity,
    //     currency: Currency,
    //     reason: String,
    // },
    #[error("This commodity {this_commodity:?} is incompatible with {other_commodity:?} because {reason:?}")]
    IncompatableCommodity {
        this_commodity: Commodity,
        other_commodity: Commodity,
        reason: String,
    },
    #[error("The currency code {0} is too long. Maximum of {} characters allowed.", CURRENCY_CODE_LENGTH)]
    TooLongCurrencyCode(String)
}

/// Represents a the type of currency held in a [Commodity](Commodity).
#[derive(Debug, Clone)]
pub struct Currency {
    /// Stores the code/id of this currency in a fixed length [ArrayString](ArrayString),
    /// with a maximum length of [CURRENCY_CODE_LENGTH](CURRENCY_CODE_LENGTH).
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
        Currency {
            code,
            name,
        }
    }
}

/// The code/id of a [Currency](Currency).
/// 
/// This is a fixed length array of characters of length [CURRENCY_CODE_LENGTH](CURRENCY_CODE_LENGTH),
/// with a backing implementation based on [ArrayString](ArrayString).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurrencyCode {
    pub array: CurrencyCodeArray,
}

impl CurrencyCode {
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

        return Ok(CurrencyCode {
            array: CurrencyCodeArray::from(code).unwrap(),
        });
    }
}

impl PartialEq<CurrencyCode> for &str {
    fn eq(&self, other: &CurrencyCode) -> bool {
        match CurrencyCodeArray::from_str(self) {
            Ok(self_as_code) => {
                self_as_code == other.array
            },
            Err(_) => false,
        }
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.array)
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
    if this_commodity.currency_code != other_commodity.currency_code {
        return Err(CurrencyError::IncompatableCommodity {
            this_commodity: this_commodity.clone(),
            other_commodity: other_commodity.clone(),
            reason: reason,
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
            currency_code: currency_code,
            value: value,
        }
    }

    pub fn from_str(value: &str, currency_code: &str) -> Result<Commodity, CurrencyError> {
        Ok(Commodity::new(
            Decimal::from_str(value).unwrap(),
            CurrencyCode::from_str(currency_code)?,
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

        return Ok(Commodity::new(
            self.value + other.value,
            self.currency_code,
        ));
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

        return Ok(Commodity::new(
            self.value - other.value,
            self.currency_code,
        ));
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
        Commodity::new(
            -self.value,
            self.currency_code, 
        )
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
