use crate::Store;

pub trait ActionMiddleware<State, Action, Error> {
    fn invoke(
        &mut self,
        store: &mut Store<State, Action, Error>,
        action: Option<Action>,
    ) -> Option<Action>;
}
