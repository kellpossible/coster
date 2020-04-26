use crate::error::CostingError;
use crate::tab::Tab;
use crate::user::UserID;

use chrono::NaiveDate;
use commodity::Commodity;
use doublecount::Transaction;
use serde::{Deserialize, Serialize};

/// Represents the settlement of a debt that one user owes another.
#[derive(Debug, Serialize, Deserialize)]
pub struct Settlement {
    /// The user who has a debt and needs to send the money.
    pub sender: UserID,
    /// The user who is owed money.
    pub receiver: UserID,
    /// The amount of money the `sender` needs to send to the `receiver`.
    pub amount: Commodity,
}

impl Settlement {
    /// Create a new [Settlement](Settlement).
    pub fn new(sender: UserID, receiver: UserID, amount: Commodity) -> Settlement {
        Settlement {
            sender,
            receiver,
            amount,
        }
    }

    pub fn to_transaction(&self, date: NaiveDate, tab: &Tab) -> Result<Transaction, CostingError> {
        Ok(Transaction::new_simple(
            Some("Settlement"),
            date,
            tab.get_user_account(&self.sender)?.id,
            tab.get_user_account(&self.receiver)?.id,
            self.amount,
            None,
        ))
    }
}
