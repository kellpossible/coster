use yew_router::{route::Route, service::RouteService, Switch};
use std::marker::PhantomData;
use yew::Callback;

pub trait SwitchRoute: Switch + Clone {
    fn to_string(&self) -> String;

    fn to_route(&self) -> Route {
        Route {
            route: self.to_string(),
            state: ()
        }
    }
}

#[derive(Debug)]
pub struct SwitchRouteService<SR> {
    callbacks: Vec<Callback<Option<SR>>>,
    service: RouteService<()>,
    sr: PhantomData<SR>,
}

impl <SR> PartialEq for SwitchRouteService<SR> where SR: SwitchRoute + 'static {
    fn eq(&self, other: &Self) -> bool {
        self.get_route_raw() == other.get_route_raw()
    }
    
}

impl <SR> SwitchRouteService<SR> where SR: SwitchRoute + 'static {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
            service: RouteService::new(),
            sr: PhantomData::default(),
        }
    }
    pub fn set_route(&mut self, switch_route: SR) {
        self.service.set_route(&switch_route.to_string(), ());
        self.notify_callbacks(Some(switch_route));
    }

    pub fn replace_route(&mut self, switch_route: SR) -> Option<SR> {
        let return_route = SR::switch(self.service.get_route());
        self.service.replace_route(&switch_route.to_string(), ());
        self.notify_callbacks(Some(switch_route));
        return_route
    }

    pub fn get_route(&self) -> Option<SR> {
        SR::switch(self.get_route_raw())
    }

    fn notify_callbacks(&self, switch_route: Option<SR>) {
        for callback in &self.callbacks {
            callback.emit(switch_route.clone());
        }
    }

    pub fn register_callback(&mut self, callback: Callback<Option<SR>>) {
        self.callbacks.push(callback.clone());
        self.service.register_callback(Callback::from(move |route| {
            callback.emit(SR::switch(route))
        }))
    }

    pub fn get_route_raw(&self) -> Route {
        self.service.get_route()
    }
}