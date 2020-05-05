use std::{
    error::Error,
    fmt::{Debug, Display},
    rc::Rc,
};

pub struct ValidationError<Key> {
    key: Key,
    message: Rc<dyn Fn(&Key) -> String>,
}

impl<Key> Clone for ValidationError<Key>
where
    Key: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            message: self.message.clone(),
        }
    }
}

impl<Key> ValidationError<Key> {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            message: Rc::new(|_| format!("Validation error")),
        }
    }

    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        let message_string = message.into();
        self.message = Rc::new(move |_| message_string.clone());
        self
    }

    pub fn with_message<F: Fn(&Key) -> String + 'static>(mut self, message: F) -> Self {
        self.message = Rc::new(message);
        self
    }

    fn get_message(&self) -> String {
        (self.message)(&self.key)
    }
}

impl<Key> Display for ValidationError<Key> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_message())
    }
}

impl<Key> Debug for ValidationError<Key>
where
    Key: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ValidationError{{ key: {0:?}, message: {1} }}",
            self.key,
            self.get_message()
        )
    }
}

impl<Key> Error for ValidationError<Key> where Key: Debug {}

#[derive(Debug, Clone)]
pub struct ValidationErrors<Key> {
    pub errors: Vec<ValidationError<Key>>,
}

impl<Key> ValidationErrors<Key>
where
    Key: PartialEq + Clone,
{
    pub fn new(errors: Vec<ValidationError<Key>>) -> Self {
        Self { errors }
    }

    pub fn get(&self, key: &Key) -> Option<ValidationErrors<Key>> {
        let errors: Vec<ValidationError<Key>> = self
            .errors
            .iter()
            .filter(|error| &error.key == key)
            .map(|error| (*error).clone())
            .collect();

        if errors.len() > 0 {
            Some(ValidationErrors::new(errors))
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn extend(&mut self, errors: ValidationErrors<Key>) {
        self.errors.extend(errors.errors)
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl<Key> Default for ValidationErrors<Key> {
    fn default() -> Self {
        Self { errors: Vec::new() }
    }
}

impl<Key> Display for ValidationErrors<Key> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let errors: Vec<String> = self.errors.iter().map(|e| format!("{}", e)).collect();
        write!(f, "{}", errors.join(", "))
    }
}

pub type ValidatorFn<Value, Key> = dyn Fn(&Value, &Key) -> Result<(), ValidationError<Key>>;

pub trait Validatable<Key> {
    fn validate(&self) -> Result<(), ValidationErrors<Key>>;
    fn validate_or_empty(&self) -> ValidationErrors<Key> {
        match self.validate() {
            Ok(()) => ValidationErrors::default(),
            Err(errors) => errors,
        }
    }
}

pub trait Validation<Value, Key> {
    fn validate_value(&self, value: &Value, key: &Key) -> Result<(), ValidationErrors<Key>>;
}

impl<Value, Key> Validation<Value, Key> for dyn Fn(&Value, &Key) -> Result<(), ValidationError<Key>>
where
    Key: Clone + PartialEq,
{
    fn validate_value(&self, value: &Value, key: &Key) -> Result<(), ValidationErrors<Key>> {
        (self)(value, key).map_err(|err| ValidationErrors::new(vec![err]))
    }
}

#[derive(Clone)]
pub struct Validator<Value, Key> {
    pub validations: Vec<Rc<ValidatorFn<Value, Key>>>,
}

impl<Value, Key> PartialEq for Validator<Value, Key> {
    fn eq(&self, other: &Self) -> bool {
        if self.validations.len() == other.validations.len() {
            let mut all_validations_same = true;

            for (i, this_validation) in self.validations.iter().enumerate() {
                let other_validation = other.validations.get(i).unwrap();
                all_validations_same &= Rc::ptr_eq(this_validation, other_validation);
            }

            all_validations_same
        } else {
            false
        }
    }
}

impl<Value, Key> Debug for Validator<Value, Key> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let validation_addresses: Vec<String> = self
            .validations
            .iter()
            .map(|validation| format!("ValidationFn: {:p}", *validation))
            .collect();

        write!(f, "Validator{{{0}}}", validation_addresses.join(", "))
    }
}

impl<Value, Key> Validator<Value, Key> {
    pub fn new() -> Self {
        Self {
            validations: Vec::new(),
        }
    }

    pub fn validation<F: Fn(&Value, &Key) -> Result<(), ValidationError<Key>> + 'static>(
        mut self,
        function: F,
    ) -> Self {
        self.validations.push(Rc::new(function));
        self
    }
}

impl<Value, Key> Validation<Value, Key> for Validator<Value, Key>
where
    Key: PartialEq + Clone,
{
    fn validate_value(&self, value: &Value, key: &Key) -> Result<(), ValidationErrors<Key>> {
        let mut errors = ValidationErrors::default();

        for validation in &self.validations {
            if let Err(new_errors) = validation.validate_value(value, key) {
                errors.extend(new_errors)
            }
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

impl<Value, Key> Default for Validator<Value, Key> {
    fn default() -> Self {
        Validator::new()
    }
}
