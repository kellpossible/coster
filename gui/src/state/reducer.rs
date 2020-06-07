use super::{CosterAction, CosterState, CosterEvent, CosterEffect, middleware::route::RouteAction};
use yew_state::{ReducerResult, Reducer};
use std::rc::Rc;

pub struct CosterReducer;

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
