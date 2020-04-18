use commodity::CommodityType;
use std::rc::Rc;
use serde::{Serialize, Deserialize};

pub type UserID = i32;

/// Represents a person using this system, and to be associated with
/// [Expense](Expenses) in a [Tab](Tab).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// The id of this user
    pub id: UserID,
    /// The name of this user
    pub name: String,
    /// The email address for this user
    pub email: Option<String>,
}

impl User {
    pub fn new(id: UserID, name: &str, email: Option<&str>, currency: Rc<CommodityType>) -> User {
        User {
            id,
            name: String::from(name),
            email: email.map(|e| String::from(e)),
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &User) -> bool {
        self.id == other.id
    }
}
