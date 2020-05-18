use std::marker::PhantomData;
use yew::Callback;
use yew_router::{route::Route, service::RouteService};

pub trait SwitchRoute<STATE = ()>: Clone + From<Route<STATE>> {
    fn to_string(&self) -> String;
    fn to_route(&self) -> Route {
        Route {
            route: self.to_string(),
            state: (),
        }
    }
    fn is_invalid(&self) -> bool;
}

#[derive(Debug)]
pub struct SwitchRouteService<SR> {
    callbacks: Vec<Callback<SR>>,
    service: RouteService<()>,
    sr: PhantomData<SR>,
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
            service: RouteService::new(),
            sr: PhantomData::default(),
        }
    }
    pub fn set_route(&mut self, switch_route: SR) {
        self.service.set_route(&switch_route.to_string(), ());
        self.notify_callbacks(switch_route);
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
        self.service
            .register_callback(Callback::from(move |route: Route| {
                callback.emit(route.into())
            }))
    }

    pub fn get_route_raw(&self) -> Route {
        self.service.get_route()
    }
}
