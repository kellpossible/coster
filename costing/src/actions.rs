use crate::error::CostingError;
use crate::expense::Expense;
use crate::tab::Tab;
use crate::user::User;
use chrono::{DateTime, Utc};
use enum_dispatch::enum_dispatch;
use fmt::Display;
use std::fmt;
use std::rc::{Rc, Weak};

/// Represents an action that a [User](crate::user::User) can perform to modify a [Tab](Tab).
#[enum_dispatch]
pub trait UserAction: fmt::Display + fmt::Debug {
    /// Get metadata about the user action.
    fn metadata(&self) -> &UserActionMetadata;

    /// Perform the action to mutate the [Tab](Tab).
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError>;
}

#[enum_dispatch(UserAction)]
#[derive(Debug)]
pub enum UserActionType {
    AddExpense,
    RemoveExpense,
}

impl Display for UserActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserActionType::AddExpense(add_expense) => add_expense.fmt(f),
            UserActionType::RemoveExpense(remove_expense) => remove_expense.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct UserActionMetadata {
    pub user: Rc<User>,
    pub datetime: DateTime<Utc>,
    pub previous_action: Option<Weak<UserActionType>>
}

#[derive(Debug)]
pub struct AddExpense {
    pub metadata: UserActionMetadata,
    pub expense: Rc<Expense>,
}

impl UserAction for AddExpense {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        unimplemented!()
    }
}

impl Display for AddExpense {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Add Expense")
    }
}

#[derive(Debug)]
pub struct RemoveExpense {
    pub metadata: UserActionMetadata,
    pub user: Rc<User>,
}

impl UserAction for RemoveExpense {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        unimplemented!()
    }
}

impl Display for RemoveExpense {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Remove Expense")
    }
}
