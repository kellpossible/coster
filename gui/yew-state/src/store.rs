use crate::{middleware::ActionMiddleware, AsListener, CallbackResults, Listener, Reducer};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

pub struct Store<State, Action, Error> {
    reducer: Box<dyn Reducer<State, Action>>,
    state: Rc<State>,
    listeners: Vec<Listener<State, Error>>,
    action_middleware: Vec<Rc<RefCell<dyn ActionMiddleware<State, Action, Error>>>>,
    phantom_action: PhantomData<Action>,
    prev_middleware: i32,
}

impl<State, Action, Error> Store<State, Action, Error> {
    pub fn new<R: Reducer<State, Action> + 'static>(reducer: R, initial_state: State) -> Self {
        Self {
            reducer: Box::new(reducer),
            state: Rc::new(initial_state),
            listeners: vec![],
            action_middleware: vec![],
            phantom_action: PhantomData,
            prev_middleware: -1,
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
        self.notify_listeners()
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

    fn notify_listeners(&mut self) -> CallbackResults<Error> {
        let mut errors = Vec::new();
        let mut listeners_to_remove: Vec<usize> = Vec::new();
        for (i, listener) in self.listeners.iter().enumerate() {
            let retain = match listener.as_callback() {
                Some(callback) => {
                    match callback.emit(self.state.clone()) {
                        Ok(()) => {}
                        Err(error) => errors.push(error),
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

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }

    pub fn subscribe<L: AsListener<State, Error>>(&mut self, listener: L) {
        self.listeners.push(listener.as_listener());
    }

    pub fn subscribe_to_action<L: AsListener<State, Error>>(&mut self, listener: L) {}

    pub fn add_action_middleware<M: ActionMiddleware<State, Action, Error> + 'static>(
        &mut self,
        middleware: M,
    ) {
        self.action_middleware
            .push(Rc::new(RefCell::new(middleware)));
    }
}

#[cfg(test)]
mod tests {
    use crate::{middleware::ActionMiddleware, Callback, Reducer, Store};
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

    impl<Error> ActionMiddleware<TestState, TestAction, Error> for TestMiddleware {
        fn invoke(
            &mut self,
            store: &mut Store<TestState, TestAction, Error>,
            action: Option<TestAction>,
            next: crate::middleware::NextFn<TestState, TestAction, Error>,
        ) -> crate::CallbackResults<Error> {
            next(store, action.map(|_| self.new_action))
        }
    }

    #[test]
    fn test_notify() {
        let initial_state = TestState { counter: 0 };
        let store = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback_test = Rc::new(RefCell::new(0));
        let callback_test_copy = callback_test.clone();
        let callback: Callback<TestState, ()> = Callback::new(move |state: Rc<TestState>| {
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
        let store = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback: Callback<TestState, anyhow::Error> =
            Callback::new(move |_: Rc<TestState>| Err(anyhow::anyhow!("Test Error")));

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
        let store = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback: Callback<TestState, TestError> =
            Callback::new(move |_: Rc<TestState>| Err(TestError::Error));

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
        let callback: Callback<TestState, ()> = Callback::new(move |state: Rc<TestState>| {
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
        let callback: Callback<TestState, ()> = Callback::new(move |state: Rc<TestState>| {
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
}
