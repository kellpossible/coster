extern crate chrono;
extern crate iso4217;
extern crate nanoid;
extern crate rust_decimal;

use crate::currency::{Commodity, Currency, CurrencyError};
use crate::exchange_rate::ExchangeRate;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::boxed::Box;
use std::fmt;
use std::rc::Rc;
use thiserror::Error;

const DECIMAL_SCALE: u32 = 2;
const ID_SIZE: usize = 20;

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
    /// TODO: in the future implement some kind of id caching if required
    fn get_account_state(&self, account_id: &str) -> Option<&AccountState> {
        self.account_states
            .iter()
            .find(|&account_state| account_state.account.id == account_id)
    }

    /// Get a mutable reference to the `AccountState` associated with the given `Account`.
    ///
    /// TODO: in the future implement some kind of id caching if required
    fn get_mut_account_state(&mut self, account_id: &str) -> Option<&mut AccountState> {
        self.account_states
            .iter_mut()
            .find(|account_state| account_state.account.id == account_id)
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
///
/// For the creation of an account, see [ProgramState::new_account()](ProgramState::new_account)
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
            id: nanoid::generate(ID_SIZE),
            name: name,
            currency: currency,
            category: category,
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
            amount: Commodity::new(account.currency.clone(), Decimal::new(0, DECIMAL_SCALE)),
            status: status,
        }
    }
}

/// Represents an action which can modify [ProgramState](ProgramState)
pub trait Action: fmt::Display + fmt::Debug {
    /// The date/time (in the account history) that the action was performed
    fn datetime(&self) -> DateTime<Utc>;

    /// Perform the action to mutate the [ProgramState](ProgramState)
    fn perform(&self, program_state: &mut ProgramState) -> Result<(), AccountingError>;
}

pub enum ActionType {
    Transaction,
}

#[derive(Debug)]
pub struct Transaction {
    description: Option<String>,
    datetime: DateTime<Utc>,
    elements: Vec<TransactionElement>,
}

impl Transaction {
    fn new(
        description: Option<String>,
        datetime: DateTime<Utc>,
        elements: Vec<TransactionElement>,
    ) -> Result<Transaction, AccountingError> {
        //TODO: perform the commodity sum
        //TODO: decide what currency to use for the sum type
        Ok(Transaction {
            description,
            datetime,
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
    fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }

    fn perform(&self, program_state: &mut ProgramState) -> Result<(), AccountingError> {
        for transaction in &self.elements {
            let mut account_state = program_state
                .get_mut_account_state(transaction.account.id.as_ref())
                .unwrap();

            match account_state.status {
                AccountStatus::Closed => Err(AccountingError::InvalidAccountStatus {
                    account: transaction.account.clone(),
                    status: account_state.status,
                }),
                _ => Ok(()),
            }?;

            // TODO: perform the currency conversion using the exchange rate (if present)

            account_state.amount = match account_state.amount.add(&transaction.amount) {
                Ok(commodity) => commodity,
                Err(err) => {
                    return Err(AccountingError::Currency(err));
                }
            }
        }

        return Ok(());
    }
}

#[derive(Debug)]
struct TransactionElement {
    /// The account to perform the transaction to
    account: Rc<Account>,

    /// The amount of [Commodity](Commodity) to add to the account
    amount: Commodity,

    /// The exchange rate to use for converting the amount in this element
    /// to a different [Currency](Currency)
    exchange_rate: Option<ExchangeRate>,
}

impl TransactionElement {
    pub fn new(
        account: Rc<Account>,
        amount: Commodity,
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
    datetime: DateTime<Utc>,
}

impl EditAccountStatus {
    pub fn new(
        account: Rc<Account>,
        newstatus: AccountStatus,
        datetime: DateTime<Utc>,
    ) -> EditAccountStatus {
        EditAccountStatus {
            account: account,
            newstatus: newstatus,
            datetime: datetime,
        }
    }
}

impl fmt::Display for EditAccountStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Edit Account Status")
    }
}

impl Action for EditAccountStatus {
    fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }

    fn perform(&self, program_state: &mut ProgramState) -> Result<(), AccountingError> {
        let mut account_state = program_state
            .get_mut_account_state(self.account.id.as_ref())
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
        Account, AccountState, AccountStatus, Action, EditAccountStatus, Program, ProgramState,
        Transaction, TransactionElement,
    };
    use crate::currency::{Commodity, Currency};
    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use std::rc::Rc;

    #[test]
    fn execute_program() {
        let currency = Rc::from(Currency::from_alpha3(Some('$'), "AUD"));
        let account1 = Rc::from(Account::new(
            Some(String::from("Account 1")),
            currency,
            None,
        ));

        let accounts = vec![account1.clone()];

        let mut program_state = ProgramState::new(accounts);

        // TODO: change Utc::now to a string parsed value
        let open_account_action =
            EditAccountStatus::new(account1.clone(), AccountStatus::Open, Utc::now());

        // TODO: change Utc::now to a string parsed value
        let transaction1 = Transaction::new(
            Some(String::from("Transaction 1")),
            Utc::now(),
            vec![TransactionElement::new(
                account1.clone(),
                Commodity::from_str(Some('$'), "AUD", "-2.52"),
                None,
            )],
        ).unwrap();

        let actions: Vec<Box<dyn Action>> =
            vec![Box::from(open_account_action), Box::from(transaction1)];
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
            Commodity::from_str(Some('$'), "AUD", "-2.52"),
            account1_state_after.amount
        );
    }
}
