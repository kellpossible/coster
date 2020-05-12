use crate::Store;
use std::{cell::RefCell, fmt::Debug, rc::Rc};
use yew::{
    html::ChildrenRenderer, ChildrenWithProps, Component, ComponentLink, Properties, Renderable,
};

#[derive(Clone)]
pub struct MapStateToProps<C: Component, State>(
    fn(Rc<State>, &C::Properties) -> Option<C::Properties>,
);

impl<C, State> PartialEq for MapStateToProps<C, State>
where
    C: Component,
{
    fn eq(&self, other: &MapStateToProps<C, State>) -> bool {
        (self.0 as *const ()) == (other.0 as *const ())
    }
}

impl<C, State> MapStateToProps<C, State>
where
    C: Component,
{
    pub fn new(function: fn(Rc<State>, &C::Properties) -> Option<C::Properties>) -> Self {
        Self(function)
    }

    pub fn perform(&self, state: Rc<State>, props: &C::Properties) -> Option<C::Properties> {
        (self.0)(state, props)
    }
}

impl<C: Component, State> Debug for MapStateToProps<C, State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MapStateToProps(function @ {:p})", &self.0)
    }
}

#[derive(Clone, Properties)]
struct Props<C, State, Action, Error>
where
    C: Component + Clone,
    C::Properties: PartialEq,
    State: Clone,
    Action: Clone,
    Error: Clone,
{
    pub map_state_to_props: MapStateToProps<C, State>,
    pub store: Rc<RefCell<Store<State, Action, Error, ()>>>,
    pub children: ChildrenWithProps<C>,
}

impl<C, State, Action, Error> Debug for Props<C, State, Action, Error>
where
    C: Component + Clone,
    C::Properties: PartialEq,
    State: Clone,
    Action: Clone,
    Error: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Provider::Props{{map_state_to_props: {0:?}, store @ {1:p}, children: {2:?}}}",
            self.map_state_to_props, &*self.store, self.children
        )
    }
}

impl<C, State, Action, Error> PartialEq for Props<C, State, Action, Error>
where
    C: Component + Clone,
    C::Properties: PartialEq,
    State: Clone,
    Action: Clone,
    Error: Clone,
{
    fn eq(&self, other: &Props<C, State, Action, Error>) -> bool {
        // TODO: this should also include the children, but it's not currently possible due to https://github.com/yewstack/yew/issues/1216
        Rc::ptr_eq(&self.store, &other.store)
            && self.map_state_to_props == other.map_state_to_props
            && false //false for children not implemented.
    }
}

enum Msg<State> {
    StateUpdate(Rc<State>),
}

struct Provider<C, State, Action, Error, Event>
where
    C: Component + Clone,
    C::Properties: PartialEq,
    State: Clone + 'static,
    Action: Clone + 'static,
    Error: Clone + 'static,
    Event: Clone + 'static,
{
    props: Props<C, State, Action, Error>,
    children: ChildrenWithProps<C>,
    _link: ComponentLink<Provider<C, State, Action, Error, Event>>,
    _callback: crate::EventCallback<State, Error, Event>,
}

impl<C, State, Action, Error, Event> Provider<C, State, Action, Error, Event>
where
    C: Component + Clone,
    C::Properties: PartialEq,
    State: Clone + 'static,
    Action: Clone + 'static,
    Error: Clone + 'static,
    Event: Clone + 'static,
{
    fn update_children_props(
        children: &ChildrenWithProps<C>,
        state: &Rc<State>,
        map_state_to_props: &MapStateToProps<C, State>,
    ) -> Option<ChildrenWithProps<C>> {
        // TODO: only make the children vec if props changed
        // alternatively request an iter_mut implementation for ChildrenWithProps...
        let mut children_vec = children.to_vec();
        let mut child_props_changed = false;

        for child in &mut children_vec {
            match map_state_to_props.perform(state.clone(), &child.props) {
                Some(properties) => {
                    child.props = properties;
                    child_props_changed = true;
                }
                None => {}
            }
        }

        if child_props_changed {
            Some(ChildrenRenderer::new(children_vec))
        } else {
            None
        }
    }
}

impl<C, State, Action, Error, Event> Component for Provider<C, State, Action, Error, Event>
where
    C: Component + Clone,
    C::Properties: PartialEq,
    State: Clone + 'static,
    Action: Clone + 'static,
    Error: Clone + 'static,
    Event: Clone + 'static,
{
    type Message = Msg<State>;
    type Properties = Props<C, State, Action, Error>;

    fn create(props: Props<C, State, Action, Error>, link: yew::ComponentLink<Self>) -> Self {
        let callback = link.callback(|(state, _)| Msg::StateUpdate(state)).into();

        let children = match Self::update_children_props(
            &props.children,
            props.store.borrow().state(),
            &props.map_state_to_props,
        ) {
            None => props.children.clone(),
            Some(children) => children,
        };

        Self {
            props,
            children,
            _link: link,
            _callback: callback,
        }
    }

    fn update(&mut self, msg: Msg<State>) -> yew::ShouldRender {
        match msg {
            Msg::StateUpdate(state) => {
                let result: Option<ChildrenWithProps<C>> = Self::update_children_props(
                    &self.props.children,
                    &state,
                    &self.props.map_state_to_props,
                );
                match result {
                    Some(new_children) => {
                        self.children = new_children;
                        true
                    }
                    None => false,
                }
            }
        }
    }

    fn change(&mut self, props: Props<C, State, Action, Error>) -> yew::ShouldRender {
        if self.props != props {
            // TODO: not currently possible due to https://github.com/yewstack/yew/issues/1216
            // workaround is to assume false.
            // if self.props.children != props.children {
            match Self::update_children_props(
                &props.children,
                props.store.borrow().state(),
                &props.map_state_to_props,
            ) {
                None => self.children = props.children.clone(),
                Some(children) => self.children = children,
            };
            // }

            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> yew::Html {
        self.children.render()
    }
}