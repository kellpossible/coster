use crate::{middleware::Middleware, AsListener, Listener, Reducer, StoreEvent};
use std::iter::FromIterator;
use std::ops::Deref;
use std::{
    cell::{Cell, RefCell},
    collections::{HashSet, VecDeque},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    rc::Rc,
};

struct ListenerEventPair<State, Event> {
    pub listener: Listener<State, Event>,
    pub events: HashSet<Event>,
}

impl<State, Event> Debug for ListenerEventPair<State, Event> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ListenerEventPair")
    }
}

enum StoreModification<State, Action, Event> {
    AddListener(ListenerEventPair<State, Event>),
    AddMiddleware(Rc<dyn Middleware<State, Action, Event>>),
}

#[derive(Clone)]
pub struct StoreRef<State, Action, Event>(Rc<Store<State, Action, Event>>);

impl<State, Action, Event> StoreRef<State, Action, Event>
where
    Event: StoreEvent + Clone + Hash + Eq,
{
    pub fn new<R: Reducer<State, Action, Event> + 'static>(
        reducer: R,
        initial_state: State,
    ) -> Self {
        Self(Rc::new(Store::new(reducer, initial_state)))
    }
}

impl<State, Action, Event> Deref for StoreRef<State, Action, Event> {
    type Target = Store<State, Action, Event>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<State, Action, Event> PartialEq for StoreRef<State, Action, Event> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

pub struct Store<State, Action, Event> {
    /// This lock is used to prevent dispatch recursion and a large stack.
    dispatch_lock: RefCell<()>,
    dispatch_queue: RefCell<VecDeque<Action>>,
    modification_queue: RefCell<VecDeque<StoreModification<State, Action, Event>>>,
    reducer: Box<dyn Reducer<State, Action, Event>>,
    state: RefCell<Rc<State>>,
    listeners: RefCell<Vec<ListenerEventPair<State, Event>>>,
    middleware: RefCell<Vec<Rc<dyn Middleware<State, Action, Event>>>>,
    prev_middleware: Cell<i32>,
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
            dispatch_lock: RefCell::new(()),
            dispatch_queue: RefCell::new(VecDeque::new()),
            modification_queue: RefCell::new(VecDeque::new()),
            reducer: Box::new(reducer),
            state: RefCell::new(Rc::new(initial_state)),
            listeners: RefCell::new(Vec::new()),
            middleware: RefCell::new(Vec::new()),
            prev_middleware: Cell::new(-1),
            phantom_action: PhantomData,
            phantom_event: PhantomData,
        }
    }

    pub fn state(&self) -> Rc<State> {
        self.state.borrow().clone()
    }

    fn dispatch_reducer(&self, action: Action) -> Vec<Event> {
        let (state, events) = self.reducer.reduce(self.state(), action);
        *self.state.borrow_mut() = state;
        events
    }

    fn dispatch_middleware_reduce(&self, action: Action) -> Vec<Event> {
        self.prev_middleware.set(-1);
        self.dispatch_middleware_reduce_next(Some(action))
    }

    fn dispatch_middleware_reduce_next(&self, action: Option<Action>) -> Vec<Event> {
        let current_middleware = self.prev_middleware.get() + 1;
        self.prev_middleware.set(current_middleware);
        if current_middleware as usize == self.middleware.borrow().len() {
            return match action {
                //TODO: move this outside the dispatch middleware because it could be
                // a situation where a middleware decides not to call next and this will
                // never be reached.
                Some(action) => self.dispatch_reducer(action),
                None => Vec::new(),
            };
        }

        let result = self.middleware.borrow()[current_middleware as usize]
            .clone()
            .on_reduce(self, action, Self::dispatch_middleware_reduce_next);

        result
    }

    fn dispatch_middleware_notify(&self, events: Vec<Event>) -> Vec<Event> {
        self.prev_middleware.set(-1);
        self.dispatch_middleware_notify_next(events)
    }

    fn dispatch_middleware_notify_next(&self, events: Vec<Event>) -> Vec<Event> {
        let current_middleware = self.prev_middleware.get() + 1;
        self.prev_middleware.set(current_middleware);

        if current_middleware as usize == self.middleware.borrow().len() {
            return events;
        }

        self.middleware.borrow()[current_middleware as usize]
            .clone()
            .on_notify(self, events, Self::dispatch_middleware_notify_next)
    }

    fn notify_listeners(&self, events: Vec<Event>) {
        let mut listeners_to_remove: Vec<usize> = Vec::new();
        for (i, pair) in self.listeners.borrow().iter().enumerate() {
            let retain = match pair.listener.as_callback() {
                Some(callback) => {
                    if pair.events.is_empty() {
                        callback.emit(self.state.borrow().clone(), Event::none());
                    } else {
                        //  call the listener for every matching listener event
                        for event in &events {
                            if pair.events.contains(event) {
                                callback.emit(self.state.borrow().clone(), event.clone());
                            }
                        }
                    }

                    true
                }
                None => false,
            };

            if !retain {
                listeners_to_remove.insert(0, i);
            }
        }

        for index in listeners_to_remove {
            self.listeners.borrow_mut().swap_remove(index);
        }
    }

    fn process_pending_modifications(&self) {
        while let Some(modification) = self.modification_queue.borrow_mut().pop_front() {
            match modification {
                StoreModification::AddListener(listener_pair) => {
                    self.listeners.borrow_mut().push(listener_pair);
                }
                StoreModification::AddMiddleware(middleware) => {
                    self.middleware.borrow_mut().push(middleware);
                }
            }
        }
    }

    pub fn dispatch(&self, action: Action) {
        self.dispatch_queue.borrow_mut().push_back(action);

        // If the lock fails to acquire, then the dispatch is already in progress.
        // This prevents recursion, when a listener callback also triggers another
        // dispatch.
        if let Ok(_lock) = self.dispatch_lock.try_borrow_mut() {
            while let Some(action) = self.dispatch_queue.borrow_mut().pop_front() {
                self.process_pending_modifications();

                let events = if self.middleware.borrow().is_empty() {
                    self.dispatch_reducer(action)
                } else {
                    self.dispatch_middleware_reduce(action)
                };

                // TODO: if there was no action (after the middleware), then don't notify.
                let middleware_events = self.dispatch_middleware_notify(events);
                self.notify_listeners(middleware_events)
            }
        }
    }

    pub fn subscribe<L: AsListener<State, Event>>(&self, listener: L) {
        self.modification_queue
            .borrow_mut()
            .push_back(StoreModification::AddListener(ListenerEventPair {
                listener: listener.as_listener(),
                events: HashSet::new(),
            }));
    }

    pub fn subscribe_event<L: AsListener<State, Event>>(&self, listener: L, event: Event) {
        let mut events = HashSet::with_capacity(1);
        events.insert(event);

        self.modification_queue
            .borrow_mut()
            .push_back(StoreModification::AddListener(ListenerEventPair {
                listener: listener.as_listener(),
                events,
            }));
    }

    pub fn subscribe_events<L: AsListener<State, Event>, E: IntoIterator<Item = Event>>(
        &self,
        listener: L,
        events: E,
    ) {
        self.modification_queue
            .borrow_mut()
            .push_back(StoreModification::AddListener(ListenerEventPair {
                listener: listener.as_listener(),
                events: HashSet::from_iter(events.into_iter()),
            }));
    }

    pub fn add_middleware<M: Middleware<State, Action, Event> + 'static>(&self, middleware: M) {
        self.modification_queue
            .borrow_mut()
            .push_back(StoreModification::AddMiddleware(Rc::new(middleware)));
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        middleware::Middleware, Callback, Reducer, ReducerResult, Store, StoreEvent, StoreRef,
    };
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
        fn reduce(
            &self,
            state: Rc<TestState>,
            action: TestAction,
        ) -> ReducerResult<TestState, TestEvent> {
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

            (Rc::new(new_state), events)
        }
    }

    struct TestMiddleware {
        new_action: TestAction,
    }

    impl Middleware<TestState, TestAction, TestEvent> for TestMiddleware {
        fn on_reduce(
            &self,
            store: &Store<TestState, TestAction, TestEvent>,
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
        let store = StoreRef::new(TestReducer, initial_state);

        let callback_test = Rc::new(RefCell::new(0));
        let callback_test_copy = callback_test.clone();
        let callback: Callback<TestState, TestEvent> =
            Callback::new(move |state: Rc<TestState>, _| {
                *callback_test_copy.borrow_mut() = state.counter;
            });

        store.subscribe(&callback);
        store.add_middleware(TestMiddleware {
            new_action: TestAction::Decrement,
        });
        store.add_middleware(TestMiddleware {
            new_action: TestAction::Decrement2,
        });

        store.dispatch(TestAction::Increment);
        assert_eq!(-2, *callback_test.borrow());
    }

    #[test]
    fn test_middleware_reverse_order() {
        let initial_state = TestState { counter: 0 };
        let store = StoreRef::new(TestReducer, initial_state);

        let callback_test = Rc::new(RefCell::new(0));
        let callback_test_copy = callback_test.clone();
        let callback: Callback<TestState, TestEvent> =
            Callback::new(move |state: Rc<TestState>, _| {
                *callback_test_copy.borrow_mut() = state.counter;
            });

        store.subscribe(&callback);
        store.add_middleware(TestMiddleware {
            new_action: TestAction::Decrement2,
        });
        store.add_middleware(TestMiddleware {
            new_action: TestAction::Decrement,
        });

        store.dispatch(TestAction::Increment);
        assert_eq!(-1, *callback_test.borrow());
    }

    #[test]
    fn test_subscribe_event() {
        let initial_state = TestState { counter: -2 };
        let store = StoreRef::new(TestReducer, initial_state);

        let callback_test: Rc<RefCell<Option<TestEvent>>> = Rc::new(RefCell::new(None));
        let callback_test_copy = callback_test.clone();

        let callback_zero_subscription: Callback<TestState, TestEvent> =
            Callback::new(move |_: Rc<TestState>, event| {
                assert_eq!(TestEvent::IsZero, event);
                *callback_test_copy.borrow_mut() = Some(TestEvent::IsZero);
            });

        store.subscribe_event(&callback_zero_subscription, TestEvent::IsZero);
        store.dispatch(TestAction::Increment);
        assert_eq!(None, *callback_test.borrow());
        store.dispatch(TestAction::Increment);
        assert_eq!(Some(TestEvent::IsZero), *callback_test.borrow());
    }
}
