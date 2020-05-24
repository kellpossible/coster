use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    rc::Rc,
};
use web_sys::{History, Location};
use yew_router_min::Switch;
use wasm_bindgen::JsValue;

pub trait SwitchRoute {
    fn is_invalid(&self) -> bool;
    fn path(&self) -> String;
}

pub struct Callback<SR>(Rc<dyn Fn(SR)>);

impl<SR> Callback<SR> {
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

#[derive(Debug)]
pub struct SwitchRouteService<SR> {
    history: History,
    location: Location,
    callbacks: Vec<Callback<SR>>,
    switch_route_type: PhantomData<SR>,
}

impl<SR> PartialEq for SwitchRouteService<SR>
where
    SR: SwitchRoute + Clone + PartialEq + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.get_route() == other.get_route()
    }
}

impl<SR> SwitchRouteService<SR>
where
    SR: SwitchRoute + Clone + PartialEq + 'static,
{
    pub fn new() -> Self {

        let window = web_sys::window()
        .expect("browser does not have a window");

        let history = window.history()
            .expect("browser does not support the history API");

        let location = window.location();

        Self {
            history,
            location,
            callbacks: Vec::new(),
            switch_route_type: PhantomData::default(),
        }
    }
    
    pub fn set_route<SRI: Into<SR>>(&mut self, switch_route: SRI) {
        let route = switch_route.into();
        //TODO: replace null with actual state storage
        self.history.push_state_with_url(&JsValue::null(), "", Some(&route.path()));
        self.notify_callbacks(route);
    }

    pub fn replace_route<SRI: Into<SR>>(&mut self, switch_route: SRI) -> SR {
        let route = switch_route.into();
        let return_route = self.get_route();
        //TODO: replace null with actual state storage
        self.history.replace_state_with_url(&JsValue::null(), "", Some(&route.path()));
        self.notify_callbacks(route);
        return_route
    }

    pub fn get_route(&self) -> SR {
        self.get_route_raw().into()
    }

    fn notify_callbacks(&self, switch_route: SR) {
        for callback in &self.callbacks {
            callback.emit(switch_route.clone());
        }
    }

    pub fn register_callback(&mut self, callback: Callback<SR>) {
        self.callbacks.push(callback.clone());
    }

    pub fn deregister_callback(&mut self, callback: &Callback<SR>) -> Option<Callback<SR>> {
        match self.callbacks.iter().position(|c| c == callback) {
            Some(position) => Some(self.callbacks.remove(position)),
            None => None,
        }
    }
}
