use super::form::Form;
use crate::validation::{ValidationError, ValidationErrors};
use std::rc::Rc;
use yew::{html::Scope, Component, ComponentLink};

pub trait FieldKey {
    fn field_label(&self) -> String;
}

impl FieldKey for &str {
    fn field_label(&self) -> String {
        self.to_string()
    }
}

pub trait FormFieldMessage {}
pub trait FormFieldProperties {}

pub trait FormField<Value, Key> {
    fn add_on_change_listener(&mut self, listener: Rc<dyn Fn(&Value)>);
    fn value(&self) -> &Value;
    fn validation_errors(&self) -> &ValidationErrors<Key>;
    fn key(&self) -> Key;
}
