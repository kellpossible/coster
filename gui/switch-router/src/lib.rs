use gloo_events::EventListener;
use std::{cell::RefCell, fmt::Debug, marker::PhantomData, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::{History, Location};

pub trait SwitchRoute: Clone + PartialEq {
    fn is_invalid(&self) -> bool;
    fn path(&self) -> String;
    fn switch(route: &str) -> Self;
}

pub struct Callback<SR>(Rc<dyn Fn(SR)>);

impl<SR> Callback<SR> {
    pub fn new<F: Fn(SR) + 'static>(f: F) -> Self {
        Self(Rc::new(f))
    }
    pub fn emit(&self, args: SR) {
        self.0(args)
    }
}

impl<SR> Debug for Callback<SR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Callback({:p})", self.0)
    }
}

impl<SR> PartialEq for Callback<SR> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<SR> Clone for Callback<SR> {
    fn clone(&self) -> Self {
        Callback(Rc::clone(&self.0))
    }
}

impl<SR, F> From<F> for Callback<SR>
where
    F: Fn(SR) + 'static,
{
    fn from(f: F) -> Self {
        Callback(Rc::new(f))
    }
}

type CallbackVec<SR> = Rc<RefCell<Vec<Callback<SR>>>>;

#[derive(Debug)]
pub struct SwitchRouteService<SR> {
    history: History,
    location: Location,
    // TODO: change this to use weak references for callback listeners. #23
    callbacks: CallbackVec<SR>,
    event_listener: EventListener,
    switch_route_type: PhantomData<SR>,
}

impl<SR> PartialEq for SwitchRouteService<SR>
where
    SR: SwitchRoute + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.get_route() == other.get_route()
    }
}

impl<SR> SwitchRouteService<SR>
where
    SR: SwitchRoute + 'static,
{
    pub fn new() -> Self {
        let window = web_sys::window().expect("browser does not have a window");

        let history = window
            .history()
            .expect("browser does not support the history API");

        let location = window.location();

        let callbacks = Rc::new(RefCell::new(Vec::new()));
        let listener_callbacks = callbacks.clone();

        let event_listener = EventListener::new(&window, "popstate", move |_event| {
            let location = web_sys::window()
                .expect("browser does not have a window")
                .location();
            let route = Self::route_from_location(&location);
            Self::notify_callbacks(&listener_callbacks, route);
        });
        Self {
            history,
            location,
            callbacks,
            event_listener,
            switch_route_type: PhantomData::default(),
        }
    }

    pub fn set_route<SRI: Into<SR>>(&mut self, switch_route: SRI) {
        let route = switch_route.into();
        //TODO: replace null with actual state storage
        self.history
            .push_state_with_url(&JsValue::null(), "", Some(&route.path()))
            .unwrap();
        Self::notify_callbacks(&self.callbacks, route);
    }

    pub fn replace_route<SRI: Into<SR>>(&mut self, switch_route: SRI) -> SR {
        let route = switch_route.into();
        let return_route = self.get_route();
        //TODO: replace null with actual state storage
        self.history
            .replace_state_with_url(&JsValue::null(), "", Some(&route.path()))
            .unwrap();
        Self::notify_callbacks(&self.callbacks, route);
        return_route
    }

    fn route_from_location(location: &Location) -> SR {
        let route = format!(
            "{pathname}{search}{hash}",
            pathname = location.pathname().unwrap(),
            search = location.search().unwrap(),
            hash = location.hash().unwrap()
        );

        SR::switch(&route)
    }

    pub fn get_route(&self) -> SR {
        Self::route_from_location(&self.location)
    }

    fn notify_callbacks(callbacks: &CallbackVec<SR>, switch_route: SR) {
        for callback in RefCell::borrow(&*callbacks).iter() {
            callback.emit(switch_route.clone());
        }
    }

    pub fn register_callback<CB: Into<Callback<SR>>>(&mut self, callback: CB) {
        self.callbacks.borrow_mut().push(callback.into());
    }

    pub fn deregister_callback(&mut self, callback: &Callback<SR>) -> Option<Callback<SR>> {
        let remove_position = match self.callbacks.borrow().iter().position(|c| c == callback) {
            Some(position) => Some(position),
            None => None,
        };

        if let Some(position) = remove_position {
            Some(self.callbacks.borrow_mut().remove(position))
        } else {
            None
        }
    }
}

impl<SR> From<yew::Callback<SR>> for Callback<SR>
where
    SR: 'static,
{
    fn from(yew_callback: yew::Callback<SR>) -> Self {
        Self::from(move |route| yew_callback.emit(route))
    }
}
