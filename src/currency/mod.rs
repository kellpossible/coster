extern crate rust_decimal;
extern crate iso4217;

use rust_decimal::Decimal;
use iso4217::CurrencyCode;
use std::rc::Rc;

pub struct CurrencyType {
    pub symbol: Option<char>,
    pub code: &'static CurrencyCode,
}

impl CurrencyType {
    fn new(symbol: Option<char>, code: &'static CurrencyCode) -> CurrencyType {
        CurrencyType {
            symbol: symbol,
            code: code,
        }
    }

    fn from_alpha3(symbol: Option<char>, alpha3: &str) -> CurrencyType {
        CurrencyType::new(symbol, iso4217::alpha3(alpha3).unwrap())
    }
}

pub struct Currency {
    pub currency_type: Rc<CurrencyType>,
    pub value: Decimal,
}

impl Currency {
    fn new(currency_type: Rc<CurrencyType>, value: Decimal) -> Currency {
        Currency {
            currency_type: currency_type,
            value: value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CurrencyType, Currency};
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use std::rc::Rc;

    #[test]
    fn new_currency_type() {
        let currency_type = CurrencyType::new(Option::from('$'), &iso4217::alpha3("AUD").unwrap());
        assert_eq!("Australian dollar", currency_type.code.name);
        assert_eq!('$', currency_type.symbol.unwrap());
    }

    #[test]
    fn new_currency() {
        let currency_type = CurrencyType::from_alpha3(Option::from('$'), "USD");
        let currency = Currency::new(Rc::from(currency_type), Decimal::new(202, 2));
        assert_eq!(Decimal::from_str("2.02").unwrap(), currency.value);
        assert_eq!("USD", currency.currency_type.code.alpha3)
    }
}