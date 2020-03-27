use thiserror::Error;
use commodity::CommodityError;
use doublecount::AccountingError;

#[derive(Error, Debug)]
pub enum CostingError {
    #[error("error relating to accounting")]
    Accounting(#[from] AccountingError),
    #[error("error relating to currencies")]
    Currency(#[from] CommodityError),
}