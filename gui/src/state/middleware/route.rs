use crate::routing::{SwitchRoute, SwitchRouteService};
use std::{hash::Hash, marker::PhantomData, rc::Rc};
use yew_state::{middleware::Middleware, Store, StoreEvent, StoreRef};

pub struct RouteMiddleware<SR, State, Action, Event> {
    pub router: SwitchRouteService<SR>,
    state_type: PhantomData<State>,
    action_type: PhantomData<Action>,
    event_type: PhantomData<Event>,
}

impl<SR, State, Action, Event> RouteMiddleware<SR, State, Action, Event>
where
    SR: SwitchRoute + 'static,
    State: 'static,
    Action: RouteAction<SR> + 'static,
    Event: StoreEvent + Clone + Hash + Eq + 'static,
{
    pub fn new(store: &StoreRef<State, Action, Event>) -> Self {
        let mut router = SwitchRouteService::new();
        let store_rc = store.clone();
        let callback: yew::Callback<SR> = yew::Callback::Callback(Rc::new(move |route: SR| {
            store_rc.dispatch(Action::change_route(route));
        }));

        // TODO: this might cause errors if the callback is called from another thread...
        router.register_callback(callback);

        Self {
            router,
            state_type: PhantomData,
            action_type: PhantomData,
            event_type: PhantomData,
        }
    }
}

impl<SR, State, Action, Event> Middleware<State, Action, Event>
    for RouteMiddleware<SR, State, Action, Event>
where
    SR: SwitchRoute + 'static,
    State: RouteState<SR>,
    Event: RouteEvent<SR> + PartialEq + StoreEvent + Clone + Hash + Eq,
{
    fn on_notify(
        &mut self,
        store: &mut Store<State, Action, Event>,
        _: Action,
        events: Vec<Event>,
        notify: yew_state::middleware::NotifyFn<State, Action, Event>,
    ) {
        for event in &events {
            if event == &Event::route_changed() {
                self.router.set_route(store.state().get_route().clone());
            }
        }
        notify(store, events);
    }
}

pub trait RouteState<SR> {
    fn get_route(&self) -> &SR;
}

pub trait RouteEvent<SR>
where
    SR: SwitchRoute + 'static,
{
    fn route_changed() -> Self;
}

pub trait RouteAction<SR>
where
    SR: SwitchRoute + 'static,
{
    fn change_route<R: Into<SR>>(route: R) -> Self;
}

pub trait RouteStoreRef<SR> {
    fn change_route<R: Into<SR>>(&self, route: R);
}

impl<SR, State, Action, Event> RouteStoreRef<SR> for StoreRef<State, Action, Event>
where
    SR: SwitchRoute + 'static,
    Action: RouteAction<SR>,
    State: RouteState<SR>,
    Event: RouteEvent<SR> + PartialEq + StoreEvent + Clone + Hash + Eq,
{
    fn change_route<R: Into<SR>>(&self, route: R) {
        self.dispatch(Action::change_route(route));
    }
}

pub trait RouteStore<SR> {
    fn change_route<R: Into<SR>>(&mut self, route: R);
}

impl<SR, State, Action, Event> RouteStore<SR> for Store<State, Action, Event>
where
    SR: SwitchRoute + 'static,
    Action: RouteAction<SR>,
    State: RouteState<SR>,
    Event: RouteEvent<SR> + PartialEq + StoreEvent + Clone + Hash + Eq,
{
    fn change_route<R: Into<SR>>(&mut self, route: R) {
        self.dispatch(Action::change_route(route));
    }
}
