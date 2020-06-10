use crate::error::CostingError;
use crate::expense::{Expense, ExpenseID};
use crate::tab::Tab;
use crate::user::{User, UserID};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UserActionType {
    AddExpense(AddExpense),
    AddUser(AddUser),
}

/// Represents an action that a [User](crate::user::User) can perform to modify a [Tab](Tab).
pub trait UserAction: fmt::Debug {
    /// Get metadata about the action.
    fn metadata(&self) -> &UserActionMetadata;

    /// Perform the action to mutate the [Tab](Tab).
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserActionMetadata {
    pub user_id: UserID,
    pub datetime: DateTime<Utc>,
}

impl UserActionMetadata {
    pub fn new(user_id: UserID, datetime: DateTime<Utc>) -> UserActionMetadata {
        UserActionMetadata { user_id, datetime }
    }
}

// TODO: potentially remove this
impl Hash for UserActionMetadata {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user_id.hash(state);
        self.datetime.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddExpense {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    pub expense: Expense,
}

impl AddExpense {
    pub fn new(action_user_id: UserID, expense: Expense) -> AddExpense {
        AddExpense {
            metadata: UserActionMetadata::new(action_user_id, Utc::now()),
            expense,
        }
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveExpense {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    pub expense_id: ExpenseID,
}

impl RemoveExpense {
    pub fn new(action_user_id: UserID, expense_to_remove_id: UserID) -> RemoveExpense {
        RemoveExpense {
            metadata: UserActionMetadata::new(action_user_id, Utc::now()),
            expense_id: expense_to_remove_id,
        }
    }
}

impl UserAction for RemoveExpense {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        for (i, e) in tab.expenses.iter().enumerate() {
            if e.id == self.expense_id {
                tab.expenses.remove(i);
                return Ok(());
            }
        }

        return Err(CostingError::ExpenseDoesNotExistOnTab(
            self.expense_id,
            tab.id,
        ));
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeTabName {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    pub name: String,
}

impl ChangeTabName {
    pub fn new(action_user_id: UserID, name: &str) -> ChangeTabName {
        ChangeTabName {
            metadata: UserActionMetadata::new(action_user_id, Utc::now()),
            name: String::from(name),
        }
    }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddUser {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    /// The user to add to the [Tab](Tab).
    pub user_to_add: User,
}

impl AddUser {
    pub fn new(action_user_id: UserID, user_to_add: User) -> AddUser {
        AddUser {
            metadata: UserActionMetadata::new(action_user_id, Utc::now()),
            user_to_add,
        }
    }
}

impl UserAction for AddUser {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        tab.add_user(self.user_to_add.clone())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveUser {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    /// [UserID](UserID) of the [User](User) to remove from the [Tab](Tab).
    pub user_id: UserID,
}

impl RemoveUser {
    pub fn new(action_user_id: UserID, user_to_remove_id: UserID) -> RemoveUser {
        RemoveUser {
            metadata: UserActionMetadata::new(action_user_id, Utc::now()),
            user_id: user_to_remove_id,
        }
    }
}

impl UserAction for RemoveUser {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        tab.remove_user(&self.user_id)
    }
}

#[cfg(test)]
pub mod tests {
    use super::{AddExpense, AddUser, ChangeTabName, RemoveExpense, RemoveUser, UserAction};
    use crate::expense::{Expense, ExpenseCategory, ExpenseID};
    use crate::tab::Tab;
    use crate::user::{User, UserID};
    use chrono::NaiveDate;
    use commodity::{Commodity, CommodityType, CommodityTypeID};
    use rust_decimal::Decimal;
    use std::rc::Rc;
    use uuid::Uuid;

    fn create_test_commodity() -> CommodityTypeID {
        CommodityType::from_currency_alpha3("USD").unwrap().id
    }

    fn create_test_tab() -> Tab {
        Tab::new(Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(), "Test Tab", create_test_commodity(), vec![], vec![])
    }

    fn create_test_user(id: UserID, name: &str) -> Rc<User> {
        let email = format!("{}@test.com", name);
        Rc::from(User::new(id, name, Some(email.as_ref())))
    }

    fn create_test_expense(
        id: ExpenseID,
        category: ExpenseCategory,
        paid_by: UserID,
        shared_by: Vec<UserID>,
    ) -> Expense {
        let description = format!("Test Expense {}", id);
        Expense::new(
            id,
            description,
            category,
            NaiveDate::from_ymd(2020, 05, 01),
            paid_by,
            shared_by,
            Commodity::new(Decimal::new(1, 0), create_test_commodity()),
            None,
        )
    }

    #[test]
    fn add_user() {
        let mut tab = create_test_tab();
        let user0 = create_test_user(0, "User 0");
        let user1 = create_test_user(1, "User 1");
        let action = AddUser::new(user0.id, (*user1).clone());

        assert_eq!(0, tab.users().len());

        action.perform(&mut tab).unwrap();

        assert_eq!(1, tab.users().len());
        assert_eq!(1, tab.users().get(0).unwrap().id);
    }

    #[test]
    fn remove_user() {
        let mut tab = create_test_tab();

        let user0 = create_test_user(0, "User 0");
        let user1 = create_test_user(1, "User 1");
        tab.add_user((*user1).clone()).unwrap();

        let action = RemoveUser::new(user0.id, user1.id);

        assert_eq!(1, tab.users().len());

        action.perform(&mut tab).unwrap();

        assert_eq!(0, tab.users().len());
    }

    #[test]
    fn add_expense() {
        let mut tab = create_test_tab();

        let user0 = create_test_user(0, "User 0");
        let user1 = create_test_user(1, "User 1");

        tab.add_user((*user0).clone()).unwrap();
        tab.add_user((*user1).clone()).unwrap();

        let expense =
            create_test_expense(0, "General".to_string(), user0.id, vec![user0.id, user1.id]);

        let action = AddExpense::new(user0.id, expense);

        assert_eq!(0, tab.expenses.len());
        action.perform(&mut tab).unwrap();
        assert_eq!(1, tab.expenses.len());
        assert_eq!(0, tab.expenses.get(0).unwrap().id);
    }

    #[test]
    fn remove_expense() {
        let mut tab = create_test_tab();

        let user0 = create_test_user(0, "User 0");
        let user1 = create_test_user(1, "User 1");

        let expense =
            create_test_expense(0, "Test".to_string(), user0.id, vec![user0.id, user1.id]);

        let action = RemoveExpense::new(user0.id, expense.id);

        tab.expenses.push(expense);

        assert_eq!(1, tab.expenses.len());
        action.perform(&mut tab).unwrap();
        assert_eq!(0, tab.expenses.len());
    }

    #[test]
    fn change_tab_name() {
        let mut tab = create_test_tab();
        let user0 = create_test_user(0, "User 0");

        let action = ChangeTabName::new(user0.id, "New Name");

        assert_eq!("Test Tab", tab.name);
        action.perform(&mut tab).unwrap();
        assert_eq!("New Name", tab.name);
    }
}
