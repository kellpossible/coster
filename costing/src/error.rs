use crate::tab::TabID;
use crate::user::UserID;
use crate::expense::ExpenseID;
use commodity::CommodityError;
use doublecount::AccountingError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CostingError {
    #[error("error relating to accounting")]
    Accounting(#[from] AccountingError),
    #[error("error relating to currencies")]
    Currency(#[from] CommodityError),
    #[error("the specified User with id {0}, already exists on the Tab with id {1}")]
    UserAlreadyExistsOnTab(UserID, TabID),
    #[error("the specified User with id {0}, does not exist on the Tab with id {1}")]
    UserDoesNotExistOnTab(UserID, TabID),
    #[error("the specified Expense with id {0}, already exists on the Tab with id {1}")]
    ExpenseAlreadyExistsOnTab(ExpenseID, TabID),
    #[error("the specified Expense with id {0}, does not exist on the Tab with id {1}")]
    ExpenseDoesNotExistOntab(ExpenseID, TabID),
}
