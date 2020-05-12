use crate::{
    middleware::ActionMiddleware, AsEventListener, CallbackResults, EventsFrom, EventListener, Reducer, Listener,
};
use std::{cell::RefCell, marker::PhantomData, rc::Rc, hash::Hash, collections::HashSet};

struct ListenerEventPair<State, Error, Event> {
    pub listener: EventListener<State, Error, Event>,
    pub events: HashSet<Event>,
}

pub struct Store<State, Action, Error, Event> {
    reducer: Box<dyn Reducer<State, Action>>,
    state: Rc<State>,
    //TODO: perhaps listeners should be collapsed into an enum like yew::Callback?
    listeners: Vec<Listener<State, Error, Event>>,
    event_listeners: Vec<ListenerEventPair<State, Error, Event>>,
    action_middleware: Vec<Rc<RefCell<dyn ActionMiddleware<State, Action, Error, Event>>>>,
    prev_middleware: i32,
    phantom_action: PhantomData<Action>,
    phantom_event: PhantomData<Event>,
}

impl<State, Action, Error, Event> Store<State, Action, Error, Event>
where
    Event: EventsFrom<State, Action> + Clone + Hash + Eq
{
    pub fn new<R: Reducer<State, Action> + 'static>(reducer: R, initial_state: State) -> Self {
        Self {
            reducer: Box::new(reducer),
            state: Rc::new(initial_state),
            listeners: Vec::new(),
            event_listeners: Vec::new(),
            action_middleware: Vec::new(),
            prev_middleware: -1,
            phantom_action: PhantomData,
            phantom_event: PhantomData,
        }
    }

    pub fn state(&self) -> &Rc<State> {
        &self.state
    }

    pub fn dispatch(&mut self, action: Action) -> CallbackResults<Error> {
        if self.action_middleware.is_empty() {
            self.dispatch_reducer(action)
        } else {
            self.dispatch_middleware(action)
        }
    }

    fn dispatch_reducer(&mut self, action: Action) -> CallbackResults<Error> {
        self.state = Rc::new(self.reducer.reduce(&self.state, &action));
        self.notify_listeners(action)
    }
    fn dispatch_middleware(&mut self, action: Action) -> CallbackResults<Error> {
        self.prev_middleware = -1;
        self.dispatch_middleware_next(Some(action))
    }

    fn dispatch_middleware_next(&mut self, action: Option<Action>) -> CallbackResults<Error> {
        let current_middleware = self.prev_middleware + 1;
        if current_middleware as usize == self.action_middleware.len() {
            return match action {
                Some(action) => self.dispatch_reducer(action),
                None => Ok(()),
            };
        }

        // assign before invoking the middleware which will rely
        // on this value for the next() function.
        self.prev_middleware = current_middleware;

        let result = self.action_middleware[current_middleware as usize]
            .clone()
            .borrow_mut()
            .invoke(self, action, Self::dispatch_middleware_next);

        result
    }

    fn notify_listeners(&mut self, action: Action) -> CallbackResults<Error> {
        let events = Event::events_from(&self.state, &action);
        let mut errors = Vec::new();
        let mut listeners_to_remove: Vec<usize> = Vec::new();
        for (i, pair) in self.event_listeners.iter().enumerate() {
            let retain = match pair.listener.as_callback() {
                Some(callback) => {
                    //  call the listener for every matching listener event
                    for event in &events {
                        if pair.events.contains(event) {
                            match callback.emit(self.state.clone(), *event) {
                                Ok(()) => {}
                                Err(error) => errors.push(error),
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
            self.event_listeners.swap_remove(index);
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }

    pub fn subscribe<L: AsListener<State, Error, Event>>(&mut self, listener: L) {
        self.listeners.push(listener);
    }

    pub fn add_action_middleware<M: ActionMiddleware<State, Action, Error, Event> + 'static>(
        &mut self,
        middleware: M,
    ) {
        self.action_middleware
            .push(Rc::new(RefCell::new(middleware)));
    }
}

pub trait EventSubscription<State, Error, Event> {
    fn subscribe_event<L: AsEventListener<State, Error, Event>>(&mut self, listener: L, event: Event);
    fn subscribe_events<L: AsEventListener<State, Error, Event>, E: Into<HashSet<Event>>>(&mut self, listener: L, events: E);
}

impl <State, Action, Error, Event> EventSubscription<State, Error, Event> for Store<State, Action, Error, Event> 
where Event: Hash + Eq
{
    fn subscribe_event<L: AsEventListener<State, Error, Event>>(&mut self, listener: L, event: Event) {
        let mut events = HashSet::with_capacity(1);
        events.insert(event);

        self.event_listeners.push(ListenerEventPair {
            listener: listener.as_listener(),
            events
        });
    }
    fn subscribe_events<L: AsEventListener<State, Error, Event>, E: Into<HashSet<Event>>>(&mut self, listener: L, events: E) {
        //TODO: implement this
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{middleware::ActionMiddleware, EventCallback, Reducer, Store, EventsFrom, EventSubscription};
    use anyhow;
    use std::{cell::RefCell, rc::Rc};
    use thiserror::Error;

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

    impl Reducer<TestState, TestAction> for TestReducer {
        fn reduce(&self, state: &TestState, action: &TestAction) -> TestState {
            match action {
                TestAction::Increment => TestState {
                    counter: state.counter + 1,
                },
                TestAction::Decrement => TestState {
                    counter: state.counter - 1,
                },
                TestAction::Decrement2 => TestState {
                    counter: state.counter - 2,
                },
            }
        }
    }

    #[derive(Error, Debug, PartialEq)]
    enum TestError {
        #[error("A test Error")]
        Error,
    }

    struct TestMiddleware {
        new_action: TestAction,
    }

    #[derive(Debug, PartialEq, Eq, Hash)]
    enum TestEvent {
        Change(i32),
        IsZero,
    }

    impl EventsFrom<TestState, TestAction> for TestEvent {
        fn events_from(state: &Rc<TestState>, action: &TestAction) -> Vec<Self> {
            let mut events = Vec::new();

            match action {
                TestAction::Increment => events.push(TestEvent::Change(1)),
                TestAction::Decrement => events.push(TestEvent::Change(-1)),
                TestAction::Decrement2 => events.push(TestEvent::Change(1)),
            }

            if state.counter == 0 {
                events.push(TestEvent::IsZero);
            }

            events
        }
    }

    impl ActionMiddleware<TestState, TestAction, (), ()> for TestMiddleware {
        fn invoke(
            &mut self,
            store: &mut Store<TestState, TestAction, (), ()>,
            action: Option<TestAction>,
            next: crate::middleware::NextFn<TestState, TestAction, (), ()>,
        ) -> crate::CallbackResults<()> {
            next(store, action.map(|_| self.new_action))
        }
    }

    #[test]
    fn test_notify() {
        let initial_state = TestState { counter: 0 };
        let store: Rc<RefCell<Store<TestState, TestAction, (), TestEvent>>> = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback_test = Rc::new(RefCell::new(0));
        let callback_test_copy = callback_test.clone();
        let callback: EventCallback<TestState, (), ()> = EventCallback::new(move |state: Rc<TestState>, _| {
            *callback_test_copy.borrow_mut() = state.counter;
            Ok(())
        });

        store.borrow_mut().subscribe(&callback);

        assert_eq!(0, store.borrow().state().counter);

        store.borrow_mut().dispatch(TestAction::Increment).unwrap();
        store.borrow_mut().dispatch(TestAction::Increment).unwrap();
        assert_eq!(2, *callback_test.borrow());
        assert_eq!(2, store.borrow().state().counter);

        store.borrow_mut().dispatch(TestAction::Decrement).unwrap();
        assert_eq!(1, store.borrow().state().counter);
    }

    #[test]
    fn test_anyhow_error() {
        let initial_state = TestState { counter: 0 };
        let store: Rc<RefCell<Store<TestState, TestAction, anyhow::Error, TestEvent>>> = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback: EventCallback<TestState, anyhow::Error> =
            EventCallback::new(move |_: Rc<TestState>| Err(anyhow::anyhow!("Test Error")));

        store.borrow_mut().subscribe(&callback);

        match store.borrow_mut().dispatch(TestAction::Increment) {
            Err(errors) => {
                assert_eq!(1, errors.len());
                let error: &anyhow::Error = errors.get(0).unwrap();
                assert_eq!("Test Error", error.to_string());
            }
            Ok(()) => {
                panic!("no error");
            }
        };
    }

    #[test]
    fn test_custom_error() {
        let initial_state = TestState { counter: 0 };
        let store: Rc<RefCell<Store<TestState, TestAction, TestError, TestEvent>>> = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback: EventCallback<TestState, TestError> =
            EventCallback::new(move |_: Rc<TestState>| Err(TestError::Error));

        store.borrow_mut().subscribe(&callback);

        match store.borrow_mut().dispatch(TestAction::Increment) {
            Err(errors) => {
                assert_eq!(1, errors.len());
                let error: &TestError = errors.get(0).unwrap();
                assert_eq!(TestError::Error, *error);
            }
            Ok(()) => {
                panic!("no error");
            }
        };
    }

    #[test]
    fn test_middleware() {
        let initial_state = TestState { counter: 0 };
        let store = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback_test = Rc::new(RefCell::new(0));
        let callback_test_copy = callback_test.clone();
        let callback: EventCallback<TestState, ()> = EventCallback::new(move |state: Rc<TestState>| {
            *callback_test_copy.borrow_mut() = state.counter;
            Ok(())
        });

        let mut store_mut = store.borrow_mut();

        store_mut.subscribe(&callback);
        store_mut.add_action_middleware(TestMiddleware {
            new_action: TestAction::Decrement,
        });
        store_mut.add_action_middleware(TestMiddleware {
            new_action: TestAction::Decrement2,
        });

        store_mut.dispatch(TestAction::Increment).unwrap();
        assert_eq!(-2, *callback_test.borrow());
    }

    #[test]
    fn test_middleware_reverse_order() {
        let initial_state = TestState { counter: 0 };
        let store = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback_test = Rc::new(RefCell::new(0));
        let callback_test_copy = callback_test.clone();
        let callback: EventCallback<TestState, ()> = EventCallback::new(move |state: Rc<TestState>| {
            *callback_test_copy.borrow_mut() = state.counter;
            Ok(())
        });

        let mut store_mut = store.borrow_mut();

        store_mut.subscribe(&callback);
        store_mut.add_action_middleware(TestMiddleware {
            new_action: TestAction::Decrement2,
        });
        store_mut.add_action_middleware(TestMiddleware {
            new_action: TestAction::Decrement,
        });

        store_mut.dispatch(TestAction::Increment).unwrap();
        assert_eq!(-1, *callback_test.borrow());
    }

    #[test]
    fn test_events() {
        let initial_state = TestState { counter: -2 };
        let store: Rc<RefCell<Store<TestState, TestAction, (), TestEvent>>> = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback_test: Rc<RefCell<Option<TestEvent>>> = Rc::new(RefCell::new(None));
        let callback_test_copy = callback_test.clone();
        
        let callback_zero: EventCallback<TestState, ()> = EventCallback::new(move |state: Rc<TestState>| {
            *callback_test_copy.borrow_mut() = Some(TestEvent::IsZero);
            Ok(())
        });

        let mut store_mut = store.borrow_mut();
        store_mut.subscribe_event(&callback_zero, TestEvent::IsZero);
        store_mut.dispatch(TestAction::Increment).unwrap();
        assert_eq!(None, *callback_test.borrow());
        store_mut.dispatch(TestAction::Increment).unwrap();
        assert_eq!(Some(TestEvent::IsZero), *callback_test.borrow());
    }
}
