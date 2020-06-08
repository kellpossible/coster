use crate::error::CostingError;
use crate::tab::Tab;
use crate::user::UserID;
use chrono::{Local, NaiveDate};
use commodity::{exchange_rate::ExchangeRate, Commodity};
use doublecount::{Transaction, TransactionElement};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

pub type ExpenseID = i32;
pub type ExpenseCategory = String;

/// An expense which is paid by a user on a given `date`, and which is
/// to be shared by a list of users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    /// The id of this expense
    pub id: ExpenseID,
    /// The description of this expense
    pub description: String,
    /// The category that the expense will be attributed to
    pub category: ExpenseCategory,
    /// The date that this expense was incurred
    pub date: NaiveDate,
    /// The [User](User) who paid this expense
    pub paid_by: UserID,
    /// [User](User)s who were involved in/benefited from/are sharing this expense
    pub shared_by: Vec<UserID>,
    /// The amount of money
    pub amount: Commodity,
    /// The exchange rate to use for converting the expense to the working currency
    pub exchange_rate: Option<ExchangeRate>,
}

impl Expense {
    /// Create a new expense
    ///
    /// # Example
    /// ```
    /// # use costing::{Expense, User};
    /// use doublecount::{Transaction, Account};
    /// use commodity::{Commodity, CommodityType};
    /// use std::rc::Rc;
    /// use chrono::NaiveDate;
    /// use std::str::FromStr;
    ///
    /// let user1 = Rc::from(User::new(1, "User 1", None));
    /// let user2 = Rc::from(User::new(2, "User 2", None));
    ///
    /// let expense = Expense::new(
    ///    1,
    ///    "some expense",
    ///    "Test",
    ///    NaiveDate::from_ymd(2020, 2, 27),
    ///    user1.id,
    ///    vec![user1.id, user2.id],
    ///    Commodity::from_str("300.0 AUD").unwrap(),
    ///    None
    /// );
    ///
    /// assert_eq!(NaiveDate::from_ymd(2020, 2, 27), expense.date);
    /// assert_eq!(user1.id, expense.paid_by);
    /// assert_eq!(vec![user1.id, user2.id], expense.shared_by);
    /// assert_eq!(Commodity::from_str("300.0 AUD").unwrap(), expense.amount);
    /// ```
    pub fn new<S: Into<String>, EC: Into<ExpenseCategory>>(
        id: ExpenseID,
        description: S,
        category: EC,
        date: NaiveDate,
        paid_by: UserID,
        shared_by: Vec<UserID>,
        amount: Commodity,
        exchange_rate: Option<ExchangeRate>,
    ) -> Expense {
        Expense {
            id,
            description: description.into(),
            category: category.into(),
            date,
            paid_by,
            shared_by,
            amount,
            exchange_rate,
        }
    }

    /// Get the transaction that occurred initially, where the user `paid_by`
    /// paid for the expense.
    ///
    /// # Example
    /// ```
    /// use costing::{Expense, ExpenseCategory, User, Tab};
    /// use doublecount::{Transaction, Account};
    /// use commodity::{Commodity, CommodityType};
    /// use std::rc::Rc;
    /// use chrono::NaiveDate;
    /// use std::str::FromStr;
    /// use uuid::Uuid;
    ///
    /// let aud = Rc::from(CommodityType::from_currency_alpha3("AUD").unwrap());
    /// let user1 = Rc::from(User::new(1, "User 1", None));
    /// let user2 = Rc::from(User::new(2, "User 2", None));
    /// let user3 = Rc::from(User::new(3, "User 3", None));
    ///
    /// let category: ExpenseCategory = "Test".to_string();
    ///
    /// let expense = Expense::new(
    ///    1,
    ///    "some expense",
    ///    category.clone(),
    ///    NaiveDate::from_ymd(2020, 2, 27),
    ///    user1.id,
    ///    vec![user1.id, user2.id, user3.id],
    ///    Commodity::from_str("300.0 AUD").unwrap(),
    ///    None
    /// );
    ///
    /// let tab = Tab::new(
    ///     Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
    ///     "Test Tab",
    ///     aud.clone(),
    ///     vec![user1.clone(), user2.clone(), user3.clone()],
    ///     vec![expense],
    /// );
    ///
    /// let actual_transaction = tab.expenses.get(0)
    ///                                      .unwrap()
    ///                                      .get_actual_transaction(&tab)
    ///                                      .unwrap();
    ///
    /// let user1_account = tab.get_user_account(&user1.id).unwrap();
    ///
    /// assert_eq!(2, actual_transaction.elements.len());
    /// let user1_element = actual_transaction.get_element(&user1_account.id).unwrap();
    /// assert_eq!(Some(Commodity::from_str("-300.0 AUD").unwrap()), user1_element.amount);
    ///
    /// let expenses_account = tab.get_expense_category_account(&category).unwrap();
    /// let expense_element = actual_transaction.get_element(&expenses_account.id).unwrap();
    /// assert_eq!(None, expense_element.amount);
    /// ```
    pub fn get_actual_transaction(&self, tab: &Tab) -> Result<Transaction, CostingError> {
        Ok(Transaction::new(
            Some(self.description.clone()),
            self.date,
            vec![
                TransactionElement::new(
                    tab.get_user_account(&self.paid_by)?.id,
                    Some(self.amount.neg()),
                    self.exchange_rate.clone(),
                ),
                TransactionElement::new(
                    tab.get_expense_category_account(&self.category)?.id,
                    None,
                    self.exchange_rate.clone(),
                ),
            ],
        ))
    }

