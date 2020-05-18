pub mod middleware;

use crate::routing::SwitchRoute;
use middleware::route::{RouteAction, RouteEvent, RouteState};
use std::fmt::Debug;
use unic_langid::LanguageIdentifier;
use yew_router::{route::Route, Switch};
use yew_state::{Reducer, StoreEvent, StoreRef};

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

impl ToString for AppRoute {
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

#[derive(Clone, Debug)]
pub enum RouteType {
    Valid(AppRoute),
    Invalid(String),
}

impl SwitchRoute<()> for RouteType {
    fn to_string(&self) -> String {
        match self {
            RouteType::Valid(app_route) => app_route.to_string(),
            RouteType::Invalid(route) => route.clone(),
        }
    }

    fn is_invalid(&self) -> bool {
        match self {
            RouteType::Invalid(_) => true,
            _ => false,
        }
    }
}

impl From<AppRoute> for RouteType {
    fn from(route: AppRoute) -> Self {
        RouteType::Valid(route)
    }
}

impl<STATE> From<Route<STATE>> for RouteType
where
    STATE: Clone,
{
    fn from(route: Route<STATE>) -> Self {
        match AppRoute::switch(Route::<STATE>::clone(&route)) {
            Some(app_route) => RouteType::Valid(app_route),
            None => RouteType::Invalid(route.to_string()),
        }
    }
}

impl Debug for AppRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Route({})", self.to_string())
    }
}

pub type StateStoreRef = StoreRef<CosterState, CosterAction, StateStoreEvent>;

pub struct CosterState {
    pub language: LanguageIdentifier,
    pub route: RouteType,
}

impl Default for CosterState {
    fn default() -> Self {
        Self {
            language: "en".parse().unwrap(),
            route: RouteType::Valid(AppRoute::Index),
        }
    }
}

impl CosterState {
    pub fn change_route(&self, route: RouteType) -> Self {
        Self {
            language: self.language.clone(),
            route,
        }
    }

    pub fn change_language(&self, language: LanguageIdentifier) -> Self {
        Self {
            language,
            route: self.route.clone(),
        }
    }
}

impl RouteState<RouteType> for CosterState {
    fn get_route(&self) -> &RouteType {
        &self.route
    }
}

pub enum CosterAction {
    ChangeLanguage(LanguageIdentifier),
    ChangeRoute(RouteType),
}

impl RouteAction<RouteType> for CosterAction {
    fn change_route<R: Into<RouteType>>(route: R) -> Self {
        CosterAction::ChangeRoute(route.into())
    }
}

pub struct CosterReducer;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum StateStoreEvent {
    LanguageChanged,
    RouteChanged,
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

impl RouteEvent<RouteType> for StateStoreEvent {
    fn route_changed() -> Self {
        StateStoreEvent::RouteChanged
    }
}

impl Reducer<CosterState, CosterAction, StateStoreEvent> for CosterReducer {
    fn reduce(
        &self,
        state: &CosterState,
        action: CosterAction,
    ) -> (CosterState, Vec<StateStoreEvent>) {
        let mut events = Vec::new();

        let state = match action {
            CosterAction::ChangeLanguage(language) => {
                events.push(StateStoreEvent::LanguageChanged);
                state.change_language(language)
            }
            CosterAction::ChangeRoute(route) => {
                events.push(StateStoreEvent::RouteChanged);
                state.change_route(route)
            }
        };

        (state, events)
    }
}
