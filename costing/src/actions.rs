use crate::error::CostingError;
use crate::expense::{Expense, ExpenseID};
use crate::tab::Tab;
use crate::user::{User, UserID};
use chrono::{DateTime, Utc};
use std::fmt;
use std::{hash::Hash, rc::Rc};

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
}

impl UserActionMetadata {
    pub fn new(user: Rc<User>, datetime: DateTime<Utc>) -> UserActionMetadata {
        UserActionMetadata { user, datetime }
    }
}

// TODO: potentially remove this
impl Hash for UserActionMetadata {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user.id.hash(state);
        self.datetime.hash(state);
    }
}

#[derive(Debug)]
pub struct AddExpense {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    pub expense: Expense,
}

impl AddExpense {
    pub fn new(action_user: Rc<User>, expense: Expense) -> AddExpense {
        AddExpense {
            metadata: UserActionMetadata::new(action_user, Utc::now()),
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

#[derive(Debug)]
pub struct RemoveExpense {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    pub expense_id: ExpenseID,
}

impl RemoveExpense {
    pub fn new(action_user: Rc<User>, expense_to_remove_id: UserID) -> RemoveExpense {
        RemoveExpense {
            metadata: UserActionMetadata::new(action_user, Utc::now()),
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

        return Err(CostingError::ExpenseDoesNotExistOntab(
            self.expense_id,
            tab.id,
        ));
    }
}

#[derive(Debug)]
pub struct ChangeTabName {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    pub name: String,
}

impl ChangeTabName {
    pub fn new(action_user: Rc<User>, name: &str) -> ChangeTabName {
        ChangeTabName {
            metadata: UserActionMetadata::new(action_user, Utc::now()),
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

#[derive(Debug)]
pub struct AddUser {
    /// Metadata about this action.
    pub metadata: UserActionMetadata,
    /// The user to add to the [Tab](Tab).
    pub user: Rc<User>,
}

impl AddUser {
    pub fn new(action_user: Rc<User>, user_to_add: Rc<User>) -> AddUser {
        AddUser {
            metadata: UserActionMetadata::new(action_user, Utc::now()),
            user: user_to_add,
        }
    }
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
    pub metadata: UserActionMetadata,
    /// [UserID](UserID) of the [User](User) to remove from the [Tab](Tab).
    pub user_id: UserID,
}

impl RemoveUser {
    pub fn new(action_user: Rc<User>, user_to_remove_id: UserID) -> RemoveUser {
        RemoveUser {
            metadata: UserActionMetadata::new(action_user, Utc::now()),
            user_id: user_to_remove_id,
        }
    }
}

impl UserAction for RemoveUser {
    fn metadata(&self) -> &UserActionMetadata {
        &self.metadata
    }
    fn perform(&self, tab: &mut Tab) -> Result<(), CostingError> {
        for (i, u) in tab.users.iter().enumerate() {
            if u.id == self.user_id {
                tab.users.remove(i);
                return Ok(());
            }
        }

        return Err(CostingError::UserDoesNotExistOnTab(self.user_id, tab.id));
    }
}

#[cfg(test)]
pub mod tests {
    use super::{AddExpense, AddUser, ChangeTabName, RemoveExpense, RemoveUser, UserAction};
    use crate::expense::{Expense, ExpenseID};
    use crate::tab::Tab;
    use crate::user::{User, UserID};
    use chrono::NaiveDate;
    use commodity::{Commodity, CommodityType};
    use doublecount::Account;
    use rust_decimal::Decimal;
    use std::rc::Rc;

    fn create_test_commodity() -> Rc<CommodityType> {
        Rc::from(CommodityType::from_currency_alpha3("USD").unwrap())
    }

    fn create_test_tab() -> Tab {
        Tab::new(0, "Test Tab", create_test_commodity(), vec![], vec![])
    }

    fn create_test_user(id: UserID, name: &str) -> Rc<User> {
        let email = format!("{}@test.com", name);
        Rc::from(User::new(
            id,
            name,
            Some(email.as_ref()),
            create_test_commodity(),
        ))
    }

    fn create_test_account(name: &str) -> Rc<Account> {
        Rc::from(Account::new(Some(name), create_test_commodity().id, None))
    }

    fn create_test_expense(
        id: ExpenseID,
        account: Rc<Account>,
        paid_by: Rc<User>,
        shared_by: Vec<Rc<User>>,
    ) -> Expense {
        let description = format!("Test Expense {}", id);
        Expense::new(
            id,
            description.as_ref(),
            account,
            NaiveDate::from_ymd(2020, 05, 01),
            paid_by,
            shared_by,
            Commodity::new(Decimal::new(1, 0), create_test_commodity().id),
            None,
        )
    }

    #[test]
    fn add_user() {
        let mut tab = create_test_tab();
        let action = AddUser::new(create_test_user(0, "User 0"), create_test_user(1, "User 1"));

        assert_eq!(0, tab.users.len());

        action.perform(&mut tab).unwrap();

        assert_eq!(1, tab.users.len());
        assert_eq!(1, tab.users.get(0).unwrap().id);
    }

    #[test]
    fn remove_user() {
        let mut tab = create_test_tab();

        let user1 = create_test_user(1, "User 1");
        tab.users.push(user1.clone());

        let action = RemoveUser::new(create_test_user(0, "User 0"), user1.id);

        assert_eq!(1, tab.users.len());

        action.perform(&mut tab).unwrap();

        assert_eq!(0, tab.users.len());
    }

    #[test]
    fn add_expense() {
        let mut tab = create_test_tab();

        let user0 = create_test_user(0, "User 0");
        let user1 = create_test_user(1, "User 1");

        tab.users.push(user0.clone());
        tab.users.push(user1.clone());

        let expense = create_test_expense(
            0,
            create_test_account("Test Account"),
            user0.clone(),
            vec![user0.clone(), user1.clone()],
        );

        let action = AddExpense::new(user0.clone(), expense);

        assert_eq!(0, tab.expenses.len());
        action.perform(&mut tab);
        assert_eq!(1, tab.expenses.len());
        assert_eq!(0, tab.expenses.get(0).unwrap().id);
    }

    #[test]
    fn remove_expense() {
        let mut tab = create_test_tab();

        let user0 = create_test_user(0, "User 0");
        let user1 = create_test_user(1, "User 1");

        let expense = create_test_expense(
            0,
            create_test_account("Test Account"),
            user0.clone(),
            vec![user0.clone(), user1.clone()],
        );

        let action = RemoveExpense::new(user0.clone(), expense.id);

        tab.expenses.push(expense);

        assert_eq!(1, tab.expenses.len());
        action.perform(&mut tab);
        assert_eq!(0, tab.expenses.len());
    }

    #[test]
    fn change_tab_name() {
        let mut tab = create_test_tab();
        let user0 = create_test_user(0, "User 0");

        let action = ChangeTabName::new(user0, "New Name");

        assert_eq!("Test Tab", tab.name);
        action.perform(&mut tab);
        assert_eq!("New Name", tab.name);
    }
}
