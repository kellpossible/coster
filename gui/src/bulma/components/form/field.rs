use crate::validation::{ValidationErrors};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

pub trait FieldKey: Clone + PartialEq + Display + Hash + Eq + Debug {
    fn field_label(&self) -> String;
}

impl FieldKey for &str {
    fn field_label(&self) -> String {
        self.to_string()
    }
}

pub trait FieldLink<Key: Clone>: Debug {
    fn field_key(&self) -> &Key;
    fn send_message(&self, msg: FieldMsg);
}

pub trait FormField<Key> {
    fn validation_errors(&self) -> &ValidationErrors<Key>;
    fn field_key(&self) -> &Key;
}

#[derive(Copy, Clone)]
pub enum FieldMsg {
    Validate,
}
