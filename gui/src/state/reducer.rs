use super::{
    db::CosterClientDBStore,
    middleware::{db::DatabaseEffect, localize::LocalizeStore, route::RouteAction},
    ChangeLastSelectedCurrency, CosterAction, CosterEffect, CosterEvent, CosterState,
};
use commodity::CommodityType;
use costing::db::{DBTransactionSerde, DatabaseValue, KeyValueDBSerde, Ids};
use costing::{Tab, TabData, TabID, TabsID};
use std::rc::Rc;
use yew_state::{Reducer, ReducerResult, Store};

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
                            transaction.put_serialize(
                                &CosterClientDBStore::General,
                                "selected_language",
                                &effect_language,
                            );
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
            CosterAction::ChangeLastSelectedCurrency(action) => {
                let last_selected_currency = &action.last_selected_currency;

                if action.write_to_database {
                    let effect_currency = last_selected_currency.clone();
                    let effect = DatabaseEffect::new(
                        "write last_selected_currency",
                        move |_store, database| {
                            let mut transaction = database.transaction();
                            transaction.put_serialize(
                                &CosterClientDBStore::General,
                                "last_selected_currency",
                                &effect_currency,
                            );
                            database
                                .write(transaction)
                                .expect("there was a problem executing a database transaction");
                        },
                    );

                    effects.push(effect.into());
                }

                events.push(CosterEvent::LastSelectedCurrencyChanged);
                Rc::new(prev_state.change_last_selected_currency(last_selected_currency.clone()))
            }
            CosterAction::CreateTab {
                tab,
                write_to_database,
            } => {
                let mut tabs = prev_state.tabs.clone();
                tabs.push(tab.clone());
                events.push(CosterEvent::TabsChanged);

                if *write_to_database {
                    let effect_tab = tab.clone();
                    let effect = DatabaseEffect::new(
                        "write tabs, and add new tab",
                        move |store: &Store<
                            CosterState,
                            CosterAction,
                            CosterEvent,
                            CosterEffect,
                        >,
                              database| {
                            let mut transaction = database.transaction();
                            let tab_ids = store.state().tabs.ids();
                            let tab_key = format!("tabs/{}", effect_tab.id);

                            let tab_data = TabData::from_tab(&effect_tab);

                            // TODO: refactor tabs vector into something within `costing` library to be shared
                            // with the server.
                            transaction.put_serialize(&CosterClientDBStore::Tabs, "tabs", &tab_ids);
                            effect_tab.write_to_db(
                                Some("tabs"),
                                &mut transaction,
                                &CosterClientDBStore::Tabs,
                            );
                            database
                                .write(transaction)
                                .expect("there was a problem executing a database transaction");
                        },
                    );

                    effects.push(effect.into());
                }

                Rc::new(prev_state.change_tabs(tabs))
            }
            CosterAction::LoadDatabase => {
                let effect = DatabaseEffect::new("load database", move |store, database| {
                    log::debug!("DatabaseEffect load database");
                    let selected_language_option: Option<Option<unic_langid::LanguageIdentifier>> =
                        database
                            .get_deserialize(&CosterClientDBStore::General, "selected_language")
                            .expect("unable to read from database");
                    if let Some(selected_language) = selected_language_option {
                        store.change_selected_language(selected_language, false);
                    }
                    let last_selected_currency_option: Option<Option<CommodityType>> = database
                        .get_deserialize(&CosterClientDBStore::General, "last_selected_currency")
                        .expect("unable to read \"last_selected_currency\" from database");
                    if let Some(last_selected_currency) = last_selected_currency_option {
                        store.dispatch(ChangeLastSelectedCurrency {
                            last_selected_currency,
                            write_to_database: false,
                        });
                    }

                    // TODO: refactor tabs vector into something within `costing` library to be shared
                    // with the server.
                    let tab_ids_option: Option<Vec<TabID>> = database
                        .get_deserialize(&CosterClientDBStore::Tabs, "tabs")
                        .expect("unable to read \"tabs\" from database");

                    let tabs_option = Vec::<Rc<Tab>>::read_from_db(
                        TabsID,
                        None,
                        &database,
                        &CosterClientDBStore::Tabs,
                    );

                    if let Some(tabs) = tabs_option {
                        store.dispatch(CosterAction::LoadTabs {
                            tabs,
                            write_to_database: false,
                        });
                    }
                });

                effects.push(effect.into());
                prev_state.clone()
            }
            CosterAction::LoadTabs {
                tabs,
                write_to_database,
            } => {
                if *write_to_database {
                    let tabs_effect = tabs.clone();
                    let effect =
                        DatabaseEffect::new("write all tabs to database", move |store, database| {
                            let mut transaction = database.transaction();
                            tabs_effect.write_to_db(
                                None,
                                &mut transaction,
                                &CosterClientDBStore::Tabs,
                            );
                            database
                                .write(transaction)
                                .expect("unable to write tabs to database");
                        });

                    effects.push(effect.into());
                }

                events.push(CosterEvent::TabsChanged);
                Rc::new(prev_state.change_tabs(tabs.clone()))
            }
        };

        ReducerResult {
            state,
            events,
            effects,
        }
    }
}
