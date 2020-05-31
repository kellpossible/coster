use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct Listener<State, Event>(Weak<dyn Fn(Rc<State>, Event)>);

pub trait AsListener<State, Event> {
    fn as_listener(&self) -> Listener<State, Event>;
}

#[derive(Clone)]
pub struct Callback<State, Event>(Rc<dyn Fn(Rc<State>, Event)>);

impl<State, Event> AsListener<State, Event> for &Callback<State, Event> {
    fn as_listener(&self) -> Listener<State, Event> {
        Listener(Rc::downgrade(&self.0))
    }
}

impl<State, Event> Callback<State, Event> {
    pub fn new<C: Fn(Rc<State>, Event) + 'static>(closure: C) -> Self {
        Callback(Rc::new(closure))
    }
    pub fn emit(&self, state: Rc<State>, event: Event) {
        (self.0)(state, event)
    }
}

impl<C, State, Event> From<C> for Callback<State, Event>
where
    C: Fn(Rc<State>, Event) + 'static,
{
    fn from(closure: C) -> Self {
        Callback(Rc::new(closure))
    }
}

impl<State, Event> Listener<State, Event> {
    pub fn as_callback(&self) -> Option<Callback<State, Event>> {
        match self.0.upgrade() {
            Some(listener_rc) => Some(Callback(listener_rc)),
            None => None,
        }
    }
}

impl<State, Event> AsListener<State, Event> for Listener<State, Event> {
    fn as_listener(&self) -> Listener<State, Event> {
        Listener(self.0.clone())
    }
}

// TODO: make make this optional based on `yew-compat` feature
impl<State, Event> From<yew::Callback<Rc<State>>> for Callback<State, Event>
where
    State: 'static,
    Event: 'static,
{
    fn from(yew_callback: yew::Callback<Rc<State>>) -> Self {
        Callback(Rc::new(move |state, _| {
            yew_callback.emit(state);
        }))
    }
}

// TODO: make make this optional based on `yew-compat` feature
impl<State, Event> From<yew::Callback<(Rc<State>, Event)>> for Callback<State, Event>
where
    State: 'static,
    Event: 'static,
{
    fn from(yew_callback: yew::Callback<(Rc<State>, Event)>) -> Self {
        Callback(Rc::new(move |state, event| {
            yew_callback.emit((state.clone(), event));
        }))
    }
}

// TODO: make make this optional based on `yew-compat` feature
impl<State, Event> From<yew::Callback<()>> for Callback<State, Event>
where
    State: 'static,
    Event: 'static,
{
    fn from(yew_callback: yew::Callback<()>) -> Self {
        Callback(Rc::new(move |_, _| {
            yew_callback.emit(());
        }))
    }
}
