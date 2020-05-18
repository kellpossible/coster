use crate::{middleware::Middleware, AsListener, Listener, Reducer, StoreEvent};
use std::iter::FromIterator;
use std::{
    cell::RefCell, collections::HashSet, convert::TryInto, hash::Hash, marker::PhantomData, rc::Rc,
};

struct ListenerEventPair<State, Event> {
    pub listener: Listener<State, Event>,
    pub events: HashSet<Event>,
}

pub struct StoreRef<State, Action, Event>(Rc<RefCell<Store<State, Action, Event>>>);

impl<State, Action, Event> Clone for StoreRef<State, Action, Event> {
    fn clone(&self) -> Self {
        StoreRef(self.0.clone())
    }
}

impl<State, Action, Event> PartialEq for StoreRef<State, Action, Event> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<State, Action, Event> StoreRef<State, Action, Event>
where
    Event: StoreEvent + Clone + Hash + Eq,
{
    pub fn new<R: Reducer<State, Action, Event> + 'static>(
        reducer: R,
        initial_state: State,
    ) -> Self {
        StoreRef(Rc::new(RefCell::new(Store::new(reducer, initial_state))))
    }

    pub fn state(&self) -> Rc<State> {
        self.0.borrow().state().clone()
    }

    pub fn dispatch(&self, action: Action) {
        self.0.borrow_mut().dispatch(action)
    }

    pub fn subscribe<L: AsListener<State, Event>>(&self, listener: L) {
        self.0.borrow_mut().subscribe(listener)
    }

    pub fn subscribe_event<L: AsListener<State, Event>>(&self, listener: L, event: Event) {
        self.0.borrow_mut().subscribe_event(listener, event)
    }

    pub fn subscribe_events<L: AsListener<State, Event>, E: IntoIterator<Item = Event>>(
        &self,
        listener: L,
        events: E,
    ) {
        self.0.borrow_mut().subscribe_events(listener, events)
    }

    pub fn add_middleware<M: Middleware<State, Action, Event> + 'static>(&self, middleware: M) {
        self.0.borrow_mut().add_middleware(middleware)
    }
}

pub struct Store<State, Action, Event> {
    reducer: Box<dyn Reducer<State, Action, Event>>,
    state: Rc<State>,
    listeners: Vec<ListenerEventPair<State, Event>>,
    action_middleware: Vec<Rc<RefCell<dyn Middleware<State, Action, Event>>>>,
    prev_middleware: i32,
    phantom_action: PhantomData<Action>,
    phantom_event: PhantomData<Event>,
}

