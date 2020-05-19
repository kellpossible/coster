use crate::routing::{SwitchRoute, SwitchRouteService};
use std::{hash::Hash, marker::PhantomData, rc::Rc, fmt::Debug, cell::RefCell};
use yew_state::{middleware::{ReduceFn, Middleware}, Store, StoreEvent, StoreRef};
use log::{debug, error};

pub struct RouteMiddleware<SR, State, Action, Event> {
    pub router: RefCell<SwitchRouteService<SR>>,
    callback: yew::Callback<SR>,
    state_type: PhantomData<State>,
    action_type: PhantomData<Action>,
    event_type: PhantomData<Event>,
}

impl<SR, State, Action, Event> RouteMiddleware<SR, State, Action, Event>
where
    SR: SwitchRoute + 'static,
    State: 'static,
    Action: RouteAction<SR> + 'static,
    Event: StoreEvent + Clone + Hash + Eq + 'static,
{
    pub fn new(store: &StoreRef<State, Action, Event>) -> Self {
        let router = RefCell::new(SwitchRouteService::new());
        let store_rc = store.clone();
        let callback: yew::Callback<SR> = yew::Callback::Callback(Rc::new(move |route: SR| {
            debug!("state::middleware::route::callback callback invoked for route: {}", route.to_string());
            if let Err(err) = store_rc.dispatch(Action::browser_change_route(route)) {
                error!("Unable to dispatch RouteAction::browser_change_route Action to Store: {}", err);
            };
        }));

        // TODO: this might cause errors if the callback is called from another thread...
        // TODO: there is multiple borrow error with this callback
        match router.try_borrow_mut() {
            Ok(mut router_mut) => {
                router_mut.register_callback(callback.clone());
            }
            Err(err) => {
                error!("Unable to register callback {:?}: {}", callback, err);
            }
        }

        Self {
            router,
            callback,
            state_type: PhantomData,
            action_type: PhantomData,
            event_type: PhantomData,
        }
    }
}

impl<SR, State, Action, Event> Middleware<State, Action, Event>
    for RouteMiddleware<SR, State, Action, Event>
where
    SR: SwitchRoute + 'static,
    Action: RouteAction<SR> + PartialEq + Debug,
    State: RouteState<SR>,
    Event: RouteEvent<SR> + PartialEq + StoreEvent + Clone + Hash + Eq,
{
    fn on_reduce(&self, store: &mut Store<State, Action, Event>, action: Option<Action>, reduce: ReduceFn<State, Action, Event>) -> Vec<Event> {
        debug!("state::middleware::route::on_reduce started with action {:?}", action);
        
        if let Some(action) = &action {
            if let Some(route) = action.get_change_route() {
                debug!("state::middleware::route::on_reduce setting route: {}", route.to_string());
                match self.router.try_borrow_mut() {
                    Ok(mut router_mut) => {
                        router_mut.set_route(route.clone())
                    }
                    Err(err) => {
                        error!("Cannot borrow mut self.router: {}", err);
                    }
                }
            } else if action == &Action::poll_browser_route() {
                match self.router.try_borrow_mut() {
                    Ok(router_mut) => {
                        let route = router_mut.get_route();
                        return reduce(store, Some(Action::browser_change_route(route)))
                    }
                    Err(err) => {
                        error!("Cannot borrow mut self.router: {}", err);
                    }
                }
            }
        }
        debug!("state::middleware::route::on_reduce finished");
        reduce(store, action)
    } 
}

pub trait RouteState<SR> {
    fn get_route(&self) -> &SR;
}

pub trait RouteEvent<SR>
where
    SR: SwitchRoute + 'static,
{
    fn route_changed() -> Self;
}

pub trait RouteAction<SR>
where
    SR: SwitchRoute + 'static,
{
    fn change_route<R: Into<SR>>(route: R) -> Self;
    fn browser_change_route(route: SR) -> Self;
    fn poll_browser_route() -> Self;
    fn get_change_route(&self) -> Option<&SR>;
    fn get_browser_change_route(&self) -> Option<&SR>;
}

pub trait RouteStoreRef<SR> {
    fn change_route<R: Into<SR>>(&self, route: R);
}

impl<SR, State, Action, Event> RouteStoreRef<SR> for StoreRef<State, Action, Event>
where
    SR: SwitchRoute + 'static,
    Action: RouteAction<SR>,
    State: RouteState<SR>,
    Event: RouteEvent<SR> + PartialEq + StoreEvent + Clone + Hash + Eq,
{
    fn change_route<R: Into<SR>>(&self, route: R) {
        if let Err(err) = self.dispatch(Action::change_route(route)) {
            error!("Unable to dispatch change route Action on RouteStoreRef: {}", err);
        }
    }
}

pub trait RouteStore<SR> {
    fn change_route<R: Into<SR>>(&mut self, route: R);
}

impl<SR, State, Action, Event> RouteStore<SR> for Store<State, Action, Event>
where
    SR: SwitchRoute + 'static,
    Action: RouteAction<SR>,
    State: RouteState<SR>,
    Event: RouteEvent<SR> + PartialEq + StoreEvent + Clone + Hash + Eq,
{
    fn change_route<R: Into<SR>>(&mut self, route: R) {
        self.dispatch(Action::change_route(route));
    }
}
