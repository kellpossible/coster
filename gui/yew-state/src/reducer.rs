pub trait Reducer<State, Action> {
    fn reduce(&self, state: &State, action: &Action) -> State;
}

impl<State, Action> Reducer<State, Action> for dyn Fn(&State, &Action) -> State {
    fn reduce(&self, state: &State, action: &Action) -> State {
        self(state, action)
    }
}
