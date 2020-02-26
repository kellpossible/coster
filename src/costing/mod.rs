//! This module holds the business logic for the `coster` application.
//!
//! What do we want this to do?
//!
//! 1. Create a new Tab (a list of expenses and their associated users)

use crate::accounting::{Account, Transaction, TransactionElement};
use crate::currency::{Commodity, Currency};
use crate::exchange_rate::ExchangeRate;
use chrono::{DateTime, NaiveDate, Utc, Local};
use std::collections::HashMap;
use std::rc::Rc;

struct User {
    id: String,
    name: String,
    email: Option<String>,
    account: Rc<Account>,
}

impl User {
    pub fn new(id: String, name: String, email: Option<String>, currency: Rc<Currency>) -> User {
        User {
            id: id.clone(),
            name,
            email,
            account: Rc::from(Account::new(Some(id), currency, None)),
        }
    }
}

struct CostingState {
    account_owners: HashMap<String, Rc<Account>>,
    users: Vec<Rc<User>>,
}

impl CostingState {}

struct UserAction<T> {
    data: Rc<T>,
    datetime: DateTime<Utc>,
    user: Rc<User>,
}

struct Ownership<T> {
    user: Rc<User>,
    data: T,
}

struct Tab {
    pub working_currency: Option<Rc<Currency>>,
    pub users: Vec<Rc<User>>,
    pub expenses: Vec<Expense>,
}

struct Expense {
    /// The description of this expense
    pub description: String,
    /// The account that this expense will be attributed to
    pub account: Rc<Account>,
    /// The date that this expense was incurred
    pub date: NaiveDate,
    /// The [User](User) who paid this expense
    pub paid_by: Rc<User>,
    /// [User](User)s who were involved in/benefited from/are sharing this expense
    pub shared_by: Vec<Rc<User>>,
    /// The amount of money
    pub amount: Commodity,
    /// The exchange rate to use for converting the expense to the working currency
    pub exchange_rate: Option<ExchangeRate>,
}

impl Expense {
    /// Get the transaction that occurred initially, where the user `paid_by`
    /// paid for the expense.
    fn get_actual_transaction(&self) -> Transaction {
        Transaction::new(
            Some(self.description.clone()),
            self.date,
            vec![
                TransactionElement::new(
                    self.paid_by.account.clone(),
                    Some(self.amount.negate()),
                    self.exchange_rate.clone(),
                ),
                TransactionElement::new(self.account.clone(), None, self.exchange_rate.clone()),
            ]
        )
    }

    /// Get a transaction where this expense is shared by all the users involved
    fn get_shared_transaction(&self) -> Transaction {
        let mut elements: Vec<TransactionElement> = Vec::new();

        Transaction::new(
            Some(self.description.clone()),
            Local::today().naive_local(),
            elements,
        )
        
    }
}

mod tests {
    #[test]
    fn balance() {}
}
