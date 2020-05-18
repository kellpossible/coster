use yew_state::middleware::Middleware;
use unic_langid::LanguageIdentifier;

pub struct LocalizeMiddleware<L> {
    localizer: L
}

pub trait LocalizeEvent {
    fn language_changed() -> Self;
}

pub trait LocalizeAction {
    fn change_language(language: LanguageIdentifier) -> Self;
}

pub trait LocalizeState {
    
}

impl <L, State, Action, Event> Middleware<State, Action, Event> for LocalizeMiddleware<L> {
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