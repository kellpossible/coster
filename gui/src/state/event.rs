use serde::Serialize;
use super::{middleware::{localize::LocalizeEvent, route::RouteEvent}, RouteType};
use yew_state::StoreEvent;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
pub enum CosterEvent {
    StateChanged,
    LanguageChanged,
    RouteChanged,
    LastSelectedCurrencyChanged,
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