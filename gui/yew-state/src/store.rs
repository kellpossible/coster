use crate::{middleware::ActionMiddleware, CallbackResults, Listener, Reducer};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

pub struct Store<State, Action, Error> {
    reducer: Box<dyn Reducer<State, Action>>,
    state: Rc<State>,
    listeners: Vec<Listener<State, Error>>,
    action_middleware: Vec<Rc<RefCell<dyn ActionMiddleware<State, Action, Error>>>>,
    phantom_action: PhantomData<Action>,
}

impl<State, Action, Error> Store<State, Action, Error> {
    pub fn new<R: Reducer<State, Action> + 'static>(reducer: R, initial_state: State) -> Self {
        Self {
            reducer: Box::new(reducer),
            state: Rc::new(initial_state),
            listeners: vec![],
            action_middleware: vec![],
            phantom_action: PhantomData,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn dispatch(&mut self, action: Action) -> CallbackResults<Error> {
        if self.action_middleware.is_empty() {
            self.dispatch_reducer(action)
        } else {
            self.dispatch_middleware(0, Some(action))
        }
    }

    fn dispatch_reducer(&mut self, action: Action) -> CallbackResults<Error> {
        self.state = Rc::new(self.reducer.reduce(&self.state, &action));
        self.notify_listeners()
    }

    fn dispatch_middleware(
        &mut self,
        index: usize,
        action: Option<Action>,
    ) -> CallbackResults<Error> {
        if index == self.action_middleware.len() {
            return match action {
                Some(action) => self.dispatch_reducer(action),
                None => Ok(()),
            };
        }

        let next = self.action_middleware[index]
            .clone()
            .borrow_mut()
            .invoke(self, action);

        return self.dispatch_middleware(index + 1, next);
    }

    fn notify_listeners(&mut self) -> CallbackResults<Error> {
        let mut errors = Vec::new();
        let mut listeners_to_remove: Vec<usize> = Vec::new();
        for (i, weak_listener) in self.listeners.iter().enumerate() {
            let retain = match weak_listener.upgrade() {
                Some(listener) => {
                    match (listener)(self.state.clone()) {
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

    pub fn subscribe(&mut self, listener: Listener<State, Error>) {
        self.listeners.push(listener);
    }

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
    use crate::{ActionMiddleware, Callback, Reducer, Store};
    use anyhow;
    use std::{cell::RefCell, rc::Rc};
    use thiserror::Error;

    #[derive(Debug, PartialEq)]
    struct TestState {
        counter: i32,
    }

    enum TestAction {
        Increment,
        Decrement,
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
            }
        }
    }

    #[derive(Error, Debug, PartialEq)]
    enum TestError {
        #[error("A test Error")]
        Error,
    }

    struct TestMiddleware;

    impl<Error> ActionMiddleware<TestState, TestAction, Error> for TestMiddleware {
        fn invoke(
            &mut self,
            _: &mut Store<TestState, TestAction, Error>,
            action: Option<TestAction>,
        ) -> Option<TestAction> {
            action.map(|_| TestAction::Decrement)
        }
    }

    #[test]
    fn test_notify() {
        let initial_state = TestState { counter: 0 };
        let store = Rc::new(RefCell::new(Store::new(TestReducer, initial_state)));

        let callback_test = Rc::new(RefCell::new(0));
        let callback_test_copy = callback_test.clone();
        let callback: Callback<TestState, ()> = Rc::new(move |state: Rc<TestState>| {
            *callback_test_copy.borrow_mut() = state.counter;
            Ok(())
        });

        store.borrow_mut().subscribe(Rc::downgrade(&callback));

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
            Rc::new(move |_: Rc<TestState>| Err(anyhow::anyhow!("Test Error")));

        store.borrow_mut().subscribe(Rc::downgrade(&callback));

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
            Rc::new(move |_: Rc<TestState>| Err(TestError::Error));

        store.borrow_mut().subscribe(Rc::downgrade(&callback));

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
        let callback: Callback<TestState, ()> = Rc::new(move |state: Rc<TestState>| {
            *callback_test_copy.borrow_mut() = state.counter;
            Ok(())
        });

        store.borrow_mut().subscribe(Rc::downgrade(&callback));
        store.borrow_mut().add_action_middleware(TestMiddleware);

        store.borrow_mut().dispatch(TestAction::Increment).unwrap();
        assert_eq!(-1, *callback_test.borrow());
    }
}
