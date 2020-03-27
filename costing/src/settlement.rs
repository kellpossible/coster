use crate::user::User;

use std::rc::Rc;
use commodity::Commodity;
use chrono::NaiveDate;
use doublecount::Transaction;

/// Represents the settlement of a debt that one user owes another.
#[derive(Debug)]
pub struct Settlement {
    /// The user who has a debt and needs to send the money.
    pub sender: Rc<User>,
    /// The user who is owed money.
    pub receiver: Rc<User>,
    /// The amount of money the `sender` needs to send to the `receiver`.
    pub amount: Commodity,
}

impl Settlement {
    /// Create a new [Settlement](Settlement).
    pub fn new(sender: Rc<User>, receiver: Rc<User>, amount: Commodity) -> Settlement {
        Settlement {
            sender,
            receiver,
            amount,
        }
    }

    pub fn to_transaction(&self, date: NaiveDate) -> Transaction {
        Transaction::new_simple(
            Some("Settlement"),
            date,
            self.sender.account.id,
            self.receiver.account.id,
            self.amount,
            None,
        )
    }
}