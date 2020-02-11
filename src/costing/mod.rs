//! This module holds the business logic for the `coster` application.
//! 
//! What do we want this to do? 
//! 
//! 1. Create a new Tab (a list of expenses and their associated users)

use chrono::{NaiveDate, DateTime, Utc};
use std::rc::Rc;
use crate::currency::{Commodity, Currency};
use crate::exchange_rate::ExchangeRate;

struct User {
    id: String,
    name: String,
    email: Option<String>,
}

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
    /// The date that this expense was incurred
    pub date: NaiveDate,
    /// The [User](User) who paid this expense
    pub paid_by: Rc<User>,
    /// [User](User)s who were involved in/benefited from/are sharing this expense
    pub shared_by: Vec<Rc<User>>,
    /// The amount of money 
    pub amount: Commodity,
    /// The exchange rate 
    pub exchange_rate: Option<ExchangeRate>,
}
