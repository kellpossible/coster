use crate::actions::UserAction;
use crate::error::CostingError;
use crate::expense::{Expense, ExpenseCategory};
use crate::settlement::Settlement;
use crate::user::{User, UserID};
use chrono::{Local, NaiveDate};
use commodity::{Commodity, CommodityType, CommodityTypeID};
use doublecount::{
    sum_account_states, Account, AccountID, AccountState, AccountStatus, AccountingError,
    ActionTypeValue, Program, ProgramState, Transaction, TransactionElement,
};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

pub type TabID = Uuid;

/// A collection of expenses, and users who are responsible
/// for/associated with those expenses.
#[derive(Debug)]
pub struct Tab {
    /// The id of this tab
    pub id: TabID,
    /// The name of this tab
    pub name: String,
    /// The working currency of this tab
    pub working_currency: Rc<CommodityType>,
    /// The users involved with this tab
    users: Vec<Rc<User>>,
    /// [Accounts](Account) associated with [Users](User).
    user_accounts: HashMap<UserID, Rc<Account>>,
    /// The expenses recorded on this tab
    pub expenses: Vec<Expense>,
    /// [Accounts](Account) associated with [ExpenseCategories](ExpenseCategory).
    expense_category_accounts: HashMap<ExpenseCategory, Rc<Account>>,
    /// Actions performed by the users of this tab
    pub user_actions: Vec<Box<dyn UserAction>>,
}

impl Tab {
    /// Construct a new [Tab](Tab).
    pub fn new<S: Into<String>>(
        id: TabID,
        name: S,
        working_currency: Rc<CommodityType>,
        users: Vec<Rc<User>>,
        expenses: Vec<Expense>,
    ) -> Tab {
        let mut user_accounts = HashMap::with_capacity(users.len());

        for user in &users {
            let account = Rc::from(Tab::new_account_for_user(&user, working_currency.id));

            match user_accounts.insert(user.id, account) {
                Some(_) => panic!("There are duplicate users with id {0}", user.id),
                None => {}
            }
        }

        let mut expense_category_accounts: HashMap<String, Rc<Account>> =
            HashMap::with_capacity(expenses.len());

        for expense in &expenses {
            if !expense_category_accounts.get(&expense.category).is_some() {
                let account = Rc::from(Tab::new_account_for_expense_category(
                    &expense,
                    working_currency.id,
                ));
                expense_category_accounts.insert(expense.category.clone(), account);
            }
        }

        Tab {
            id,
            name: name.into(),
            working_currency,
            users,
            user_accounts,
            expenses,
            expense_category_accounts,
            user_actions: vec![],
        }
    }

    fn new_account_for_user(user: &Rc<User>, working_currency: CommodityTypeID) -> Account {
        Account::new_with_id(
            Some(format!("User-{}-{}", user.id.to_string(), user.name)),
            working_currency,
            Some("Users".to_string()),
        )
    }

    fn new_account_for_expense_category(
        expense: &Expense,
        working_currency: CommodityTypeID,
    ) -> Account {
        Account::new_with_id(
            Some(expense.category.clone()),
            working_currency,
            Some("Expense".to_string()),
        )
    }

    pub fn user(&self, user_id: &UserID) -> Result<&Rc<User>, CostingError> {
        for u in self.users.iter() {
            if &u.id == user_id {
                return Ok(u);
            }
        }

        Err(CostingError::UserDoesNotExistOnTab(*user_id, self.id))
    }

    pub fn get_user_account(&self, user_id: &UserID) -> Result<&Rc<Account>, CostingError> {
        self.user_accounts
            .get(user_id)
            .ok_or_else(|| CostingError::UserAccountDoesNotExistOnTab(*user_id, self.id))
    }

    pub fn get_expense_category_account(
        &self,
        category: &ExpenseCategory,
    ) -> Result<&Rc<Account>, CostingError> {
        self.expense_category_accounts
            .get(category)
            .ok_or_else(|| CostingError::NoExpenseCategoryAccountOnTab(category.clone(), self.id))
    }

    pub fn remove_user(&mut self, user_id: &UserID) -> Result<(), CostingError> {
        for (i, u) in self.users.iter().enumerate() {
            if &u.id == user_id {
                self.users.remove(i);
                self.user_accounts.remove(user_id);
                return Ok(());
            }
        }

        Err(CostingError::UserDoesNotExistOnTab(*user_id, self.id))
    }

    pub fn add_user(&mut self, user: User) -> Result<(), CostingError> {
        match self.users.iter().find(|u| u.id == user.id) {
            Some(user) => Err(CostingError::UserAlreadyExistsOnTab(user.id, self.id)),
            None => {
                let u = Rc::from(user);
                self.users.push(u.clone());
                self.user_accounts.insert(
                    u.id,
                    Rc::new(Tab::new_account_for_user(&u, self.working_currency.id)),
                );
                Ok(())
            }
        }
    }

