use super::{CosterAction, CosterState, StateStoreEvent, StateStoreRef};
use crate::routing::{SwitchRoute, SwitchRouteService};
use std::rc::Rc;
use yew_router::Switch;
use yew_state::{middleware::Middleware, Store};

#[derive(Switch, Clone, Hash, Eq, PartialEq)]
pub enum AppRoute {
    /// Matches the `/tab` route.
    #[to = "/tab"]
    CostingTab,
    /// Matches the `/new` route.
    #[to = "/new"]
    NewCostingTab,
    /// Matches the `/help` route.
    #[to = "/help"]
    Help,
    /// Matches the `/about` route.
    #[to = "/about"]
    About,
    /// Matches the `/` route.
    #[to = "/"]
    Index, // Order is important here, the index needs to be last.
}

impl SwitchRoute for AppRoute {
    fn to_string(&self) -> String {
        match self {
            AppRoute::CostingTab => "/tab".to_string(),
            AppRoute::NewCostingTab => "/new".to_string(),
            AppRoute::Help => "/help".to_string(),
            AppRoute::About => "/about".to_string(),
            AppRoute::Index => "/".to_string(),
        }
    }
}

pub struct RouteMiddleware {
    router: SwitchRouteService<AppRoute>,
}

impl RouteMiddleware {
    pub fn new(store: &StateStoreRef) -> Self {
        let mut router = SwitchRouteService::new();
        let store_rc = store.clone();
        let callback: yew::Callback<Option<AppRoute>> =
            yew::Callback::Callback(Rc::new(move |route: Option<AppRoute>| {
                if let Some(route) = route {
                    store_rc.dispatch(CosterAction::ChangeRoute(route));
                }
            }));

        // TODO: this might cause errors if the callback is called from another thread...
        router.register_callback(callback);

        Self { router }
    }
}

impl Middleware<CosterState, CosterAction, StateStoreEvent> for RouteMiddleware {
    fn on_notify(
        &mut self,
        store: &mut Store<CosterState, CosterAction, StateStoreEvent>,
        _: CosterAction,
        events: Vec<StateStoreEvent>,
        notify: yew_state::middleware::NotifyFn<CosterState, CosterAction, StateStoreEvent>,
    ) {
        for event in &events {
            match event {
                StateStoreEvent::RouteChanged(route) => {
                    self.router.set_route(route.clone());
                }
                _ => {}
            }
        }
        notify(store, events);
    }
}
