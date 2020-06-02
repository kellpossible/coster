pub mod middleware;

use middleware::{
    localize::{LocalizeAction, LocalizeEvent, LocalizeState},
    route::{RouteAction, RouteEvent, RouteState},
};
use serde::{Serialize, Deserialize};
use serde_diff::SerdeDiff;
use std::{fmt::{Display, Debug}, rc::Rc};
use switch_router::SwitchRoute;
use unic_langid::LanguageIdentifier;
use yew_router::{route::Route, Switch};
use yew_state::{Reducer, StoreEvent, StoreRef};

pub type StateCallback = yew_state::Callback<CosterState, StateStoreEvent>;

#[derive(Switch, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RouteType {
    Valid(AppRoute),
    Invalid(String),
}

impl SwitchRoute for RouteType {
    fn path(&self) -> String {
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

    fn switch(route: &str) -> Self {
        match AppRoute::switch(Route::new_no_state(route)) {
            Some(app_route) => RouteType::Valid(app_route),
            None => RouteType::Invalid(route.to_string()),
        }
    }
}

impl From<AppRoute> for RouteType {
    fn from(route: AppRoute) -> Self {
        RouteType::Valid(route)
    }
}

impl Debug for AppRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let route_name = match self {
            AppRoute::CostingTab => {
                "CostingTab"
            }
            AppRoute::NewCostingTab => {
                "NewCostingTab"
            }
            AppRoute::Help => {
                "Help"
            }
            AppRoute::About => {
                "About"
            }
            AppRoute::Index => {
                "Index"
            }
        };
        write!(f, "{}: \"{}\"", route_name, self.to_string())
    }
}

pub type StateStoreRef = StoreRef<CosterState, CosterAction, StateStoreEvent>;

#[derive(Debug, Clone, SerdeDiff, Serialize, Deserialize)]
pub struct CosterState {
    #[serde_diff(opaque)]
    pub selected_language: Option<LanguageIdentifier>,
    #[serde_diff(opaque)]
    pub route: RouteType,
}

impl Default for CosterState {
    fn default() -> Self {
        Self {
            selected_language: None,
            route: RouteType::Valid(AppRoute::Index),
        }
    }
}

impl CosterState {
    pub fn change_route(&self, route: RouteType) -> Self {
        Self {
            selected_language: self.selected_language.clone(),
            route,
        }
    }

    pub fn change_selected_language(&self, selected_language: Option<LanguageIdentifier>) -> Self {
        Self {
            selected_language,
            route: self.route.clone(),
        }
    }
}

impl RouteState<RouteType> for CosterState {
    fn get_route(&self) -> &RouteType {
        &self.route
    }
}

impl LocalizeState for CosterState {
    fn get_selected_language(&self) -> &Option<LanguageIdentifier> {
        &self.selected_language
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum CosterAction {
    ChangeSelectedLanguage(Option<LanguageIdentifier>),
    ChangeRoute(RouteType),
    BrowserChangeRoute(RouteType),
    PollBrowserRoute,
}

impl Display for CosterAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CosterAction::ChangeSelectedLanguage(selected_language) => {
                let language_display = match selected_language {
                    Some(language) => {
                        language.to_string()
                    }
                    None => {
                        "None".to_string()
                    }
                };
                write!(f, "ChangeSelectedLanguage({})", language_display)
            }
            CosterAction::ChangeRoute(route) => {
                write!(f, "ChangeRoute({:?})", route)
            }
            CosterAction::BrowserChangeRoute(route) => {
                write!(f, "BrowserChangeRoute({:?})", route)
            }
            CosterAction::PollBrowserRoute => {
                write!(f, "PollBrowserRoute")
            }
        }
    }
    
}

impl LocalizeAction for CosterAction {
    fn change_selected_language(selected_language: Option<LanguageIdentifier>) -> Self {
        CosterAction::ChangeSelectedLanguage(selected_language)
    }
    fn get_change_selected_language(&self) -> Option<Option<&LanguageIdentifier>> {
        match self {
            CosterAction::ChangeSelectedLanguage(selected_language) => {
                Some(selected_language.as_ref())
            }
            _ => None,
        }
    }
}

impl RouteAction<RouteType> for CosterAction {
    fn change_route<R: Into<RouteType>>(route: R) -> Self {
        CosterAction::ChangeRoute(route.into())
    }
    fn browser_change_route(route: RouteType) -> Self {
        CosterAction::BrowserChangeRoute(route)
    }
    fn get_browser_change_route(&self) -> Option<&RouteType> {
        match self {
            CosterAction::BrowserChangeRoute(route) => Some(route),
            _ => None,
        }
    }
    fn get_change_route(&self) -> Option<&RouteType> {
        match self {
            CosterAction::ChangeRoute(route) => Some(route),
            _ => None,
        }
    }
    fn poll_browser_route() -> Self {
        CosterAction::PollBrowserRoute
    }
}

pub struct CosterReducer;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
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

impl LocalizeEvent for StateStoreEvent {
    fn language_changed() -> Self {
        StateStoreEvent::LanguageChanged
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
        prev_state: Rc<CosterState>,
        action: CosterAction,
    ) -> (Rc<CosterState>, Vec<StateStoreEvent>) {
        let mut events = Vec::new();

        let state = match action {
            CosterAction::ChangeSelectedLanguage(language) => {
                events.push(StateStoreEvent::LanguageChanged);
                Rc::new(prev_state.change_selected_language(language))
            }
            CosterAction::ChangeRoute(route) => {
                events.push(StateStoreEvent::RouteChanged);
                Rc::new(prev_state.change_route(route))
            }
            CosterAction::BrowserChangeRoute(route) => {
                events.push(StateStoreEvent::RouteChanged);
                Rc::new(prev_state.change_route(route))
            }
            CosterAction::PollBrowserRoute => prev_state.clone(),
        };

        (state, events)
    }
}
