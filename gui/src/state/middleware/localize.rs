use i18n_embed::LanguageRequester;
use log::debug;
use serde::Serialize;
use std::{cell::RefCell, fmt::Display, hash::Hash, rc::Rc};
use unic_langid::LanguageIdentifier;
use yew::{Component, ComponentLink};
use reactive_state::{
    middleware::{Middleware, ReduceMiddlewareResult},
    Callback, Store, StoreEvent,
};

pub struct LocalizeMiddleware<LR> {
    pub language_requester: Rc<RefCell<LR>>,
}

impl<'a, LR> LocalizeMiddleware<LR>
where
    LR: LanguageRequester<'a>,
{
    pub fn new(language_requester: Rc<RefCell<LR>>) -> Self {
        Self { language_requester }
    }
}

impl<'a, LR, State, Action, Event, Effect> Middleware<State, Action, Event, Effect>
    for LocalizeMiddleware<LR>
where
    LR: LanguageRequester<'a>,
    Action: LocalizeAction,
{
    fn on_reduce(
        &self,
        store: &Store<State, Action, Event, Effect>,
        action: Option<&Action>,
        reduce: reactive_state::middleware::ReduceFn<State, Action, Event, Effect>,
    ) -> ReduceMiddlewareResult<Event, Effect> {
        if let Some(action) = action {
            if let Some(action) = action.get_change_selected_language() {
                let selected_language = action.selected_language.clone();
                debug!(
                    "LocalizeMiddleware::on_reduce Processing selected language: {:?}",
                    &selected_language
                );
                self.language_requester
                    .borrow_mut()
                    .set_language_override(selected_language.map(|l| l.clone()))
                    .unwrap();

                self.language_requester.borrow_mut().poll().unwrap();
            }
        }
        reduce(store, action)
    }
}

pub trait LocalizeEvent {
    fn language_changed() -> Self;
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct ChangeSelectedLanguage {
    pub selected_language: Option<LanguageIdentifier>,
    pub write_to_database: bool,
}

impl Display for ChangeSelectedLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let language_display = match &self.selected_language {
            Some(language) => language.to_string(),
            None => "None".to_string(),
        };
        write!(
            f,
            "ChangeSelectedLanguage({}, write: {:?})",
            language_display, self.write_to_database
        )
    }
}

pub trait LocalizeAction {
    fn change_selected_language(action: ChangeSelectedLanguage) -> Self;
    fn get_change_selected_language(&self) -> Option<&ChangeSelectedLanguage>;
}

pub trait LocalizeState {
    fn get_selected_language(&self) -> &Option<LanguageIdentifier>;
}

pub trait LocalizeStore<State, Event> {
    fn change_selected_language(
        &self,
        selected_language: Option<LanguageIdentifier>,
        write_to_database: bool,
    );
    fn subscribe_language_changed<COMP: Component>(
        &self,
        link: &ComponentLink<COMP>,
        message: COMP::Message,
    ) -> Callback<State, Event>
    where
        COMP::Message: Clone;
}

impl<State, Action, Event, Effect> LocalizeStore<State, Event>
    for Store<State, Action, Event, Effect>
where
    Action: LocalizeAction,
    State: LocalizeState + 'static,
    Event: LocalizeEvent + PartialEq + StoreEvent + Clone + Hash + Eq + 'static,
{
    fn change_selected_language(
        &self,
        selected_language: Option<LanguageIdentifier>,
        write_to_database: bool,
    ) {
        self.dispatch(Action::change_selected_language(ChangeSelectedLanguage {
            selected_language,
            write_to_database,
        }))
    }

    fn subscribe_language_changed<COMP: Component>(
        &self,
        link: &ComponentLink<COMP>,
        message: COMP::Message,
    ) -> Callback<State, Event>
    where
        COMP::Message: Clone,
    {
        let callback = link.callback(move |()| message.clone()).into();
        self.subscribe_event(&callback, LocalizeEvent::language_changed());
        callback
    }
}
