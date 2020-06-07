pub mod middleware;

use middleware::{
    localize::{LocalizeAction, LocalizeEvent, LocalizeState},
    route::{IsRouteAction, RouteAction, RouteEvent, RouteState},
};
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use std::{
    convert::TryInto,
    fmt::{Debug, Display},
    rc::Rc,
};
use switch_router::SwitchRoute;
use unic_langid::LanguageIdentifier;
use yew_router::{route::Route, Switch};
use yew_state::{Reducer, ReducerResult, StoreEvent, StoreRef};

pub type StateCallback = yew_state::Callback<CosterState, CosterEvent>;

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
            AppRoute::CostingTab => "CostingTab",
            AppRoute::NewCostingTab => "NewCostingTab",
            AppRoute::Help => "Help",
            AppRoute::About => "About",
            AppRoute::Index => "Index",
        };
        write!(f, "{}: \"{}\"", route_name, self.to_string())
    }
}

pub type StateStoreRef = StoreRef<CosterState, CosterAction, CosterEvent, CosterEffect>;

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
    RouteAction(RouteAction<RouteType>),
}

impl Display for CosterAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CosterAction::ChangeSelectedLanguage(selected_language) => {
                let language_display = match selected_language {
                    Some(language) => language.to_string(),
                    None => "None".to_string(),
                };
                write!(f, "ChangeSelectedLanguage({})", language_display)
            }
            CosterAction::RouteAction(route_action) => write!(f, "RouteAction::{}", route_action),
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

impl From<RouteAction<RouteType>> for CosterAction {
    fn from(route_action: RouteAction<RouteType>) -> Self {
        CosterAction::RouteAction(route_action)
    }
}

impl IsRouteAction<RouteType> for CosterAction {
    fn route_action(&self) -> Option<&RouteAction<RouteType>> {
        match self {
            CosterAction::RouteAction(route_action) => Some(route_action),
            _ => None,
        }
    }
}

pub struct CosterReducer;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
pub enum CosterEvent {
    LanguageChanged,
    RouteChanged,
    None,
}

impl StoreEvent for CosterEvent {
    fn none() -> Self {
        CosterEvent::None
    }
    fn is_none(&self) -> bool {
        self == &Self::none()
    }
}

impl LocalizeEvent for CosterEvent {
    fn language_changed() -> Self {
        CosterEvent::LanguageChanged
    }
}

impl RouteEvent<RouteType> for CosterEvent {
    fn route_changed() -> Self {
        CosterEvent::RouteChanged
    }
}

#[derive(Debug, Clone)]
pub enum CosterEffect {}

impl Reducer<CosterState, CosterAction, CosterEvent, CosterEffect> for CosterReducer {
    fn reduce(
        &self,
        prev_state: &Rc<CosterState>,
        action: &CosterAction,
    ) -> ReducerResult<CosterState, CosterEvent, CosterEffect> {
        let mut events = Vec::new();
        let effects = Vec::new();

        let state = match action {
            CosterAction::ChangeSelectedLanguage(language) => {
                events.push(CosterEvent::LanguageChanged);
                Rc::new(prev_state.change_selected_language(language.clone()))
            }
            CosterAction::RouteAction(route_action) => match route_action {
                RouteAction::ChangeRoute(route) => {
                    events.push(CosterEvent::RouteChanged);
                    Rc::new(prev_state.change_route(route.clone()))
                }
                RouteAction::BrowserChangeRoute(route) => {
                    events.push(CosterEvent::RouteChanged);
                    Rc::new(prev_state.change_route(route.clone()))
                }
                RouteAction::PollBrowserRoute => prev_state.clone(),
            },
        };

        ReducerResult {
            state,
            events,
            effects,
        }
    }
}
