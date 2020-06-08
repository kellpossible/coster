use crate::expense::{ExpenseCategory, ExpenseID};
use crate::user::UserID;
use commodity::CommodityError;
use doublecount::AccountingError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum CostingError {
    #[error("error relating to accounting")]
    Accounting(#[from] AccountingError),
    #[error("error relating to currencies")]
    Currency(#[from] CommodityError),
    #[error("the specified User with id {0}, already exists on the Tab with id {1}")]
    UserAlreadyExistsOnTab(UserID, Uuid),
    #[error("the specified User with id {0}, does not exist on the Tab with id {1}")]
    UserDoesNotExistOnTab(UserID, Uuid),
    #[error("there is no Account associated with the User with id {0} on the Tab with id {1}")]
    UserAccountDoesNotExistOnTab(UserID, Uuid),
    #[error("the specified Expense with id {0}, already exists on the Tab with id {1}")]
    ExpenseAlreadyExistsOnTab(ExpenseID, Uuid),
    #[error("the specified Expense with id {0}, does not exist on the Tab with id {1}")]
    ExpenseDoesNotExistOnTab(ExpenseID, Uuid),
    #[error("the specified Expense category {0}, does not have an account on the tab with id {1}")]
    NoExpenseCategoryAccountOnTab(ExpenseCategory, Uuid),
}