    /// Get a transaction where this expense is shared by all the users involved
    ///
    /// # Example
    /// ```
    /// use costing::{Expense, ExpenseCategory, User, Tab};
    /// use doublecount::{Transaction, Account};
    /// use commodity::{Commodity, CommodityType};
    /// use std::rc::Rc;
    /// use chrono::NaiveDate;
    /// use std::str::FromStr;
    /// use uuid::Uuid;
    ///
    /// let aud = Rc::from(CommodityType::from_currency_alpha3("AUD").unwrap());
    /// let user1 = Rc::from(User::new(1, "User 1", None));
    /// let user2 = Rc::from(User::new(2, "User 2", None));
    /// let user3 = Rc::from(User::new(3, "User 3", None));
    ///
    /// let category: ExpenseCategory = "Test".to_string();
    ///
    /// let expense = Expense::new(
    ///    1,
    ///    "some expense",
    ///    category.clone(),
    ///    NaiveDate::from_ymd(2020, 2, 27),
    ///    user1.id,
    ///    vec![user2.id, user3.id],
    ///    Commodity::from_str("300.0 AUD").unwrap(),
    ///    None
    /// );
    ///
    /// let tab = Tab::new(
    ///     Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
    ///     "Test Tab",
    ///     aud.clone(),
    ///     vec![user1.clone(), user2.clone(), user3.clone()],
    ///     vec![expense],
    /// );
    ///
    /// let user1_account = tab.get_user_account(&user1.id).unwrap();
    /// let user2_account = tab.get_user_account(&user2.id).unwrap();
    /// let user3_account = tab.get_user_account(&user3.id).unwrap();
    ///
    /// let shared_transaction = tab.expenses.get(0)
    ///                                      .unwrap()
    ///                                      .get_shared_transaction(&tab)
    ///                                      .unwrap();
    ///
    /// assert_eq!(3, shared_transaction.elements.len());
    /// assert!(shared_transaction.get_element(&user1_account.id).is_none());
    /// let user2_element = shared_transaction.get_element(&user2_account.id).unwrap();
    /// let user3_element = shared_transaction.get_element(&user3_account.id).unwrap();
    /// assert_eq!(Some(Commodity::from_str("-150.0 AUD").unwrap()), user2_element.amount);
    /// assert_eq!(Some(Commodity::from_str("-150.0 AUD").unwrap()), user3_element.amount);
    ///
    /// let expenses_account = tab.get_expense_category_account(&category).unwrap();
    /// let expense_element = shared_transaction.get_element(&expenses_account.id).unwrap();
    /// assert_eq!(None, expense_element.amount);
    /// ```
    pub fn get_shared_transaction(&self, tab: &Tab) -> Result<Transaction, CostingError> {
        let mut elements: Vec<TransactionElement> = Vec::with_capacity(self.shared_by.len());

        // TODO: perhaps consider using divide_share instead
        let divided = self
            .amount
            .div_i64(self.shared_by.len().try_into().unwrap())
            .neg();

        for user_id in &self.shared_by {
            let element = TransactionElement::new(
                tab.get_user_account(user_id)?.id,
                Some(divided),
                self.exchange_rate.clone(),
            );
            elements.push(element);
        }

        elements.push(TransactionElement::new(
            tab.get_expense_category_account(&self.category)?.id,
            None,
            self.exchange_rate.clone(),
        ));

        Ok(Transaction::new(
            Some(self.description.clone()),
            Local::today().naive_local(),
            elements,
        ))
    }
}
