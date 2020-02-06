extern crate chrono;
extern crate iso4217;
extern crate nanoid;
extern crate rust_decimal;

use crate::currency::{Commodity, Currency, CurrencyError, CurrencyCode};
use crate::exchange_rate::ExchangeRate;

use chrono::NaiveDate;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use std::boxed::Box;
use std::fmt;
use std::rc::Rc;
use thiserror::Error;

const DECIMAL_SCALE: u32 = 2;
const ACCOUNT_ID_SIZE: usize = 20;

/// TODO: add context for the error for where it occurred
/// within the `Program`
#[derive(Error, Debug)]
pub enum AccountingError {
    #[error("error relating to currencies")]
    Currency(#[from] CurrencyError),
    #[error("invalid account status ({:?}) for account {}", .status, .account.id)]
    InvalidAccountStatus {
        account: Rc<Account>,
        status: AccountStatus,
    },
    #[error("error parsing a date from string")]
    DateParseError(#[from] chrono::ParseError),
    #[error("invalid transaction {0:?} because {1}")]
    InvalidTransaction(Transaction, String),
    #[error("failed checksum, the sum of account values in the common currency ({0}) does not equal zero")]
    FailedCheckSum(Commodity),
}

pub struct Program {
    actions: Vec<Box<dyn Action>>,
}

impl Program {
    pub fn new(actions: Vec<Box<dyn Action>>) -> Program {
        Program { actions }
    }

    pub fn execute(&self, program_state: &mut ProgramState) -> Result<(), AccountingError> {
        for (index, action) in self.actions.iter().enumerate() {
            action.perform(program_state)?;
            program_state.current_action_index = index;
        }

        let account_sum = program_state.sum_accounts();
        if account_sum != Commodity::zero(CurrencyCode::Common) {
            return Err(AccountingError::FailedCheckSum(account_sum));
        }

        Ok(())
    }
}

pub struct ProgramState {
    /// list of accounts (can only grow)
    accounts: Vec<Rc<Account>>,

    /// list of states associated with accounts (can only grow)
    account_states: Vec<AccountState>,

    /// the index of the currently executing action
    current_action_index: usize,
}

impl ProgramState {
    pub fn new(accounts: Vec<Rc<Account>>) -> ProgramState {
        let account_states = accounts
            .iter()
            .map(|account: &Rc<Account>| AccountState::new_default(account.clone()))
            .collect();

        ProgramState {
            accounts,
            account_states,
            current_action_index: 0,
        }
    }

    /// Get a reference to the `AccountState` associated with a given `Account`.
    ///
    /// TODO: performance, in the future implement some kind of id caching if required
    fn get_account_state(&self, account_id: &str) -> Option<&AccountState> {
        self.account_states
            .iter()
            .find(|&account_state| account_state.account.id == account_id)
    }

    /// Get a mutable reference to the `AccountState` associated with the given `Account`.
    ///
    /// TODO: performance, in the future implement some kind of id caching if required
    fn get_account_state_mut(&mut self, account_id: &str) -> Option<&mut AccountState> {
        self.account_states
            .iter_mut()
            .find(|account_state| account_state.account.id == account_id)
    }

    fn sum_accounts(&self, exchange_rate: &ExchangeRate) -> Commodity {
        let mut sum = Commodity::zero(CurrencyCode::Common);

        for account_state in &self.account_states {
            account_state.amount
        }
        
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AccountStatus {
    Open,
    Closed,
}

#[derive(Debug)]
pub struct AccountCategory {
    pub name: String,
    pub parent: Option<Rc<AccountCategory>>,
}

/// Details for an account, which holds a [Commodity](Commodity)
/// with a type of [Currency](Currency).
#[derive(Debug, Clone)]
pub struct Account {
    /// A unique identifier for this `Account`
    pub id: String,

    /// The name of this `Account`
    pub name: Option<String>,

    /// The type of currency to be stored in this account
    pub currency: Rc<Currency>,

    /// The category that this account part of
    pub category: Option<Rc<AccountCategory>>,
}

impl Account {
    /// Create a new account and add it to this program state (and create its associated
    /// [AccountState](AccountState))
    pub fn new(
        name: Option<String>,
        currency: Rc<Currency>,
        category: Option<Rc<AccountCategory>>,
    ) -> Account {
        Account {
            id: nanoid::generate(ACCOUNT_ID_SIZE),
            name,
            currency,
            category,
        }
    }
}

/// Mutable state associated with an [Account](Account)
#[derive(Debug, Clone)]
struct AccountState {
    /// The [Account](Account) associated with this state
    account: Rc<Account>,

    /// The amount of the commodity currently stored in this account
    amount: Commodity,

    /// The status of this account (open/closed/etc...)
    status: AccountStatus,
}

impl AccountState {
    fn new(account: Rc<Account>, amount: Commodity, status: AccountStatus) -> AccountState {
        AccountState {
            account,
            amount,
            status,
        }
    }

    fn new_default(account: Rc<Account>) -> AccountState {
        AccountState::new_default_amount(account, AccountStatus::Closed)
    }

    fn new_default_amount(account: Rc<Account>, status: AccountStatus) -> AccountState {
        AccountState {
            account: account.clone(),
            amount: Commodity::new(Decimal::new(0, DECIMAL_SCALE), account.currency.code),
            status: status,
        }
    }
}

/// Represents an action which can modify [ProgramState](ProgramState)
pub trait Action: fmt::Display + fmt::Debug {
    /// The date/time (in the account history) that the action was performed
    fn date(&self) -> NaiveDate;

    /// Perform the action to mutate the [ProgramState](ProgramState)
    fn perform(&self, program_state: &mut ProgramState) -> Result<(), AccountingError>;
}

pub enum ActionType {
    Transaction,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    description: Option<String>,
    date: NaiveDate,
    elements: Vec<TransactionElement>,
}

impl Transaction {
    fn new(
        description: Option<String>,
        date: NaiveDate,
        elements: Vec<TransactionElement>,
    ) -> Result<Transaction, AccountingError> {
        //TODO: perform the commodity sum
        //TODO: decide what currency to use for the sum type
        Ok(Transaction {
            description,
            date,
            elements,
        })
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction")
    }
}

impl Action for Transaction {
    fn date(&self) -> NaiveDate {
        self.date
    }

    fn perform(&self, program_state: &mut ProgramState) -> Result<(), AccountingError> {
        // check that the transaction has at least 2 elements
        if self.elements.len() < 2 {
            return Err(AccountingError::InvalidTransaction(
                self.clone(),
                String::from("a transaction cannot have less than 2 elements"),
            ));
        }

        //TODO: add check to ensure that transaction doesn't have duplicate account references?

        // first process the elements to automatically calculate amounts

        let mut empty_amount_element: Option<usize> = None;
        for (i, element) in self.elements.iter().enumerate() {
            if element.amount.is_none() {
                if empty_amount_element.is_none() {
                    empty_amount_element = Some(i)
                } else {
                    return Err(AccountingError::InvalidTransaction(
                        self.clone(),
                        String::from("multiple elements with no amount specified"),
                    ));
                }
            }
        }

        let mut sum_currency = match empty_amount_element {
            Some(empty_i) => {
                let empty_element = self.elements.get(empty_i).unwrap();
                empty_element.account.currency.clone()
            }
            None => self
                .elements
                .get(0)
                .expect("there should be at least 2 elements in the transaction")
                .account
                .currency
                .clone(),
        };

        let mut sum = Commodity::new(Decimal::zero(), sum_currency.code);

        let mut modified_elements = self.elements.clone();

        for (i, element) in self.elements.iter().enumerate() {
            match empty_amount_element {
                Some(empty_i) => {
                    if i != empty_i {
                        //TODO: perform currency conversion here if required
                        sum = match sum.add(&element.amount.as_ref().unwrap()) {
                            Ok(value) => value,
                            Err(error) => return Err(AccountingError::Currency(error)),
                        }
                    }
                }
                None => {}
            }
        }

        match empty_amount_element {
            Some(empty_i) => {
                let modified_emtpy_element: &mut TransactionElement =
                    modified_elements.get_mut(empty_i).unwrap();
                let negated_sum = sum.negate();
                modified_emtpy_element.amount = Some(negated_sum.clone());

                sum = match sum.add(&negated_sum) {
                    Ok(value) => value,
                    Err(error) => return Err(AccountingError::Currency(error)),
                }
            }
            None => {}
        };

        if sum.value != Decimal::zero() {
            return Err(AccountingError::InvalidTransaction(
                self.clone(),
                String::from("sum of transaction elements does not equal zero"),
            ));
        }

        for transaction in &modified_elements {
            let mut account_state = program_state
                .get_account_state_mut(transaction.account.id.as_ref())
                .expect(
                    format!(
                        "unable to find state for account with id: {}, please ensure this 
                account was added to the program state before execution.",
                        transaction.account.id
                    )
                    .as_ref(),
                );

            match account_state.status {
                AccountStatus::Closed => Err(AccountingError::InvalidAccountStatus {
                    account: transaction.account.clone(),
                    status: account_state.status,
                }),
                _ => Ok(()),
            }?;

            // TODO: perform the currency conversion using the exchange rate (if present)

            let transaction_amount = match &transaction.amount {
                Some(amount) => amount,
                None => {
                    return Err(AccountingError::InvalidTransaction(
                        self.clone(),
                        String::from(
                            "unable to calculate all required amounts for this transaction",
                        ),
                    ))
                }
            };

            account_state.amount = match account_state.amount.add(transaction_amount) {
                Ok(commodity) => commodity,
                Err(err) => {
                    return Err(AccountingError::Currency(err));
                }
            }
        }

        return Ok(());
    }
}

#[derive(Debug, Clone)]
struct TransactionElement {
    /// The account to perform the transaction to
    pub account: Rc<Account>,

    /// The amount of [Commodity](Commodity) to add to the account
    pub amount: Option<Commodity>,

    /// The exchange rate to use for converting the amount in this element
    /// to a different [Currency](Currency)
    pub exchange_rate: Option<ExchangeRate>,
}

impl TransactionElement {
    pub fn new(
        account: Rc<Account>,
        amount: Option<Commodity>,
        exchange_rate: Option<ExchangeRate>,
    ) -> TransactionElement {
        TransactionElement {
            account,
            amount,
            exchange_rate,
        }
    }
}

#[derive(Debug)]
pub struct EditAccountStatus {
    account: Rc<Account>,
    newstatus: AccountStatus,
    date: NaiveDate,
}

impl EditAccountStatus {
    pub fn new(
        account: Rc<Account>,
        newstatus: AccountStatus,
        date: NaiveDate,
    ) -> EditAccountStatus {
        EditAccountStatus {
            account: account,
            newstatus: newstatus,
            date: date,
        }
    }
}

impl fmt::Display for EditAccountStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Edit Account Status")
    }
}

impl Action for EditAccountStatus {
    fn date(&self) -> NaiveDate {
        self.date
    }

    fn perform(&self, program_state: &mut ProgramState) -> Result<(), AccountingError> {
        let mut account_state = program_state
            .get_account_state_mut(self.account.id.as_ref())
            .unwrap();
        account_state.status = self.newstatus;
        return Ok(());
    }
}

// create a list of actions with associated dates
// a transaction is a type of action
// opening an account is another type of action
// the list is a program which will be executed, to compute
// the final resulting values. All should add up to zero.

#[cfg(test)]
mod tests {
    use super::{
        Account, AccountState, AccountStatus, Action, EditAccountStatus, NaiveDate, Program,
        ProgramState, Transaction, TransactionElement,
    };
    use crate::currency::{Commodity, Currency, CurrencyCode};
    use std::rc::Rc;
    use std::str::FromStr;

    #[test]
    fn execute_program() {
        let currency = Rc::from(Currency::new(CurrencyCode::from_str("AUD").unwrap(), None));
        let account1 = Rc::from(Account::new(
            Some(String::from("Account 1")),
            currency.clone(),
            None,
        ));

        let account2 = Rc::from(Account::new(
            Some(String::from("Account 2")),
            currency.clone(),
            None,
        ));

        let accounts = vec![account1.clone(), account2.clone()];

        let mut program_state = ProgramState::new(accounts);

        let open_account1 = EditAccountStatus::new(
            account1.clone(),
            AccountStatus::Open,
            NaiveDate::from_str("2020-01-01").unwrap(),
        );

        let open_account2 = EditAccountStatus::new(
            account2.clone(),
            AccountStatus::Open,
            NaiveDate::from_str("2020-01-01").unwrap(),
        );

        let transaction1 = Transaction::new(
            Some(String::from("Transaction 1")),
            NaiveDate::from_str("2020-01-02").unwrap(),
            vec![
                TransactionElement::new(
                    account1.clone(),
                    Some(Commodity::from_str("-2.52", "AUD").unwrap()),
                    None,
                ),
                TransactionElement::new(
                    account2.clone(),
                    Some(Commodity::from_str("2.52", "AUD").unwrap()),
                    None,
                ),
            ],
        )
        .unwrap();

        let transaction2 = Transaction::new(
            Some(String::from("Transaction 2")),
            NaiveDate::from_str("2020-01-02").unwrap(),
            vec![
                TransactionElement::new(
                    account1.clone(),
                    Some(Commodity::from_str("-1.0", "AUD").unwrap()),
                    None,
                ),
                TransactionElement::new(
                    account2.clone(),
                    None,
                    None,
                ),
            ],
        )
        .unwrap();

        let actions: Vec<Box<dyn Action>> = vec![
            Box::from(open_account1),
            Box::from(open_account2),
            Box::from(transaction1),
            Box::from(transaction2),
        ];

        let program = Program::new(actions);

        let account1_state_before: AccountState = program_state
            .get_account_state(account1.id.as_ref())
            .unwrap()
            .clone();

        assert_eq!(AccountStatus::Closed, account1_state_before.status);

        program.execute(&mut program_state).unwrap();

        let account1_state_after: AccountState = program_state
            .get_account_state(account1.id.as_ref())
            .unwrap()
            .clone();

        assert_eq!(AccountStatus::Open, account1_state_after.status);
        assert_eq!(
            Commodity::from_str("-3.52", "AUD").unwrap(),
            account1_state_after.amount
        );
    }
}
