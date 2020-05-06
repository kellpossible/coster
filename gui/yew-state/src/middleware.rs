use crate::reducer;
use crate::Store;

pub trait Middleware<State, Action, Reducer, Error>
where
    Reducer: reducer::Reducer<State, Action>,
{
    fn invoke(store: &mut Store<State, Action, Reducer, Error>, action: Action);
}
