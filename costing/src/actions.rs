use crate::error::CostingError;
use crate::expense::{Expense, ExpenseID};
use crate::tab::Tab;
use crate::user::{User, UserID};
use chrono::{DateTime, Utc};
use nanoid::nanoid;
use std::fmt;
use std::{
    hash::Hash,
    rc::{Rc, Weak},
};

/// Represents an action that a [User](crate::user::User) can perform to modify a [Tab](Tab).
pub trait UserAction: fmt::Debug {
    /// Get metadata about the action.
    fn metadata(&self) -> &UserActionMetadata;

    /// Perform the action to mutate the [Tab](Tab).
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError>;
}

#[derive(Debug)]
pub struct UserActionMetadata {
    pub user: Rc<User>,
    pub datetime: DateTime<Utc>,
    pub previous_action: Weak<dyn UserAction>,
}

// TODO: potentially remove this
impl Hash for UserActionMetadata {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user.id.hash(state);
        self.datetime.hash(state);
        match self.previous_action.upgrade() {
            Some(action) => {
                // action.hash(state)
            }
            None => {}
        }
    }
}

#[derive(Debug)]
pub struct AddExpense {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    pub expense: Expense,
}

impl UserAction for AddExpense {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        match tab.expenses.iter().find(|e| e.id == self.expense.id) {
            Some(expense) => Err(CostingError::ExpenseAlreadyExistsOnTab(expense.id, tab.id)),
            None => {
                tab.expenses.push(self.expense.clone());
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct RemoveExpense {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    pub expense_id: ExpenseID,
}

impl UserAction for RemoveExpense {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        for (i, e) in tab.expenses.iter().enumerate() {
            if e.id == self.expense_id {
                tab.expenses.remove(i);
                return Ok(())
            }
        }

        return Err(CostingError::ExpenseDoesNotExistOntab(self.expense_id, tab.id));
    }
}

#[derive(Debug)]
pub struct ChangeTabName {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    pub name: String,
}

impl UserAction for ChangeTabName {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        tab.name = self.name.clone();
        Ok(())
    }
}

#[derive(Debug)]
pub struct AddUser {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    /// The user to add to the [Tab](Tab).
    pub user: Rc<User>,
}

impl UserAction for AddUser {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        match tab.users.iter().find(|u| u.id == self.user.id) {
            Some(user) => Err(CostingError::UserAlreadyExistsOnTab(user.id, tab.id)),
            None => {
                tab.users.push(self.user.clone());
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct RemoveUser {
    /// Metadata about this action.
    metadata: UserActionMetadata,
    /// [UserID](UserID) of the [User](User) to remove from the [Tab](Tab).
    pub user_id: UserID,
}

impl UserAction for RemoveUser {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        for (i, u) in tab.users.iter().enumerate() {
            if u.id == self.user_id {
                tab.users.remove(i);
                return Ok(())
            }
        }

        return Err(CostingError::UserDoesNotExistOnTab(self.user_id, tab.id));
    }
}