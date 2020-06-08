use serde::Serialize;
use commodity::CommodityType;
use super::{RouteType, middleware::{route::{IsRouteAction, RouteAction}, localize::{LocalizeAction, ChangeSelectedLanguage}}};
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ChangeLastSelectedCurrency {
    pub last_selected_currency: Option<CommodityType>,
    pub write_to_database: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum CosterAction {
    /// Selected language, and whether or not to write the value to the database.
    ChangeSelectedLanguage(ChangeSelectedLanguage),
    RouteAction(RouteAction<RouteType>),
    LoadDatabase,
    ChangeLastSelectedCurrency(ChangeLastSelectedCurrency),
}

impl Display for CosterAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CosterAction::ChangeSelectedLanguage(action) => write!(f, "{}", action),
            CosterAction::RouteAction(route_action) => write!(f, "RouteAction::{}", route_action),
            CosterAction::LoadDatabase => write!(f, "LoadDatabase"),
            CosterAction::ChangeLastSelectedCurrency(action) => {
                let currency = &action.last_selected_currency;
                let currency_display = match currency {
                    Some(currency) => format!("{}", currency),
                    None => "None".to_string(),
                };
                write!(f, "ChangeLastSelectedCurrency({})", currency_display)
            }
        }
    }
}

impl LocalizeAction for CosterAction {
    fn change_selected_language(action: ChangeSelectedLanguage) -> Self {
        CosterAction::ChangeSelectedLanguage(action)
    }
    fn get_change_selected_language(&self) -> Option<&ChangeSelectedLanguage> {
        match self {
            CosterAction::ChangeSelectedLanguage(action) => Some(action),
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

impl From<ChangeLastSelectedCurrency> for CosterAction {
    fn from(action: ChangeLastSelectedCurrency) -> Self {
        CosterAction::ChangeLastSelectedCurrency(action)
    }
}