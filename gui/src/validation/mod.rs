use std::{
    error::Error,
    fmt::{Debug, Display}, rc::Rc,
};

pub struct ValidationError<Key> {
    key: Key,
    message: Rc<dyn Fn(&Key) -> String>,
}

impl<Key> Clone for ValidationError<Key>
where Key: Clone
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
            message: Rc::new(|key| format!("Validation error")),
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

#[derive(Debug)]
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

    pub fn get(&self, key: Key) -> Option<ValidationErrors<Key>> {
        let errors: Vec<ValidationError<Key>> = self
            .errors
            .iter()
            .filter(|error| error.key == key)
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

pub type ValidatorFn<Value, Key> = Box<dyn Fn(&Value, &Key) -> Result<(), ValidationError<Key>>>;

pub struct Validated<Value, Key = &'static str> {
    pub value: Value,
    pub key: Key,
    pub validators: Vec<ValidatorFn<Value, Key>>,
}

impl<Value, Key> Validated<Value, Key> {
    pub fn new(value: Value, key: Key) -> Self {
        Self {
            value,
            key,
            validators: Vec::new(),
        }
    }

    pub fn validator<F: Fn(&Value, &Key) -> Result<(), ValidationError<Key>> + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.validators.push(Box::new(f));
        self
    }
}

pub trait Validatable<Key> {
    fn validate(&self) -> Result<(), ValidationErrors<Key>>;
}

impl<Value, Key> Validatable<Key> for Validated<Value, Key>
where
    Key: PartialEq + Clone,
{
    fn validate(&self) -> Result<(), ValidationErrors<Key>> {
        let errors: Vec<ValidationError<Key>> = self
            .validators
            .iter()
            .filter_map(|validator: &ValidatorFn<Value, Key>| (validator)(&self.value, &self.key).err())
            .collect();

        if errors.len() > 0 {
            Err(ValidationErrors::new(errors))
        } else {
            Ok(())
        }
    }
}
