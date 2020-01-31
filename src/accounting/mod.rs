/// So I need:
use crate::currency::Currency;
use chrono::{DateTime, Utc};
use std::rc::Rc;
use std::boxed::Box;

pub struct Program {
    actions: Vec<Box<dyn Action>>
}

impl Program {
    pub fn execute(&self, program_state: &mut ProgramState) {
        for action in &self.actions {
            action.perform(program_state);
        }
    }
}

pub struct ProgramState {
    accounts: Vec<Account>,
    account_states: Vec<AccountState>,
}

impl ProgramState {
    /// Get a reference to the `AccountState` associated with a given `Account`.
    /// 
    /// TODO: in the future implement some kind of id caching if required
    fn get_account_state(&self, account: Rc<Account>) -> Option<&AccountState> {
        let mut iter = self.account_states.iter();
        return iter.find(|&account_state| account_state.account == account);
    }

    /// Get a mutable reference to the `AccountState` associated with the given `Account`.
    /// 
    /// TODO: in the future implement some kind of id caching if required
    fn get_mut_account_state(&mut self, account: Rc<Account>) -> Option<&mut AccountState> {
        let mut iter = self.account_states.iter_mut();
        return iter.find(|account_state| account_state.account == account);
    }
}

#[derive(Copy, Clone)]
pub enum AccountStatus {
    OPEN,
    CLOSED,
}

pub struct AccountCategory {
    name: String,
    parent: Option<Rc<AccountCategory>>,
}

pub struct Account {
    id: String,
    name: Option<String>,
    category: Option<Rc<AccountCategory>>,
}

impl PartialEq for Account {
    fn eq(&self, other: &Account) -> bool {
        self.id == other.id
    }    
}

/// Mutable state associated with an `Account`
struct AccountState {
    account: Rc<Account>,
    amount: Currency,
    status: AccountStatus,
}

struct TransactionElement {
    account: Rc<Account>,
    amount: Currency,
}

pub trait Action{
    fn datetime(&self) -> DateTime<Utc>;
    fn perform(&self, program_state: &mut ProgramState);
}

pub struct Transaction {
    description: String,
    datetime: DateTime<Utc>,
    elements: Vec<TransactionElement>
}

impl Action for Transaction{
    fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }

    fn perform(&self, program_state: &mut ProgramState) {
        
    }
}

pub struct EditAccountStatus {
    account: Rc<Account>,
    newstatus: AccountStatus,
    datetime: DateTime<Utc>,
}

impl Action for EditAccountStatus {
    fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }

    /// Perform an `Action` to mutate the `ProgramState`
    /// TODO: return an error type (instead of unwrap)
    fn perform(&self, program_state: &mut ProgramState) {
        let mut account_state = program_state.get_mut_account_state(self.account.clone()).unwrap();
        account_state.status = self.newstatus;
    }
}

// create a list of actions with associated dates
// a transaction is a type of action
// opening an account is another type of action
// the list is a program which will be executed, to compute
// the final resulting values. All should add up to zero.