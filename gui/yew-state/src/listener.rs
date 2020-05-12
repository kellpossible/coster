use std::rc::{Rc, Weak};

pub type CallbackResult<Error> = Result<(), Error>;
pub type CallbackResults<Error> = Result<(), Vec<Error>>;

#[derive(Clone)]
pub struct Listener<State, Error, Event>(Weak<dyn Fn(Rc<State>, Option<Event>) -> CallbackResult<Error>>);

pub trait AsListener<State, Error, Event> {
    fn as_listener(&self) -> Listener<State, Error, Event>;
}

#[derive(Clone)]
pub struct EventListener<State, Error, Event>(Weak<dyn Fn(Rc<State>, Event) -> CallbackResult<Error>>);

pub trait AsEventListener<State, Error, Event> {
    fn as_event_listener(&self) -> EventListener<State, Error, Event>;
}

#[derive(Clone)]
pub struct Callback<State, Error, Event>(Rc<dyn Fn(Rc<State>, Option<Event>) -> CallbackResult<Error>>);

#[derive(Clone)]
pub struct EventCallback<State, Error, Event>(Rc<dyn Fn(Rc<State>, Event) -> CallbackResult<Error>>);


impl<State, Error, Event> AsEventListener<State, Error, Event> for &EventCallback<State, Error, Event> {
    fn as_event_listener(&self) -> EventListener<State, Error, Event> {
        EventListener(Rc::downgrade(&self.0))
    }
}

impl<State, Error, Event> EventCallback<State, Error, Event> {
    pub fn new<C: Fn(Rc<State>, Event) -> CallbackResult<Error> + 'static>(closure: C) -> Self {
        EventCallback(Rc::new(closure))
    }
    pub fn emit(&self, state: Rc<State>, event: Event) -> CallbackResult<Error> {
        (self.0)(state, event)
    }
}

impl<C, State, Error, Event> From<C> for EventCallback<State, Error, Event>
where
    C: Fn(Rc<State>, Event) -> CallbackResult<Error> + 'static,
{
    fn from(closure: C) -> Self {
        EventCallback(Rc::new(closure))
    }
}

impl<State, Error, Event> EventListener<State, Error, Event> {
    pub fn as_callback(&self) -> Option<EventCallback<State, Error, Event>> {
        match self.0.upgrade() {
            Some(listener_rc) => Some(EventCallback(listener_rc)),
            None => None,
        }
    }
}

impl<State, Error, Event> AsEventListener<State, Error, Event> for EventListener<State, Error, Event> {
    fn as_listener(&self) -> EventListener<State, Error, Event> {
        EventListener(self.0.clone())
    }
}

impl<State, Error, Event> From<yew::Callback<Rc<State>>> for EventCallback<State, Error, Event>
where
    State: 'static,
    Event: 'static,
{
    fn from(yew_callback: yew::Callback<Rc<State>>) -> Self {
        EventCallback(Rc::new(move |state, _| {
            yew_callback.emit(state.clone());
            Ok(())
        }))
    }
}

impl<State, Error, Event> From<yew::Callback<Event>> for Callback<State, Error, Event>
where
    State: 'static,
    Event: 'static,
{
    fn from(yew_callback: yew::Callback<Event>) -> Self {
        Callback(Rc::new(move |_, event| {
            yew_callback.emit(event);
            Ok(())
        }))
    }
}

impl<State, Error, Event> From<yew::Callback<(Rc<State>, Event)>> for EventCallback<State, Error, Event>
where
    State: 'static,
    Event: 'static,
{
    fn from(yew_callback: yew::Callback<(Rc<State>, Event)>) -> Self {
        EventCallback(Rc::new(move |state, event| {
            yew_callback.emit((state.clone(), event));
            Ok(())
        }))
    }
}