    pub fn users(&self) -> &Vec<Rc<User>> {
        &self.users
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
        let zero = Commodity::zero(self.working_currency.id);

        let mut actual_transactions: Vec<Rc<ActionTypeValue>> =
            Vec::with_capacity(self.expenses.len());
        let mut shared_transactions: Vec<Rc<ActionTypeValue>> =
            Vec::with_capacity(self.expenses.len());

        let mut accounts: HashMap<AccountID, Rc<Account>> = HashMap::new();

        for expense in &self.expenses {
            actual_transactions.push(Rc::new(expense.get_actual_transaction(self)?.into()));
            shared_transactions.push(Rc::new(expense.get_shared_transaction(self)?.into()));

            let account = self.get_expense_category_account(&expense.category)?;
            accounts.insert(account.id, account.clone());
        }

        let expense_accounts: Vec<Rc<Account>> = accounts.iter().map(|(_, v)| v.clone()).collect();

        let actual_program = Program::new(actual_transactions.clone());

        for user in &self.users {
            let account = self.get_user_account(&user.id)?;
            match accounts.insert(account.id, account.clone()) {
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
            sum_account_states(account_states_from, self.working_currency.id, None)?;
        assert!(from_sum_with_expenses.eq_approx(zero, Commodity::default_epsilon()));
        let to_sum_with_expenses =
            sum_account_states(account_states_to, self.working_currency.id, None)?;
        assert!(to_sum_with_expenses.eq_approx(zero, Commodity::default_epsilon()));

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
            sum_account_states(&account_differences, self.working_currency.id, None)?;

        assert!(differences_sum.eq_approx(zero, Commodity::default_epsilon()));

        let mut negative_differences: Vec<AccountState> =
            Vec::with_capacity(account_differences.len());
        let mut positive_differences: Vec<AccountState> =
            Vec::with_capacity(account_differences.len());

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

        // dbg!(&negative_differences);
        // dbg!(&positive_differences);

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

            // cache today's date
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
                            negative_difference_state.account.id,
                            positive_difference_state.account.id,
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
        balancing_transactions
            .iter()
            .for_each(|bt| actual_with_balancing_transactions.push(Rc::new(bt.clone().into())));

        // run a program which includes the actual transactions, plus
        // the proposed balancing transactions, in order to test that
        // the proposed transactions produce the desired result.
        let actual_balanced_program = Program::new(actual_with_balancing_transactions);
        let mut actual_balanced_transactions_states =
            ProgramState::new(&accounts_vec, AccountStatus::Open);
        actual_balanced_transactions_states.execute_program(&actual_balanced_program)?;

        let actual_balanced_states = &actual_balanced_transactions_states.account_states;

        let actual_balanced_sum =
            sum_account_states(&actual_balanced_states, self.working_currency.id, None)?;
        assert!(actual_balanced_sum.eq_approx(zero, Commodity::default_epsilon()));

        // dbg!(&account_states_to);
        // dbg!(&actual_balanced_states);

        assert_eq!(account_states_to.len(), actual_balanced_states.len());
        for (id, to_state) in account_states_to {
            let balanced_state = actual_balanced_states.get(id).unwrap();
            to_state.eq_approx(balanced_state, Commodity::default_epsilon());
        }

        let settlements: Vec<Settlement> = balancing_transactions
            .iter()
            .map(|transaction: &Transaction| {
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

                let sender = self
                    .get_user_with_account(&sender_element.account_id)
                    .unwrap();
                let receiver = self
                    .get_user_with_account(&receiver_element.account_id)
                    .unwrap();

                Settlement::new(sender.id, receiver.id, amount)
            })
            .collect();

        Ok(settlements)
    }

    fn get_user_with_account(&self, account_id: &AccountID) -> Result<Rc<User>, CostingError> {
        self.user_accounts
            .iter()
            .find(|(_, v)| v.id == *account_id)
            .map(|(k, _)| self.user(k).map(|u| u.clone()))
            .unwrap()
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
            .map_err(|e| AccountingError::Commodity(e))?;

        let difference_state = AccountState::new(
            to_state.account.clone(),
            difference_amount,
            AccountStatus::Open,
        );

        result.insert(from_id.clone(), difference_state);
    }

    Ok(result)
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
        negative_difference_state.account.id,
        positive_difference_state.account.id,
        negative_difference_state.amount.neg(),
        None,
    )];

    positive_difference_state.amount = positive_difference_state
        .amount
        .add(&negative_difference_state.amount)?;
    negative_difference_state.amount = *zero;

    return Ok(transactions);
}
