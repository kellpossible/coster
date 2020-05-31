use std::rc::Rc;

pub trait Reducer<State, Action, Event> {
    fn reduce(&self, prev_state: Rc<State>, action: Action) -> ReducerResult<State, Event>;
}

impl<State, Action, Event> Reducer<State, Action, Event>
    for dyn Fn(Rc<State>, Action) -> (Rc<State>, Vec<Event>)
{
    fn reduce(&self, prev_state: Rc<State>, action: Action) -> ReducerResult<State, Event> {
        self(prev_state, action)
    }
}

pub type ReducerResult<State, Event> = (Rc<State>, Vec<Event>);
