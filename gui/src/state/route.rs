use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use switch_router::SwitchRoute;
use yew_router::{route::Route, Switch};

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
