use chrono::{DateTime, Utc};
use std::rc::Rc;

struct User {
    id: String,
    name: String,
    email: Option<String>,
}

struct UserAction<T> {
    data: Rc<T>,
    datetime: DateTime<Utc>,
    user: Rc<User>,
}

struct Ownership<T> {
    user: Rc<User>,
    data: T,
}
