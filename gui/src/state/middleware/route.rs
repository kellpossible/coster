use log::error;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};
use switch_router::{SwitchRoute, SwitchRouteService};
use yew_state::{
    middleware::{Middleware, ReduceFn},
    Store, StoreEvent, StoreRef,
};

pub struct RouteMiddleware<SR, State, Action, Event, Effect> {
    pub router: RefCell<SwitchRouteService<SR>>,
    callback: switch_router::Callback<SR>,
    state_type: PhantomData<State>,
    action_type: PhantomData<Action>,
    event_type: PhantomData<Event>,
    effect_type: PhantomData<Effect>,
}

impl<SR, State, Action, Event, Effect> RouteMiddleware<SR, State, Action, Event, Effect>
where
    SR: SwitchRoute + 'static,
    State: 'static,
    Action: IsRouteAction<SR> + 'static,
    Event: Clone + Hash + Eq + StoreEvent + 'static,
    Effect: 'static,
{
    pub fn new(store: StoreRef<State, Action, Event, Effect>) -> Self {
        let router = RefCell::new(SwitchRouteService::new());
        let callback: switch_router::Callback<SR> =
            switch_router::Callback::new(move |route: SR| {
                store.dispatch(RouteAction::BrowserChangeRoute(route));
            });

        // FIXME: there is multiple borrow error with this callback
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
            effect_type: PhantomData,
        }
    }

    fn set_route_no_callback<SRI: Into<SR>>(&self, switch_route: SRI) {
        match self.router.try_borrow_mut() {
            Ok(mut router) => {
                router.deregister_callback(&self.callback);
                router.set_route(switch_route);
                router.register_callback(self.callback.clone());
            }
            Err(err) => {
                error!("Unable to set route with no callback: {}", err);
            }
        }
    }
}

impl<SR, State, Action, Event, Effect> Middleware<State, Action, Event, Effect>
    for RouteMiddleware<SR, State, Action, Event, Effect>
where
    SR: SwitchRoute + 'static,
    Action: IsRouteAction<SR> + PartialEq + Debug + 'static,
    State: RouteState<SR> + 'static,
    Event: RouteEvent<SR> + PartialEq + Clone + Hash + Eq + StoreEvent + 'static,
    Effect: 'static,
{
    fn on_reduce(
        &self,
        store: &Store<State, Action, Event, Effect>,
        action: Option<&Action>,
        reduce: ReduceFn<State, Action, Event, Effect>,
    ) -> yew_state::middleware::ReduceMiddlewareResult<Event, Effect> {
        if let Some(action) = &action {
            if let Some(route_action) = action.route_action() {
                match route_action {
                    RouteAction::ChangeRoute(route) => {
                        self.set_route_no_callback(route.clone());
                    }
                    RouteAction::PollBrowserRoute => match self.router.try_borrow_mut() {
                        Ok(router_mut) => {
                            let route = router_mut.get_route();
                            return reduce(
                                store,
                                Some(&RouteAction::BrowserChangeRoute(route).into()),
                            );
                        }
                        Err(err) => {
                            error!("Cannot borrow mut self.router: {}", err);
                        }
                    },
                    _ => {}
                }
            }
        }
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum RouteAction<SR> {
    ChangeRoute(SR),
    BrowserChangeRoute(SR),
    PollBrowserRoute,
}

impl<SR> Display for RouteAction<SR>
where
    SR: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteAction::ChangeRoute(route) => write!(f, "ChangeRoute({:?})", route),
            RouteAction::BrowserChangeRoute(route) => write!(f, "BrowserChangeRoute({:?})", route),
            RouteAction::PollBrowserRoute => write!(f, "PollBrowserRoute"),
        }
    }
}

pub trait IsRouteAction<SR>: Clone + From<RouteAction<SR>>
where
    SR: SwitchRoute + 'static,
{
    fn route_action(&self) -> Option<&RouteAction<SR>>;
}

pub trait RouteStore<SR> {
    fn change_route<R: Into<SR>>(&self, route: R);
}

impl<SR, State, Action, Event, Effect> RouteStore<SR> for Store<State, Action, Event, Effect>
where
    SR: SwitchRoute + 'static,
    Action: IsRouteAction<SR>,
    State: RouteState<SR>,
    Event: RouteEvent<SR> + PartialEq + StoreEvent + Clone + Hash + Eq,
{
    fn change_route<R: Into<SR>>(&self, route: R) {
        self.dispatch(RouteAction::ChangeRoute(route.into()));
    }
}
