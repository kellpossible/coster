use super::{
    middleware::{
        localize::{ChangeSelectedLanguage, LocalizeAction},
        route::{IsRouteAction, RouteAction},
    },
    RouteType,
};
use commodity::CommodityType;
use costing::Tab;
use serde::Serialize;
use std::{fmt::Display, rc::Rc};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ChangeLastSelectedCurrency {
    pub last_selected_currency: Option<CommodityType>,
    pub write_to_database: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum CosterAction {
    /// Selected language, and whether or not to write the value to the database.
    ChangeSelectedLanguage(ChangeSelectedLanguage),
    RouteAction(RouteAction<RouteType>),
    LoadDatabase,
    ChangeLastSelectedCurrency(ChangeLastSelectedCurrency),
    CreateTab {
        tab: Rc<Tab>,
        write_to_database: bool,
    },
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
                write!(
                    f,
                    "ChangeLastSelectedCurrency({}, write: {:?})",
                    currency_display, action.write_to_database
                )
            }
            CosterAction::CreateTab {
                tab,
                write_to_database,
            } => write!(f, "CreateTab({}, write: {:?})", tab.id, write_to_database),
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
