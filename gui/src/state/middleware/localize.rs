use yew_state::{StoreEvent, middleware::Middleware, Store, StoreRef};
use unic_langid::LanguageIdentifier;
use i18n_embed::LanguageRequester;
use std::{cell::RefCell, rc::Rc, hash::Hash};

pub struct LocalizeMiddleware<LR> {
    pub language_requester: Rc<RefCell<LR>>
}

impl <'a, LR> LocalizeMiddleware<LR>
where LR: LanguageRequester<'a> 
{
    pub fn new(language_requester: Rc<RefCell<LR>>) -> Self {
        Self {
            language_requester
        }
    }
}

impl <LR, State, Action, Event> Middleware<State, Action, Event> for LocalizeMiddleware<LR> {
    fn on_notify(
        &mut self,
        store: &mut yew_state::Store<State, Action, Event>,
        _: Action,
        events: Vec<Event>,
        notify: yew_state::middleware::NotifyFn<State, Action, Event>,
    ) {
        for event in &events {

        }
        notify(store, events);
    }
}

pub trait LocalizeEvent {
    fn language_changed() -> Self;
}

pub trait LocalizeAction {
    fn change_selected_language(selected_language: Option<LanguageIdentifier>) -> Self;
}

pub trait LocalizeState {
    fn get_selected_language(&self) -> &Option<LanguageIdentifier>;
}

pub trait LocalizeStore {
    fn change_selected_language(&mut self, selected_language: Option<LanguageIdentifier>);
}

impl<State, Action, Event> LocalizeStore for Store<State, Action, Event>
where
    Action: LocalizeAction,
    State: LocalizeState,
    Event: LocalizeEvent + PartialEq + StoreEvent + Clone + Hash + Eq,
{
    fn change_selected_language(&mut self, selected_language: Option<LanguageIdentifier>) {
        self.dispatch(Action::change_selected_language(selected_language))
    }
}

pub trait LocalizeStoreRef {
    fn change_selected_language(&self, selected_language: Option<LanguageIdentifier>);
}

impl<State, Action, Event> LocalizeStoreRef for StoreRef<State, Action, Event>
where
    Action: LocalizeAction,
    State: LocalizeState,
    Event: LocalizeEvent + PartialEq + StoreEvent + Clone + Hash + Eq,
{
    fn change_selected_language(&self, selected_language: Option<LanguageIdentifier>) {
        self.dispatch(Action::change_selected_language(selected_language))
    }
}
