use std::rc::{Rc, Weak};

pub type CallbackResult<Error> = Result<(), Error>;
pub type CallbackResults<Error> = Result<(), Vec<Error>>;

#[derive(Clone)]
pub struct Listener<State, Error>(Weak<dyn Fn(Rc<State>) -> CallbackResult<Error>>);

#[derive(Clone)]
pub struct Callback<State, Error>(Rc<dyn Fn(Rc<State>) -> CallbackResult<Error>>);

pub trait AsListener<State, Error> {
    fn as_listener(&self) -> Listener<State, Error>;
}

impl<State, Error> AsListener<State, Error> for &Callback<State, Error> {
    fn as_listener(&self) -> Listener<State, Error> {
        Listener(Rc::downgrade(&self.0))
    }
}

impl<State, Error> Callback<State, Error> {
    pub fn new<C: Fn(Rc<State>) -> CallbackResult<Error> + 'static>(closure: C) -> Self {
        Callback(Rc::new(closure))
    }
    pub fn emit(&self, state: Rc<State>) -> CallbackResult<Error> {
        (self.0)(state)
    }
}

impl<State, Error, C> From<C> for Callback<State, Error>
where
    C: Fn(Rc<State>) -> CallbackResult<Error> + 'static,
{
    fn from(closure: C) -> Self {
        Callback(Rc::new(closure))
    }
}

impl<State, Error> Listener<State, Error> {
    pub fn as_callback(&self) -> Option<Callback<State, Error>> {
        match self.0.upgrade() {
            Some(listener_rc) => Some(Callback(listener_rc)),
            None => None,
        }
    }
}

impl<State, Error> AsListener<State, Error> for Listener<State, Error> {
    fn as_listener(&self) -> Listener<State, Error> {
        Listener(self.0.clone())
    }
}

impl<State, Error> From<yew::Callback<Rc<State>>> for Callback<State, Error>
where
    State: 'static,
{
    fn from(yew_callback: yew::Callback<Rc<State>>) -> Self {
        Callback(Rc::new(move |state| {
            yew_callback.emit(state.clone());
            Ok(())
        }))
    }
}
