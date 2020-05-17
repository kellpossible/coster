use std::{cell::RefCell, rc::Rc};
use unic_langid::LanguageIdentifier;
use yew_state::{Reducer, Store, StoreEvent, middleware::Middleware};
use yew_router::Switch;
use crate::routing::{SwitchRouteService, SwitchRoute};

pub type StateStore = Rc<RefCell<Store<CosterState, CosterAction, anyhow::Error, StateStoreEvent>>>;

#[derive(Switch, Clone, Debug, Hash, Eq, PartialEq)]
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

pub struct CosterState {
    current_language: LanguageIdentifier,
    route: AppRoute,
}

impl Default for CosterState {
    fn default() -> Self {
        Self {
            current_language: "en".parse().unwrap(),
            route: AppRoute::Index,
        }
    }
}

pub enum CosterAction {
    ChangeLanguage(LanguageIdentifier),
    ChangeRoute(AppRoute),
}

pub struct CosterReducer;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum StateStoreEvent {
    LanguageChanged,
    RouteChanged(AppRoute),
    None,
}

impl StoreEvent for StateStoreEvent {
    fn none() -> Self {
        StateStoreEvent::None
    }
    fn is_none(&self) -> bool {
        self == &Self::none()
    }
}

impl Reducer<CosterState, CosterAction, StateStoreEvent> for CosterReducer {
    fn reduce(
        &self,
        state: &CosterState,
        action: &CosterAction,
    ) -> (CosterState, Vec<StateStoreEvent>) {
        let mut events = Vec::new();

        let state = match action {
            CosterAction::ChangeLanguage(language) => {
                events.push(StateStoreEvent::LanguageChanged);
                CosterState {
                    current_language: language.clone(),
                    ..*state
                }
            }
            CosterAction::ChangeRoute(route) => {
                events.push(StateStoreEvent::RouteChanged(route.clone()));
                CosterState {
                    route: route.clone(),
                    ..*state
                }
            }
        };

        (state, events)
    }
}

struct RouteMiddleware {
    router: SwitchRouteService<AppRoute>,
}

impl RouteMiddleware {
    pub fn new(store: &StateStore) -> Self {
        let mut router =  SwitchRouteService::new();
        let store_rc = store.clone();
        let callback: yew::Callback<Option<AppRoute>> = yew::Callback::Callback(Rc::new(move |route: Option<AppRoute>| {
            if let Some(route) = route {
                store_rc.borrow_mut().dispatch(CosterAction::ChangeRoute(route));
            }
        }));

        router.register_callback(callback);

        Self {
            router,
        }
    }
}

impl Middleware<CosterState, CosterAction, anyhow::Error, StateStoreEvent> for RouteMiddleware {
    fn on_notify(
        &mut self,
        store: &mut Store<CosterState, CosterAction, anyhow::Error, StateStoreEvent>,
        action: CosterAction,
        events: Vec<StateStoreEvent>,
        notify: yew_state::middleware::NotifyFn<CosterState, CosterAction, anyhow::Error, StateStoreEvent>,
    ) -> yew_state::CallbackResults<anyhow::Error> {
        for event in events {
            match event {
                StateStoreEvent::RouteChanged(route) => {
                    self.router.set_route(route);
                }
                _ => {}
            }
        }
        notify(store, events)
    }
}
