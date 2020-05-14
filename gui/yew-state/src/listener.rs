use std::rc::{Rc, Weak};

pub type CallbackResult<Error> = Result<(), Error>;
pub type CallbackResults<Error> = Result<(), Vec<Error>>;

#[derive(Clone)]
pub struct Listener<State, Error, Event>(Weak<dyn Fn(Rc<State>, Event) -> CallbackResult<Error>>);

pub trait AsListener<State, Error, Event> {
    fn as_listener(&self) -> Listener<State, Error, Event>;
}

#[derive(Clone)]
pub struct Callback<State, Error, Event>(Rc<dyn Fn(Rc<State>, Event) -> CallbackResult<Error>>);

impl<State, Error, Event> AsListener<State, Error, Event> for &Callback<State, Error, Event> {
    fn as_listener(&self) -> Listener<State, Error, Event> {
        Listener(Rc::downgrade(&self.0))
    }
}

impl<State, Error, Event> Callback<State, Error, Event> {
    pub fn new<C: Fn(Rc<State>, Event) -> CallbackResult<Error> + 'static>(closure: C) -> Self {
        Callback(Rc::new(closure))
    }
    pub fn emit(&self, state: Rc<State>, event: Event) -> CallbackResult<Error> {
        (self.0)(state, event)
    }
}

impl<C, State, Error, Event> From<C> for Callback<State, Error, Event>
where
    C: Fn(Rc<State>, Event) -> CallbackResult<Error> + 'static,
{
    fn from(closure: C) -> Self {
        Callback(Rc::new(closure))
    }
}

impl<State, Error, Event> Listener<State, Error, Event> {
    pub fn as_callback(&self) -> Option<Callback<State, Error, Event>> {
        match self.0.upgrade() {
            Some(listener_rc) => Some(Callback(listener_rc)),
            None => None,
        }
    }
}

impl<State, Error, Event> AsListener<State, Error, Event> for Listener<State, Error, Event> {
    fn as_listener(&self) -> Listener<State, Error, Event> {
        Listener(self.0.clone())
    }
}

impl<State, Error, Event> From<yew::Callback<Rc<State>>> for Callback<State, Error, Event>
where
    State: 'static,
    Event: 'static,
{
    fn from(yew_callback: yew::Callback<Rc<State>>) -> Self {
        Callback(Rc::new(move |state, _| {
            yew_callback.emit(state);
            Ok(())
        }))
    }
}

impl<State, Error, Event> From<yew::Callback<(Rc<State>, Event)>> for Callback<State, Error, Event>
where
    State: 'static,
    Event: 'static,
{
    fn from(yew_callback: yew::Callback<(Rc<State>, Event)>) -> Self {
        Callback(Rc::new(move |state, event| {
            yew_callback.emit((state.clone(), event));
            Ok(())
        }))
    }
}

impl<State, Error, Event> From<yew::Callback<()>> for Callback<State, Error, Event>
where
    State: 'static,
    Event: 'static,
{
    fn from(yew_callback: yew::Callback<()>) -> Self {
        Callback(Rc::new(move |_, _| {
            yew_callback.emit(());
            Ok(())
        }))
    }
}
