//! A double entry accounting system

extern crate chrono;
extern crate iso4217;
extern crate nanoid;
extern crate rust_decimal;

use crate::currency::{Commodity, Currency, CurrencyCode, CurrencyError};
use crate::exchange_rate::{ExchangeRate, ExchangeRateError};
use std::collections::HashMap;

use chrono::NaiveDate;
use nanoid::nanoid;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use std::boxed::Box;
use std::fmt;
use std::rc::Rc;
use thiserror::Error;

const DECIMAL_SCALE: u32 = 2;
const ACCOUNT_ID_SIZE: usize = 20;

/// TODO: add context for the error for where it occurred within the [Program](Program)
#[derive(Error, Debug)]
pub enum AccountingError {
    #[error("error relating to currencies")]
    Currency(#[from] CurrencyError),
    #[error("error relating to exchange rates")]
    ExchangeRate(#[from] ExchangeRateError),
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
    #[error("no exchange rate supplied, unable to convert commodity {0} to currency {1}")]
    NoExchangeRateSupplied(Commodity, CurrencyCode),
    #[error("the account state with the id {0} was requested but cannot be found")]
    MissingAccountState(AccountID),
}

pub struct Program {
    actions: Vec<Box<dyn Action>>,
}

impl Program {
    pub fn new(actions: Vec<Box<dyn Action>>) -> Program {
        Program { actions }
    }
}

pub struct ProgramState {
    /// list of states associated with accounts (can only grow)
    pub account_states: HashMap<AccountID, AccountState>,

    /// the index of the currently executing action
    current_action_index: usize,
}

/// Sum the values in all the accounts into a single [Commodity](Commodity), and
/// use the supplied exchange rate if required to convert a currency in an account
/// to the `sum_currency`.
pub fn sum_account_states(
    account_states: &HashMap<AccountID, AccountState>,
    sum_currency: CurrencyCode,
    exchange_rate: Option<&ExchangeRate>,
) -> Result<Commodity, AccountingError> {
    let mut sum = Commodity::zero(sum_currency);

    for (key, account_state) in account_states {
        let account_amount = if account_state.amount.currency_code != sum_currency {
            match exchange_rate {
                Some(rate) => rate.convert(account_state.amount, sum_currency)?,
                None => {
                    return Err(AccountingError::NoExchangeRateSupplied(
                        account_state.amount,
                        sum_currency,
                    ))
                }
            }
        } else {
            account_state.amount
        };

        sum = sum.add(&account_amount)?;
    }

    Ok(sum)
}

impl ProgramState {
    pub fn new(accounts: Vec<Rc<Account>>) -> ProgramState {
        let mut account_states = HashMap::new();

        for account in accounts {
            account_states.insert(
                account.id.clone(),
                AccountState::new_default(account.clone()),
            );
        }

        ProgramState {
            account_states,
            current_action_index: 0,
        }
    }

    pub fn execute_program(&mut self, program: &Program) -> Result<(), AccountingError> {
        for (index, action) in program.actions.iter().enumerate() {
            action.perform(self)?;
            self.current_action_index = index;
        }

        Ok(())
    }

    /// Get a reference to the `AccountState` associated with a given `Account`.
    ///
    /// TODO: performance, in the future implement some kind of id caching if required
    fn get_account_state(&self, account_id: &AccountID) -> Option<&AccountState> {
        self.account_states.get(account_id)
    }

    /// Get a mutable reference to the `AccountState` associated with the given `Account`.
    ///
    /// TODO: performance, in the future implement some kind of id caching if required
    fn get_account_state_mut(&mut self, account_id: &AccountID) -> Option<&mut AccountState> {
        self.account_states.get_mut(account_id)
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

pub type AccountID = String;

/// Details for an account, which holds a [Commodity](Commodity)
/// with a type of [Currency](Currency).
#[derive(Debug, Clone)]
pub struct Account {
    /// A unique identifier for this `Account`
    pub id: AccountID,

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
        name: Option<&str>,
        currency: Rc<Currency>,
        category: Option<Rc<AccountCategory>>,
    ) -> Account {
        Account {
            id: nanoid!(ACCOUNT_ID_SIZE),
            name: name.map(|s| String::from(s)),
            currency,
            category,
        }
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Account) -> bool {
        self.id == other.id
    }
}

/// Mutable state associated with an [Account](Account)
#[derive(Debug, Clone)]
pub struct AccountState {
    /// The [Account](Account) associated with this state
    pub account: Rc<Account>,

    /// The amount of the commodity currently stored in this account
    pub amount: Commodity,

    /// The status of this account (open/closed/etc...)
    pub status: AccountStatus,
}

impl AccountState {
    pub fn new(account: Rc<Account>, amount: Commodity, status: AccountStatus) -> AccountState {
        AccountState {
            account,
            amount,
            status,
        }
    }

    pub fn new_default(account: Rc<Account>) -> AccountState {
        AccountState::new_default_amount(account, AccountStatus::Closed)
    }

    pub fn new_default_amount(account: Rc<Account>, status: AccountStatus) -> AccountState {
        AccountState {
            account: account.clone(),
            amount: Commodity::new(Decimal::new(0, DECIMAL_SCALE), account.currency.code),
            status,
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
    pub description: Option<String>,
    pub date: NaiveDate,
    pub elements: Vec<TransactionElement>,
}

impl Transaction {
    pub fn new(
        description: Option<String>,
        date: NaiveDate,
        elements: Vec<TransactionElement>,
    ) -> Transaction {
        Transaction {
            description,
            date,
            elements,
        }
    }

    pub fn get_element(&self, account: &Account) -> Option<&TransactionElement> {
        self.elements.iter().find(|e| e.account.as_ref() == account)
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

        let sum_currency = match empty_amount_element {
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

        // Calculate the sum of elements (not including the empty element if there is one)
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

        // Calculate the value to use for the empty element (negate the sum of the other elements)
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
                .get_account_state_mut(&transaction.account.id)
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
pub struct TransactionElement {
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
            account,
            newstatus,
            date,
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
            .get_account_state_mut(&self.account.id)
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
        sum_account_states, Account, AccountState, AccountStatus, Action, EditAccountStatus,
        NaiveDate, Program, ProgramState, Transaction, TransactionElement,
    };
    use crate::currency::{Commodity, Currency, CurrencyCode};
    use std::rc::Rc;
    use std::str::FromStr;

    #[test]
    fn execute_program() {
        let currency = Rc::from(Currency::new(CurrencyCode::from_str("AUD").unwrap(), None));
        let account1 = Rc::from(Account::new(Some("Account 1"), currency.clone(), None));

        let account2 = Rc::from(Account::new(Some("Account 2"), currency.clone(), None));

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
                    Some(Commodity::from_str("-2.52 AUD").unwrap()),
                    None,
                ),
                TransactionElement::new(
                    account2.clone(),
                    Some(Commodity::from_str("2.52 AUD").unwrap()),
                    None,
                ),
            ],
        );

        let transaction2 = Transaction::new(
            Some(String::from("Transaction 2")),
            NaiveDate::from_str("2020-01-02").unwrap(),
            vec![
                TransactionElement::new(
                    account1.clone(),
                    Some(Commodity::from_str("-1.0 AUD").unwrap()),
                    None,
                ),
                TransactionElement::new(account2.clone(), None, None),
            ],
        );

        let actions: Vec<Box<dyn Action>> = vec![
            Box::from(open_account1),
            Box::from(open_account2),
            Box::from(transaction1),
            Box::from(transaction2),
        ];

        let program = Program::new(actions);

        let account1_state_before: AccountState = program_state
            .get_account_state(&account1.id)
            .unwrap()
            .clone();

        assert_eq!(AccountStatus::Closed, account1_state_before.status);

        program_state.execute_program(&program).unwrap();

        let account1_state_after: AccountState = program_state
            .get_account_state(&account1.id)
            .unwrap()
            .clone();

        assert_eq!(AccountStatus::Open, account1_state_after.status);
        assert_eq!(
            Commodity::from_str("-3.52 AUD").unwrap(),
            account1_state_after.amount
        );

        assert_eq!(
            Commodity::from_str("0.0 AUD").unwrap(),
            sum_account_states(
                &program_state.account_states,
                CurrencyCode::from_str("AUD").unwrap(),
                None
            )
            .unwrap()
        );
    }
}
