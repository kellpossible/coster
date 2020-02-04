extern crate chrono;
extern crate iso4217;
extern crate rust_decimal;

// use crate::exchange_rate::ExchangeRate;
use iso4217::CurrencyCode;
use rust_decimal::Decimal;
use std::fmt;
use std::rc::Rc;
use std::str::FromStr;
use thiserror::Error;

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
}

/// Represents a the type of currency held in a [Commodity](Commodity)
#[derive(Debug, Clone)]
pub struct Currency {
    pub symbol: Option<char>,
    pub code: &'static CurrencyCode,
}

impl Currency {
    /// Create a new [Currency](Currency).
    ///
    /// # Example
    /// ```
    /// # use coster::currency::Currency;
    ///
    /// let currency = Currency::new(Option::from('$'), &iso4217::alpha3("AUD").unwrap());
    /// assert_eq!("Australian dollar", currency.code.name);
    /// assert_eq!('$', currency.symbol.unwrap());
    /// ```
    pub fn new(symbol: Option<char>, code: &'static CurrencyCode) -> Currency {
        Currency {
            symbol: symbol,
            code: code,
        }
    }

    pub fn from_alpha3(symbol: Option<char>, alpha3: &str) -> Currency {
        Currency::new(symbol, iso4217::alpha3(alpha3).unwrap())
    }
}

impl PartialEq for Currency {
    /// Implementation of [PartialEq](core::cmp::PartialEq) for [Currency](Currency) which compares
    /// their currency code `alpha3` values.
    fn eq(&self, other: &Self) -> bool {
        self.code.alpha3 == other.code.alpha3
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code.alpha3)
    }
}

/// A commodity, which holds a value of a type of [Currrency](Currency)
#[derive(Debug, Clone, PartialEq)]
pub struct Commodity {
    pub currency: Rc<Currency>,
    pub value: Decimal,
}

/// Check whether the currencies of two commodities are compatible (the same),
/// if they aren't then return a [IncompatableCommodity](CurrencyError::IncompatableCommodity) error in the `Result`.
fn check_currency_compatible(
    this_commodity: &Commodity,
    other_commodity: &Commodity,
    reason: String,
) -> Result<(), CurrencyError> {
    if this_commodity.currency != other_commodity.currency {
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
    /// # use coster::currency::{Commodity, Currency};
    /// # use std::str::FromStr;
    /// use rust_decimal::Decimal;
    /// use std::rc::Rc;
    ///
    /// let currency = Rc::from(Currency::from_alpha3(Option::from('$'), "USD"));
    /// let commodity = Commodity::new(currency.clone(), Decimal::new(202, 2));
    ///
    /// assert_eq!(Decimal::from_str("2.02").unwrap(), commodity.value);
    /// assert_eq!(currency, commodity.currency)
    /// ```
    pub fn new(currency: Rc<Currency>, value: Decimal) -> Commodity {
        Commodity {
            currency: currency,
            value: value,
        }
    }

    pub fn from_str(symbol: Option<char>, currency_alpha3: &str, value: &str) -> Commodity {
        Commodity::new(
            Rc::from(Currency::from_alpha3(symbol, currency_alpha3)),
            Decimal::from_str(value).unwrap(),
        )
    }

    /// Add the value of commodity `other` to `self`
    /// such that `result = self + other`.
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity, Currency};
    /// use rust_decimal::Decimal;
    /// use std::rc::Rc;
    ///
    /// let currency = Rc::from(Currency::from_alpha3(Option::from('$'), "USD"));
    /// let commodity1 = Commodity::new(currency.clone(), Decimal::new(400, 2));
    /// let commodity2 = Commodity::new(currency.clone(), Decimal::new(250, 2));
    ///
    /// // perform the add
    /// let result = commodity1.add(&commodity2).unwrap();
    ///
    /// assert_eq!(Decimal::new(650, 2), result.value);
    /// assert_eq!(currency, result.currency);
    /// ```
    pub fn add(&self, other: &Commodity) -> Result<Commodity, CurrencyError> {
        check_currency_compatible(
            self,
            other,
            String::from("cannot add commodities with different currencies"),
        )?;

        return Ok(Commodity::new(
            self.currency.clone(),
            self.value + other.value,
        ));
    }

    /// Subtract the value of commodity `other` from `self`
    /// such that `result = self - other`.
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity, Currency};
    /// use rust_decimal::Decimal;
    /// use std::rc::Rc;
    ///
    /// let currency = Rc::from(Currency::from_alpha3(Option::from('$'), "USD"));
    /// let commodity1 = Commodity::new(currency.clone(), Decimal::new(400, 2));
    /// let commodity2 = Commodity::new(currency.clone(), Decimal::new(250, 2));
    ///
    /// // perform the subtraction
    /// let result = commodity1.subtract(&commodity2).unwrap();
    ///
    /// assert_eq!(Decimal::new(150, 2), result.value);
    /// assert_eq!(currency, result.currency);
    /// ```
    pub fn subtract(&self, other: &Commodity) -> Result<Commodity, CurrencyError> {
        check_currency_compatible(
            self,
            other,
            String::from("cannot subtract commodities with different currencies"),
        )?;

        return Ok(Commodity::new(
            self.currency.clone(),
            self.value - other.value,
        ));
    }

    /// Negate the value of this commodity such that `result = -self`
    ///
    /// # Example
    /// ```
    /// # use coster::currency::{Commodity, Currency};
    /// # use std::str::FromStr;
    /// use rust_decimal::Decimal;
    /// use std::rc::Rc;
    ///
    /// let currency = Rc::from(Currency::from_alpha3(Option::from('$'), "USD"));
    /// let commodity = Commodity::new(currency.clone(), Decimal::new(202, 2));
    ///
    /// // perform the negation
    /// let result = commodity.negate();
    ///
    /// assert_eq!(Decimal::from_str("-2.02").unwrap(), result.value);
    /// assert_eq!(currency, result.currency)
    /// ```
    pub fn negate(&self) -> Commodity {
        Commodity::new(self.currency.clone(), -self.value)
    }
}

impl fmt::Display for Commodity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}

#[cfg(test)]
mod tests {
    use super::{Commodity, Currency, CurrencyError};
    use rust_decimal::Decimal;
    use std::rc::Rc;

    #[test]
    fn commodity_incompatible_currency() {
        let currency1 = Rc::from(Currency::from_alpha3(Option::from('$'), "USD"));
        let currency2 = Rc::from(Currency::from_alpha3(Option::from('$'), "AUD"));

        let commodity1 = Commodity::new(currency1, Decimal::new(400, 2));
        let commodity2 = Commodity::new(currency2, Decimal::new(250, 2));

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
