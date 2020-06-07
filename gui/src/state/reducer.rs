use super::{
    middleware::{db::DBTransactionSerde, db::DatabaseEffect, db::KeyValueDBSerde, route::RouteAction},
    CosterAction, CosterEffect, CosterEvent, CosterState,
};
use std::rc::Rc;
use yew_state::{Reducer, ReducerResult};

pub struct CosterReducer;

impl Reducer<CosterState, CosterAction, CosterEvent, CosterEffect> for CosterReducer {
    fn reduce(
        &self,
        prev_state: &Rc<CosterState>,
        action: &CosterAction,
    ) -> ReducerResult<CosterState, CosterEvent, CosterEffect> {
        let mut events = Vec::new();
        let mut effects = Vec::new();

        let state = match action {
            CosterAction::ChangeSelectedLanguage(language) => {
                events.push(CosterEvent::LanguageChanged);

                // TODO: There is a problem here if the database middleware hasn't been added yet (because it's added in an async),
                // this event may miss being fired.
                let effect_language = language.clone();
                let effect = DatabaseEffect::new(move |_store, database| {
                    log::debug!("DatabaseEffect write selected_language");
                    let mut transaction = database.transaction();
                    transaction.put_serialize(0, "selected_language", &effect_language);
                    database
                        .write(transaction)
                        .expect("there was a problem executing a database transaction");
                }, false);

                effects.push(effect.into());

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
            CosterAction::LoadDatabase => {
                let effect = DatabaseEffect::new(move |store, database| {
                    log::debug!("DatabaseEffect load database");
                    let selected_language_option: Option<Option<unic_langid::LanguageIdentifier>> = database.get_deserialize(0, "selected_language").expect("unable to read from database");
                    if let Some(selected_language) = selected_language_option {
                        store.dispatch(CosterAction::ChangeSelectedLanguage(selected_language));
                    }
                }, true);

                effects.push(effect.into());
                prev_state.clone()
            }
        };

        ReducerResult {
            state,
            events,
            effects,
        }
    }
}
