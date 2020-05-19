use std::{rc::Rc, marker::PhantomData, fmt::Debug};
use yew_router_min::Switch;

pub trait SwitchRoute: ToString {
    fn is_invalid(&self) -> bool;
}

pub struct Callback<IN>(Rc<dyn Fn(IN)>);

impl <IN> Debug for Callback<IN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Callback({:p})", self.0)
    }
}

impl <IN> PartialEq for Callback<IN> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl <IN> Clone for Callback<IN> {
    fn clone(&self) -> Self {
        Callback(Rc::clone(&self.0))
    }
}

#[derive(Debug)]
pub struct SwitchRouteService<SR> {
    callbacks: Vec<Callback<SR>>,
    switch_route_type: PhantomData<SR>,
}

impl<SR> PartialEq for SwitchRouteService<SR>
where
    SR: SwitchRoute + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.get_route_raw() == other.get_route_raw()
    }
}

impl<SR> SwitchRouteService<SR>
where
    SR: SwitchRoute + 'static,
{
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
            switch_route_type: PhantomData::default(),
        }
    }
    pub fn set_route<SRI: Into<SR>>(&mut self, switch_route: SRI) {
        let route = switch_route.into();
        self.service.set_route(&route.to_string(), ());
        self.notify_callbacks(route);
    }

    pub fn replace_route(&mut self, switch_route: SR) -> SR {
        let return_route = self.service.get_route().into();
        self.service.replace_route(&switch_route.to_string(), ());
        self.notify_callbacks(switch_route);
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
