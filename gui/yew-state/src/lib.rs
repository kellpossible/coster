mod middleware;
mod reducer;
mod store;

pub use reducer::Reducer;
use std::rc::{Rc, Weak};
pub use store::Store;

pub type CallbackResult<Error> = Result<(), Error>;
pub type CallbackResults<Error> = Result<(), Vec<Error>>;
pub type Listener<State, Error> = Weak<dyn Fn(Rc<State>) -> CallbackResult<Error>>;
pub type Callback<State, Error> = Rc<dyn Fn(Rc<State>) -> CallbackResult<Error>>;
