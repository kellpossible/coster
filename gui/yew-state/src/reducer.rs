pub trait Reducer<State, Action, Event> {
    fn reduce(&self, state: &State, action: Action) -> ReducerResult<State, Event>;
}

impl<State, Action, Event> Reducer<State, Action, Event>
    for dyn Fn(&State, Action) -> (State, Vec<Event>)
{
    fn reduce(&self, state: &State, action: Action) -> ReducerResult<State, Event> {
        self(state, action)
    }
}

pub type ReducerResult<State, Event> = (State, Vec<Event>);
