use crate::user::User;
use doublecount::{Account, Transaction, TransactionElement};
use commodity::{Commodity, exchange_rate::ExchangeRate};
use chrono::{NaiveDate, Local};
use std::rc::Rc;
use std::convert::TryInto;


/// An expense which is paid by a user on a given `date`, and which is
/// to be shared by a list of users.
#[derive(Debug)]
pub struct Expense {
    /// The id of this expense
    pub id: i32,
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
    /// let aud = Rc::from(CommodityType::from_currency_alpha3("AUD").unwrap());
    /// let user1 = Rc::from(User::new(1, "User 1", None, aud.clone()));
    /// let user2 = Rc::from(User::new(2, "User 2", None, aud.clone()));
    ///
    /// let expenses_account = Rc::from(Account::new(Some("Expenses"), aud.id, None));
    ///
    /// let expense = Expense::new(
    ///    1,
    ///    "some expense", expenses_account.clone(),
    ///    NaiveDate::from_ymd(2020, 2, 27),
    ///    user1.clone(),
    ///    vec![user1.clone(), user2.clone()],
    ///    Commodity::from_str("300.0 AUD").unwrap(),
    ///    None
    /// );
    ///
    /// assert_eq!(expense.account, expenses_account);
    /// assert_eq!(NaiveDate::from_ymd(2020, 2, 27), expense.date);
    /// assert_eq!(user1.clone(), expense.paid_by);
    /// assert_eq!(vec![user1.clone(), user2.clone()], expense.shared_by);
    /// assert_eq!(Commodity::from_str("300.0 AUD").unwrap(), expense.amount);
    /// ```
    pub fn new(
        id: i32,
        description: &str,
        account: Rc<Account>,
        date: NaiveDate,
        paid_by: Rc<User>,
        shared_by: Vec<Rc<User>>,
        amount: Commodity,
        exchange_rate: Option<ExchangeRate>,
    ) -> Expense {
        Expense {
            id,
            description: String::from(description),
            account,
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
    /// # use costing::{Expense, User};
    /// use doublecount::{Transaction, Account};
    /// use commodity::{Commodity, CommodityType};
    /// use std::rc::Rc;
    /// use chrono::NaiveDate;
    /// use std::str::FromStr;
    ///
    /// let aud = Rc::from(CommodityType::from_currency_alpha3("AUD").unwrap());
    /// let user1 = Rc::from(User::new(1, "User 1", None, aud.clone()));
    /// let user2 = Rc::from(User::new(2, "User 2", None, aud.clone()));
    /// let user3 = Rc::from(User::new(3, "User 3", None, aud.clone()));
    ///
    /// let expenses_account = Rc::from(Account::new(Some("Expenses"), aud.id, None));
    ///
    /// let expense = Expense::new(
    ///    1,
    ///    "some expense", expenses_account.clone(),
    ///    NaiveDate::from_ymd(2020, 2, 27),
    ///    user1.clone(),
    ///    vec!(user1.clone(), user2.clone(), user3.clone()),
    ///    Commodity::from_str("300.0 AUD").unwrap(),
    ///    None
    /// );
    ///
    /// let actual_transaction = expense.get_actual_transaction();
    ///
    /// assert_eq!(2, actual_transaction.elements.len());
    /// let user1_element = actual_transaction.get_element(&user1.account.id).unwrap();
    /// assert_eq!(Some(Commodity::from_str("-300.0 AUD").unwrap()), user1_element.amount);
    /// let expense_element = actual_transaction.get_element(&expenses_account.id).unwrap();
    /// assert_eq!(None, expense_element.amount);
    /// ```
    pub fn get_actual_transaction(&self) -> Transaction {
        Transaction::new(
            Some(self.description.clone()),
            self.date,
            vec![
                TransactionElement::new(
                    self.paid_by.account.id,
                    Some(self.amount.neg()),
                    self.exchange_rate.clone(),
                ),
                TransactionElement::new(self.account.id, None, self.exchange_rate.clone()),
            ],
        )
    }

    /// Get a transaction where this expense is shared by all the users involved
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
    /// let aud = Rc::from(CommodityType::from_currency_alpha3("AUD").unwrap());
    /// let user1 = Rc::from(User::new(1, "User 1", None, aud.clone()));
    /// let user2 = Rc::from(User::new(2, "User 2", None, aud.clone()));
    /// let user3 = Rc::from(User::new(3, "User 3", None, aud.clone()));
    ///
    /// let expenses_account = Rc::from(Account::new(Some("Expenses"), aud.id, None));
    ///
    /// let expense = Expense::new(
    ///    1,
    ///    "some expense", expenses_account.clone(),
    ///    NaiveDate::from_ymd(2020, 2, 27),
    ///    user1.clone(),
    ///    vec!(user2.clone(), user3.clone()),
    ///    Commodity::from_str("300.0 AUD").unwrap(),
    ///    None
    /// );
    ///
    /// let shared_transaction = expense.get_shared_transaction();
    ///
    /// assert_eq!(3, shared_transaction.elements.len());
    /// assert!(shared_transaction.get_element(&user1.account.id).is_none());
    ///
    /// let user2_element = shared_transaction.get_element(&user2.account.id).unwrap();
    /// let user3_element = shared_transaction.get_element(&user3.account.id).unwrap();
    /// assert_eq!(Some(Commodity::from_str("-150.0 AUD").unwrap()), user2_element.amount);
    /// assert_eq!(Some(Commodity::from_str("-150.0 AUD").unwrap()), user3_element.amount);
    ///
    /// let expense_element = shared_transaction.get_element(&expenses_account.id).unwrap();
    /// assert_eq!(None, expense_element.amount);
    /// ```
    pub fn get_shared_transaction(&self) -> Transaction {
        let mut elements: Vec<TransactionElement> = Vec::with_capacity(self.shared_by.len());

        // TODO: perhaps consider using divide_share instead
        let divided = self
            .amount
            .div_i64(self.shared_by.len().try_into().unwrap())
            .neg();

        for user in &self.shared_by {
            let element = TransactionElement::new(
                user.account.id,
                Some(divided),
                self.exchange_rate.clone(),
            );
            elements.push(element);
        }

        elements.push(TransactionElement::new(
            self.account.id,
            None,
            self.exchange_rate.clone(),
        ));

        Transaction::new(
            Some(self.description.clone()),
            Local::today().naive_local(),
            elements,
        )
    }
}