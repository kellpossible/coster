//! This module holds the business logic for the `coster` application.

use crate::accounting::{
    sum_account_states, Account, AccountID, AccountState, AccountStatus, AccountingError, Action,
    Program, ProgramState, Transaction, TransactionElement,
};
use crate::currency::{Commodity, Currency, CurrencyError};
use crate::exchange_rate::ExchangeRate;
use chrono::{DateTime, Local, NaiveDate, Utc};
use std::collections::HashMap;
use std::{cmp::Reverse, convert::TryInto, rc::Rc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CostingError {
    #[error("error relating to accounting")]
    Accounting(#[from] AccountingError),
    #[error("error relating to currencies")]
    Currency(#[from] CurrencyError),
}

#[derive(Debug)]
/// Represents a person using this system, and to be associated with
/// [Expense](Expenses) in a [Tab](Tab).
pub struct User {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub account: Rc<Account>,
}

impl User {
    pub fn new(id: &str, name: &str, email: Option<&str>, currency: Rc<Currency>) -> User {
        User {
            id: String::from(id),
            name: String::from(name),
            email: email.map(|e| String::from(e)),
            account: Rc::from(Account::new(Some(id), currency, None)),
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &User) -> bool {
        self.id == other.id
    }
}

/// Calculate the differences in amounts between two sets of account
/// states, per account.
fn account_state_difference(
    account_states_from: &HashMap<AccountID, AccountState>,
    account_states_to: &HashMap<AccountID, AccountState>,
) -> Result<HashMap<AccountID, AccountState>, CostingError> {
    assert!(account_states_from.len() == account_states_to.len());

    let mut result: HashMap<AccountID, AccountState> = HashMap::new();

    for (from_id, from_state) in account_states_from {
        let to_state = match account_states_to.get(from_id) {
            Some(state) => state,
            None => {
                return Err(CostingError::Accounting(
                    AccountingError::MissingAccountState(from_id.clone()),
                ))
            }
        };

        let difference_amount = to_state
            .amount
            .sub(&from_state.amount)
            .map_err(|e| AccountingError::Currency(e))?;

        let difference_state = AccountState::new(
            to_state.account.clone(),
            difference_amount,
            AccountStatus::Open,
        );

        result.insert(from_id.clone(), difference_state);
    }

    Ok(result)
}

/// A collection of expenses, and users who are responsible
/// for/associated with those expenses.
pub struct Tab {
    pub working_currency: Rc<Currency>,
    pub users: Vec<Rc<User>>,
    pub expenses: Vec<Expense>,
}

impl Tab {
    /// Construct a new [Tab](Tab).
    pub fn new(
        working_currency: Rc<Currency>,
        users: Vec<Rc<User>>,
        expenses: Vec<Expense>,
    ) -> Tab {
        Tab {
            working_currency,
            users,
            expenses,
        }
    }
    /// Produce a set of transactions, that, when applied to the
    /// result of the actual transactions generated by this Tab's
    /// expenses, will ensure that each user has fairly shared each
    /// expense that they have participated in.
    ///
    /// The aim here, is to produce a minimal set of transactions,
    /// which favour users who have smaller debts making less
    /// transactions, and those with larget debts making more
    /// transactions.
    pub fn balance_transactions(&self) -> Result<Vec<Settlement>, CostingError> {
        let mut actual_transactions: Vec<Rc<dyn Action>> = Vec::new();
        let mut shared_transactions: Vec<Rc<dyn Action>> = Vec::new();

        let mut accounts: HashMap<AccountID, Rc<Account>> = HashMap::new();

        for expense in &self.expenses {
            actual_transactions.push(Rc::from(expense.get_actual_transaction()) as Rc<dyn Action>);
            shared_transactions.push(Rc::from(expense.get_shared_transaction()) as Rc<dyn Action>);

            accounts.insert(expense.account.id.clone(), expense.account.clone());
        }

        let expense_accounts: Vec<Rc<Account>> = accounts.iter().map(|(_, v)| v.clone()).collect();

        let actual_program = Program::new(actual_transactions.clone());

        for user in &self.users {
            match accounts.insert(user.account.id.clone(), user.account.clone()) {
                Some(account) => {
                    panic!(format!(
                        "there is a duplicate account with id: {}",
                        account.id
                    ));
                }
                None => {}
            }
        }

        let accounts_vec: Vec<Rc<Account>> = accounts.into_iter().map(|(_, v)| v).collect();
        let mut actual_program_state = ProgramState::new(&accounts_vec, AccountStatus::Open);

        actual_program_state.execute_program(&actual_program)?;

        // the shared_program_state (after execution) is the desired
        // end-state where all users have fairly shared the expenses
        // that they have participated in.
        let shared_program = Program::new(shared_transactions);
        let mut shared_program_state = ProgramState::new(&accounts_vec, AccountStatus::Open);
        shared_program_state.execute_program(&shared_program)?;

        let account_states_from = &mut actual_program_state.account_states;
        let account_states_to = &mut shared_program_state.account_states;

        let from_sum_with_expenses =
            sum_account_states(account_states_from, self.working_currency.code, None)?;
        assert_eq!(
            Commodity::zero(self.working_currency.code),
            from_sum_with_expenses
        );
        let to_sum_with_expenses =
            sum_account_states(account_states_to, self.working_currency.code, None)?;
        assert_eq!(
            Commodity::zero(self.working_currency.code),
            to_sum_with_expenses
        );

        let mut account_states_from_without_expenses = account_states_from.clone();
        let mut account_states_to_without_expenses = account_states_to.clone();

        // remove the expense accounts from the states
        for account in &expense_accounts {
            account_states_from_without_expenses.remove(&account.id);
            account_states_to_without_expenses.remove(&account.id);
        }

        let account_differences = account_state_difference(
            &account_states_from_without_expenses,
            &account_states_to_without_expenses,
        )?;

        let differences_sum =
            sum_account_states(&account_differences, self.working_currency.code, None)?;

        assert_eq!(Commodity::zero(self.working_currency.code), differences_sum);

        let mut negative_differences: Vec<AccountState> = Vec::new();
        let mut positive_differences: Vec<AccountState> = Vec::new();

        let zero = Commodity::zero(self.working_currency.code);

        // create two lists of account state differences associated with those users
        // one list of negative, and one list of positive
        for (_, state) in &account_differences {
            if state.amount.lt(&zero)? {
                negative_differences.push(state.clone());
            } else if state.amount.gt(&zero)? {
                positive_differences.push(state.clone());
            }
        }

        // sort lists smallest (abs) to largest.
        negative_differences.sort_unstable_by_key(|state| Reverse(state.amount));
        positive_differences.sort_unstable_by_key(|state| state.amount);

        dbg!(&negative_differences);
        dbg!(&positive_differences);

        let mut balancing_transactions: Vec<Transaction> = Vec::new();
        let mut to_remove_positive: Vec<usize> = Vec::new();

        for negative_difference_state in &mut negative_differences {
            if negative_difference_state.amount == zero {
                continue;
            }

            // turns the negative difference (the debt), into a
            // positive number to use for comparison with the positive
            // differences (the accounts which are owed)
            let negated_negative_state_amount = negative_difference_state.amount.neg();

            let today = Local::today().naive_local();

            // find continue on to find the first state which is
            // bigger or equal to the selected state if found, create
            // a transaction to cancel out the selected state's debt,
            // altering the two states involved.
            for i in 0..positive_differences.len() {
                let positive_difference_state = positive_differences.get_mut(i).unwrap();

                if positive_difference_state.amount >= negated_negative_state_amount {
                    let mut transactions = balance_entire_negative_into_positive(
                        today,
                        negative_difference_state,
                        positive_difference_state,
                        &zero,
                    )?;
                    balancing_transactions.append(&mut transactions);

                    if positive_difference_state.amount == zero {
                        to_remove_positive.push(i);
                    }

                    break;
                }
            }

            // if no bigger/equal state has been found, then restart
            // the search at the start of the list, (ignoring self),
            // and create transactions cancelling out the selected
            // state's debt, until finished.
            if negative_difference_state.amount != zero {
                for i in 0..positive_differences.len() {
                    let positive_difference_state = positive_differences.get_mut(i).unwrap();

                    if positive_difference_state.amount <= negated_negative_state_amount {
                        balancing_transactions.push(Transaction::new_simple(
                            Some("balancing"),
                            today,
                            negative_difference_state.account.clone(),
                            positive_difference_state.account.clone(),
                            positive_difference_state.amount,
                            None,
                        ));

                        negative_difference_state.amount = negative_difference_state
                            .amount
                            .add(&positive_difference_state.amount)?;
                        positive_difference_state.amount = zero;
                    } else {
                        let mut transactions = balance_entire_negative_into_positive(
                            today,
                            negative_difference_state,
                            positive_difference_state,
                            &zero,
                        )?;
                        balancing_transactions.append(&mut transactions);
                    }

                    if positive_difference_state.amount == zero {
                        to_remove_positive.push(i);
                    }

                    if negative_difference_state.amount == zero {
                        break;
                    }
                }
            }

            // remove positive differences with a now zero amount
            for i in &to_remove_positive {
                positive_differences.remove(*i);
            }

            to_remove_positive.clear();
        }

        // apply the transactions to the actual account states, and
        // check that it matches the desired account states

        // dbg!(&balancing_transactions);

        let mut actual_with_balancing_transactions = actual_transactions.clone();
        balancing_transactions.iter().for_each(|bt| actual_with_balancing_transactions.push(Rc::from(bt.clone())));

        let actual_balanced_program = Program::new(actual_with_balancing_transactions);
        let mut actual_balanced_transactions_states =
            ProgramState::new(&accounts_vec, AccountStatus::Open);
        actual_balanced_transactions_states.execute_program(&actual_balanced_program)?;

        dbg!(&actual_balanced_transactions_states.account_states);

        let actual_balanced_sum = sum_account_states(&actual_balanced_transactions_states.account_states, self.working_currency.code, None)?;
        assert_eq!(zero, actual_balanced_sum);
        assert_eq!(account_states_to, &actual_balanced_transactions_states.account_states);

        let settlements: Vec<Settlement> = balancing_transactions.iter().map(|transaction: &Transaction| {
            assert_eq!(2, transaction.elements.len());
            
            let element0: &TransactionElement = transaction.elements.get(0).unwrap();
            let element1: &TransactionElement = transaction.elements.get(1).unwrap();

            let (sender_element, receiver_element) = if element0.amount.is_none() {
                (element1, element0)
            } else {
                (element0, element1)
            };

            // there is an assumption that the balancing transactions
            // are set to be a negative amount to take from the
            // sender's account, and an automatically calculated none
            // amount for the receiver's account.
            let amount = sender_element.amount.unwrap().neg();

            assert!(amount.gt(&zero).unwrap());
            assert!(receiver_element.amount.is_none());

            let sender = self.get_user_with_account(&sender_element.account).unwrap();
            let receiver = self.get_user_with_account(&receiver_element.account).unwrap();

            Settlement::new(sender.clone(), receiver.clone(), amount)
        }).collect();

        Ok(settlements)
    }

    fn get_user_with_account(&self, account: &Account) -> Option<Rc<User>> {
        self.users.iter().find(|u| *u.account == *account).map(|u: &Rc<User>| u.clone())
    }
}

/// Create a transaction that pays the entire debt of the account of
/// `negative_difference_state` the account of
/// `positive_difference_state`.
fn balance_entire_negative_into_positive(
    date: NaiveDate,
    negative_difference_state: &mut AccountState,
    positive_difference_state: &mut AccountState,
    zero: &Commodity,
) -> Result<Vec<Transaction>, CostingError> {
    let transactions = vec![Transaction::new_simple(
        Some("balancing"),
        date,
        negative_difference_state.account.clone(),
        positive_difference_state.account.clone(),
        negative_difference_state.amount.neg(),
        None,
    )];

    positive_difference_state.amount = positive_difference_state
        .amount
        .add(&negative_difference_state.amount)?;
    negative_difference_state.amount = *zero;

    return Ok(transactions);
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

/// Represents the settlement of a debt that one user owes another.
#[derive(Debug)]
pub struct Settlement {
    /// The user who has a debt and needs to send the money.
    sender: Rc<User>,
    /// The user who is owed money.
    receiver: Rc<User>,
    /// The amount of money the `sender` needs to send to the `receiver`.
    amount: Commodity,
}

impl Settlement {
    /// Create a new [Settlement](Settlement).
    fn new(sender: Rc<User>, receiver: Rc<User>, amount: Commodity) -> Settlement {
        Settlement {
            sender,
            receiver,
            amount,
        }
    }

    fn to_transaction(&self, date: NaiveDate) -> Transaction {
        Transaction::new_simple(
            Some("Settlement"),
            date,
            self.sender.account.clone(), 
            self.receiver.account.clone(), 
            self.amount,
            None,
        )
    }
}

/// An expense which is paid by a user on a given `date`, and which is
/// to be shared by a list of users.
#[derive(Debug)]
pub struct Expense {
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
    /// # use coster::costing::{Expense, User};
    /// use coster::accounting::{Transaction, Account};
    /// use coster::currency::{Commodity, Currency};
    /// use std::rc::Rc;
    /// use chrono::NaiveDate;
    ///
    /// let aud = Rc::from(Currency::from_alpha3("AUD").unwrap());
    /// let user1 = Rc::from(User::new("user1", "User 1", None, aud.clone()));
    /// let user2 = Rc::from(User::new("user2", "User 2", None, aud.clone()));
    ///
    /// let expenses_account = Rc::from(Account::new(Some("Expenses"), aud.clone(), None));
    ///
    /// let expense = Expense::new(
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
        description: &str,
        account: Rc<Account>,
        date: NaiveDate,
        paid_by: Rc<User>,
        shared_by: Vec<Rc<User>>,
        amount: Commodity,
        exchange_rate: Option<ExchangeRate>,
    ) -> Expense {
        Expense {
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
    /// # use coster::costing::{Expense, User};
    /// use coster::accounting::{Transaction, Account};
    /// use coster::currency::{Commodity, Currency};
    /// use std::rc::Rc;
    /// use chrono::NaiveDate;
    ///
    /// let aud = Rc::from(Currency::from_alpha3("AUD").unwrap());
    /// let user1 = Rc::from(User::new("user1", "User 1", None, aud.clone()));
    /// let user2 = Rc::from(User::new("user2", "User 2", None, aud.clone()));
    /// let user3 = Rc::from(User::new("user3", "User 3", None, aud.clone()));
    ///
    /// let expenses_account = Rc::from(Account::new(Some("Expenses"), aud.clone(), None));
    ///
    /// let expense = Expense::new(
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
    /// let user1_element = actual_transaction.get_element(&user1.account).unwrap();
    /// assert_eq!(Some(Commodity::from_str("-300.0 AUD").unwrap()), user1_element.amount);
    /// let expense_element = actual_transaction.get_element(&expenses_account).unwrap();
    /// assert_eq!(None, expense_element.amount);
    /// ```
    pub fn get_actual_transaction(&self) -> Transaction {
        Transaction::new(
            Some(self.description.clone()),
            self.date,
            vec![
                TransactionElement::new(
                    self.paid_by.account.clone(),
                    Some(self.amount.neg()),
                    self.exchange_rate.clone(),
                ),
                TransactionElement::new(self.account.clone(), None, self.exchange_rate.clone()),
            ],
        )
    }

    /// Get a transaction where this expense is shared by all the users involved
    ///
    /// # Example
    /// ```
    /// # use coster::costing::{Expense, User};
    /// use coster::accounting::{Transaction, Account};
    /// use coster::currency::{Commodity, Currency};
    /// use std::rc::Rc;
    /// use chrono::NaiveDate;
    ///
    /// let aud = Rc::from(Currency::from_alpha3("AUD").unwrap());
    /// let user1 = Rc::from(User::new("user1", "User 1", None, aud.clone()));
    /// let user2 = Rc::from(User::new("user2", "User 2", None, aud.clone()));
    /// let user3 = Rc::from(User::new("user3", "User 3", None, aud.clone()));
    ///
    /// let expenses_account = Rc::from(Account::new(Some("Expenses"), aud.clone(), None));
    ///
    /// let expense = Expense::new(
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
    /// assert!(shared_transaction.get_element(&user1.account).is_none());
    ///
    /// let user2_element = shared_transaction.get_element(&user2.account).unwrap();
    /// let user3_element = shared_transaction.get_element(&user3.account).unwrap();
    /// assert_eq!(Some(Commodity::from_str("-150.0 AUD").unwrap()), user2_element.amount);
    /// assert_eq!(Some(Commodity::from_str("-150.0 AUD").unwrap()), user3_element.amount);
    ///
    /// let expense_element = shared_transaction.get_element(&expenses_account).unwrap();
    /// assert_eq!(None, expense_element.amount);
    /// ```
    pub fn get_shared_transaction(&self) -> Transaction {
        let mut elements: Vec<TransactionElement> = Vec::new();

        // TODO: perhaps consider using divide_share instead
        let divided = self
            .amount
            .div_i64(self.shared_by.len().try_into().unwrap())
            .neg();

        for user in &self.shared_by {
            let element = TransactionElement::new(
                user.account.clone(),
                Some(divided),
                self.exchange_rate.clone(),
            );
            elements.push(element);
        }

        elements.push(TransactionElement::new(
            self.account.clone(),
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

mod tests {
    use super::{Expense, Tab, User};
    use crate::accounting::{Account, Transaction};
    use crate::currency::{Commodity, Currency};
    use chrono::NaiveDate;
    use std::rc::Rc;

    #[test]
    fn balance() {
        let aud = Rc::from(Currency::from_alpha3("AUD").unwrap());

        let user1 = Rc::from(User::new("user1", "User 1", None, aud.clone()));
        let user2 = Rc::from(User::new("user2", "User 2", None, aud.clone()));
        let user3 = Rc::from(User::new("user3", "User 3", None, aud.clone()));

        let expenses_account = Rc::from(Account::new(Some("Expenses"), aud.clone(), None));

        let expense = Expense::new(
            "some expense",
            expenses_account.clone(),
            NaiveDate::from_ymd(2020, 2, 27),
            user1.clone(),
            vec![user2.clone(), user3.clone()],
            Commodity::from_str("300.0 AUD").unwrap(),
            None,
        );

        let tab = Tab::new(
            aud.clone(),
            vec![user1.clone(), user2.clone(), user3.clone()],
            vec![expense],
        );

        let settlements = tab.balance_transactions().unwrap();
    }
}