impl<State, Action, Event> Store<State, Action, Event>
where
    Event: StoreEvent + Clone + Hash + Eq,
{
    pub fn new<R: Reducer<State, Action, Event> + 'static>(
        reducer: R,
        initial_state: State,
    ) -> Self {
        Self {
            reducer: Box::new(reducer),
            state: Rc::new(initial_state),
            listeners: Vec::new(),
            action_middleware: Vec::new(),
            prev_middleware: -1,
            phantom_action: PhantomData,
            phantom_event: PhantomData,
        }
    }

    pub fn state(&self) -> &Rc<State> {
        &self.state
    }

    fn dispatch_reducer(&mut self, action: Action) -> Vec<Event> {
        let (state, events) = self.reducer.reduce(&self.state, action);
        self.state = Rc::new(state);
        events
    }

    fn dispatch_middleware_reduce(&mut self, action: Action) -> Vec<Event> {
        self.prev_middleware = -1;
        self.dispatch_middleware_reduce_next(Some(action))
    }

    fn dispatch_middleware_reduce_next(&mut self, action: Option<Action>) -> Vec<Event> {
        let current_middleware = self.prev_middleware + 1;
        if current_middleware as usize == self.action_middleware.len() {
            return match action {
                Some(action) => self.dispatch_reducer(action),
                None => Vec::new(),
            };
        }

        // assign before invoking the middleware which will rely
        // on this value for the next() function.
        self.prev_middleware = current_middleware;

        let result = self.action_middleware[current_middleware as usize]
            .clone()
            .borrow_mut()
            .on_reduce(self, action, Self::dispatch_middleware_reduce_next);

        result
    }

    fn notify_listeners(&mut self, events: Vec<Event>) {
        let mut listeners_to_remove: Vec<usize> = Vec::new();
        for (i, pair) in self.listeners.iter().enumerate() {
            let retain = match pair.listener.as_callback() {
                Some(callback) => {
                    if pair.events.is_empty() {
                        callback.emit(self.state.clone(), Event::none());
                    } else {
                        //  call the listener for every matching listener event
                        for event in &events {
                            if pair.events.contains(event) {
                                callback.emit(self.state.clone(), event.clone());
                            }
                        }
                    }

                    true
                }
                None => false,
            };

            if !retain {
                listeners_to_remove.push(i);
            }
        }

        for index in listeners_to_remove {
            self.listeners.swap_remove(index);
        }
    }

    pub fn dispatch(&mut self, action: Action) {
        let events = if self.action_middleware.is_empty() {
            self.dispatch_reducer(action)
        } else {
            self.dispatch_middleware_reduce(action)
        };

        // TODO: if there was no action (after the middleware), then don't notify.
        self.notify_listeners(events);
    }

    pub fn subscribe<L: AsListener<State, Event>>(&mut self, listener: L) {
        self.listeners.push(ListenerEventPair {
            listener: listener.as_listener(),
            events: HashSet::new(),
        });
    }

    pub fn subscribe_event<L: AsListener<State, Event>>(&mut self, listener: L, event: Event) {
        let mut events = HashSet::with_capacity(1);
        events.insert(event);

        self.listeners.push(ListenerEventPair {
            listener: listener.as_listener(),
            events,
        });
    }

    pub fn subscribe_events<L: AsListener<State, Event>, E: IntoIterator<Item = Event>>(
        &mut self,
        listener: L,
        events: E,
    ) {
        self.listeners.push(ListenerEventPair {
            listener: listener.as_listener(),
            events: HashSet::from_iter(events.into_iter()),
        });
    }

    pub fn add_middleware<M: Middleware<State, Action, Event> + 'static>(&mut self, middleware: M) {
        self.action_middleware
            .push(Rc::new(RefCell::new(middleware)));
    }
}

#[cfg(test)]
mod tests {
    use crate::{middleware::Middleware, Callback, Reducer, Store, StoreEvent};
    use std::{cell::RefCell, rc::Rc};

    #[derive(Debug, PartialEq)]
    struct TestState {
        counter: i32,
    }

    #[derive(Copy, Clone)]
    enum TestAction {
        Increment,
        Decrement,
        Decrement2,
    }

    struct TestReducer;

    impl Reducer<TestState, TestAction, TestEvent> for TestReducer {
        fn reduce(&self, state: &TestState, action: TestAction) -> (TestState, Vec<TestEvent>) {
            let mut events = Vec::new();
            let new_state = match action {
                TestAction::Increment => TestState {
                    counter: state.counter + 1,
                },
                TestAction::Decrement => TestState {
                    counter: state.counter - 1,
                },
                TestAction::Decrement2 => TestState {
                    counter: state.counter - 2,
                },
            };

            if new_state.counter != state.counter && new_state.counter == 0 {
                events.push(TestEvent::IsZero);
            }

            (new_state, events)
        }
    }
    struct TestMiddleware {
        new_action: TestAction,
    }

