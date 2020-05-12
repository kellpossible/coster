use std::rc::Rc;

pub trait EventsFrom<State, Action>
where
    Self: Sized,
{
    fn events_from(state: &Rc<State>, action: &Action) -> Vec<Self>;
}

impl<State, Action> EventsFrom<State, Action> for () {
    fn events_from(state: &Rc<State>, action: &Action) -> Vec<Self> {
        Vec::new()
    }
}