use super::{
    middleware::{
        db::DBTransactionSerde, db::DatabaseEffect, db::KeyValueDBSerde, localize::LocalizeStore,
        route::RouteAction,
    },
    CosterAction, CosterEffect, CosterEvent, CosterState, ChangeLastSelectedCurrency,
};
use std::rc::Rc;
use yew_state::{Reducer, ReducerResult};
use commodity::CommodityType;

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
            CosterAction::ChangeSelectedLanguage(action) => {
                events.push(CosterEvent::LanguageChanged);

                // TODO: There is a problem here if the database middleware hasn't been added yet (because it's added in an async),
                // this event may miss being fired. #18
                if action.write_to_database {
                    let effect_language = action.selected_language.clone();
                    let effect =
                        DatabaseEffect::new("write selected_language", move |_store, database| {
                            let mut transaction = database.transaction();
                            transaction.put_serialize(0, "selected_language", &effect_language);
                            database
                                .write(transaction)
                                .expect("there was a problem executing a database transaction");
                        });

                    effects.push(effect.into());
                }

                Rc::new(prev_state.change_selected_language(action.selected_language.clone()))
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
                let effect = DatabaseEffect::new("load database", move |store, database| {
                    log::debug!("DatabaseEffect load database");
                    let selected_language_option: Option<Option<unic_langid::LanguageIdentifier>> =
                        database
                            .get_deserialize(0, "selected_language")
                            .expect("unable to read from database");
                    if let Some(selected_language) = selected_language_option {
                        store.change_selected_language(selected_language, false);
                    }
                    let last_selected_currency_option: Option<Option<CommodityType>> =
                        database
                            .get_deserialize(0, "last_selected_currency")
                            .expect("unable to read from database");
                    if let Some(last_selected_currency) = last_selected_currency_option {
                        store.dispatch(ChangeLastSelectedCurrency {
                           last_selected_currency,
                           write_to_database: false, 
                        });
                    }
                });

                effects.push(effect.into());
                prev_state.clone()
            }
            CosterAction::ChangeLastSelectedCurrency(action) => {
                let last_selected_currency = &action.last_selected_currency;

                if action.write_to_database {
                    let effect_currency = last_selected_currency.clone();
                    let effect =
                        DatabaseEffect::new("write last_selected_currency", move |_store, database| {
                            let mut transaction = database.transaction();
                            transaction.put_serialize(0, "last_selected_currency", &effect_currency);
                            database
                                .write(transaction)
                                .expect("there was a problem executing a database transaction");
                        });

                    effects.push(effect.into());
                }

                events.push(CosterEvent::LastSelectedCurrencyChanged);
                Rc::new(prev_state.change_last_selected_currency(last_selected_currency.clone()))
            }
            CosterAction::AddTab(tab) => {
                let mut tabs = prev_state.tabs.clone();
                tabs.push(tab.clone());
                events.push(CosterEvent::TabsChanged);
                events.push(CosterEvent::TabChanged(tab.id));
                Rc::new(prev_state.change_tabs(tabs))
            }
        };

        ReducerResult {
            state,
            events,
            effects,
        }
    }
}