    impl Middleware<TestState, TestAction, TestEvent> for TestMiddleware {
        fn on_reduce(
            &mut self,
            store: &mut Store<TestState, TestAction, TestEvent>,
            action: Option<TestAction>,
            reduce: crate::middleware::ReduceFn<TestState, TestAction, TestEvent>,
        ) -> Vec<TestEvent> {
            reduce(store, action.map(|_| self.new_action))
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    enum TestEvent {
        Change(i32),
        IsZero,
        None,
    }

    impl StoreEvent for TestEvent {
        fn none() -> Self {
            Self::None
        }

        fn is_none(&self) -> bool {
            match self {
                TestEvent::None => true,
                _ => false,
            }
        }
    }

    #[test]
    fn test_notify() {
        let initial_state = TestState { counter: 0 };
        let store: Rc<RefCell<Store<TestState, TestAction, TestEvent>>> =
            Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback_test = Rc::new(RefCell::new(0));
        let callback_test_copy = callback_test.clone();
        let callback: Callback<TestState, TestEvent> =
            Callback::new(move |state: Rc<TestState>, _| {
                *callback_test_copy.borrow_mut() = state.counter;
            });

        store.borrow_mut().subscribe(&callback);

        assert_eq!(0, store.borrow().state().counter);

        store.borrow_mut().dispatch(TestAction::Increment);
        store.borrow_mut().dispatch(TestAction::Increment);
        assert_eq!(2, *callback_test.borrow());
        assert_eq!(2, store.borrow().state().counter);

        store.borrow_mut().dispatch(TestAction::Decrement);
        assert_eq!(1, store.borrow().state().counter);
    }

    #[test]
    fn test_middleware() {
        let initial_state = TestState { counter: 0 };
        let store = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback_test = Rc::new(RefCell::new(0));
        let callback_test_copy = callback_test.clone();
        let callback: Callback<TestState, TestEvent> =
            Callback::new(move |state: Rc<TestState>, _| {
                *callback_test_copy.borrow_mut() = state.counter;
            });

        let mut store_mut = store.borrow_mut();

        store_mut.subscribe(&callback);
        store_mut.add_middleware(TestMiddleware {
            new_action: TestAction::Decrement,
        });
        store_mut.add_middleware(TestMiddleware {
            new_action: TestAction::Decrement2,
        });

        store_mut.dispatch(TestAction::Increment);
        assert_eq!(-2, *callback_test.borrow());
    }

    #[test]
    fn test_middleware_reverse_order() {
        let initial_state = TestState { counter: 0 };
        let store = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback_test = Rc::new(RefCell::new(0));
        let callback_test_copy = callback_test.clone();
        let callback: Callback<TestState, TestEvent> =
            Callback::new(move |state: Rc<TestState>, _| {
                *callback_test_copy.borrow_mut() = state.counter;
            });

        let mut store_mut = store.borrow_mut();

        store_mut.subscribe(&callback);
        store_mut.add_middleware(TestMiddleware {
            new_action: TestAction::Decrement2,
        });
        store_mut.add_middleware(TestMiddleware {
            new_action: TestAction::Decrement,
        });

        store_mut.dispatch(TestAction::Increment);
        assert_eq!(-1, *callback_test.borrow());
    }

    #[test]
    fn test_subscribe_event() {
        let initial_state = TestState { counter: -2 };
        let store: Rc<RefCell<Store<TestState, TestAction, TestEvent>>> =
            Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback_test: Rc<RefCell<Option<TestEvent>>> = Rc::new(RefCell::new(None));
        let callback_test_copy = callback_test.clone();

        let callback_zero_subscription: Callback<TestState, TestEvent> =
            Callback::new(move |_: Rc<TestState>, event| {
                assert_eq!(TestEvent::IsZero, event);
                *callback_test_copy.borrow_mut() = Some(TestEvent::IsZero);
            });

        let mut store_mut = store.borrow_mut();
        store_mut.subscribe_event(&callback_zero_subscription, TestEvent::IsZero);
        store_mut.dispatch(TestAction::Increment);
        assert_eq!(None, *callback_test.borrow());
        store_mut.dispatch(TestAction::Increment);
        assert_eq!(Some(TestEvent::IsZero), *callback_test.borrow());
    }
}
