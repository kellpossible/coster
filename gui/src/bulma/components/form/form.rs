use crate::{bulma::components::form::field::FieldKey, validation::ValidationErrors};

use yew::html::Renderable;
use yew::{html, Children, Component, ComponentLink, Html, Properties, ShouldRender};

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    rc::Rc,
};
use tr::tr;

#[derive(Debug)]
pub struct Form<Key>
where
    Key: Clone + PartialEq + FieldKey + Display + Hash + Eq + 'static,
{
    validation_errors: HashMap<Key, ValidationErrors<Key>>,
    pub props: Props<Key>,
    link: ComponentLink<Self>,
}

impl<Key> Form<Key>
where
    Key: Clone + PartialEq + FieldKey + Display + Hash + Eq + 'static,
{
    pub fn validation_errors(&self) -> ValidationErrors<Key> {
        let mut errors = ValidationErrors::default();
        for errors_for_key in self.validation_errors.values() {
            errors.extend(errors_for_key.clone())
        }
        errors
    }
}

pub enum Msg<Key> {
    FieldUpdate(Key, ValidationErrors<Key>),
    Submit,
    Cancel,
}

#[derive(Clone, Properties, Debug)]
pub struct Props<Key>
where
    Key: Clone + PartialEq + FieldKey + Display + Hash + Eq + 'static,
{
    pub field_link: FormFieldLink<Key>,
    pub children: Children,
}

impl<Key> Component for Form<Key>
where
    Key: Clone + PartialEq + Display + FieldKey + Hash + Eq + 'static,
{
    type Message = Msg<Key>;
    type Properties = Props<Key>;

    fn create(props: Props<Key>, link: ComponentLink<Self>) -> Self {
        props.field_link.register(link.clone());
        Form {
            validation_errors: HashMap::new(),
            props,
            link,
        }
    }

    fn update(&mut self, msg: Msg<Key>) -> ShouldRender {
        match msg {
            Msg::FieldUpdate(key, errors) => {
                self.validation_errors.insert(key, errors);
            }
            Msg::Submit => {
                //TODO: trigger validation on the fields using the link? Might need to
                // modify the link to make this work.
            }
            Msg::Cancel => {}
        }
        true
    }

    fn view(&self) -> Html {
        let onclick_submit = self.link.callback(|_| Msg::Submit);
        let onclick_cancel = self.link.callback(|_| Msg::Cancel);

        // TODO: extract the buttons to their own components
        html! {
            <>
                { self.props.children.render() }
                <div class="field is-grouped">
                    <div class="control">
                        <button
                            class="button is-link"
                            onclick=onclick_submit
                            disabled=!self.validation_errors().is_empty()>
                            { tr!("Create") }
                        </button>
                    </div>
                    <div class="control">
                        <button class="button is-link is-light" onclick=onclick_cancel>{ tr!("Cancel") }</button>
                    </div>
                </div>
            </>
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        false
    }
}

#[derive(Clone, Debug)]
pub struct FormFieldLink<Key = &'static str>
where
    Key: Clone + PartialEq + FieldKey + Display + Hash + Eq + 'static,
{
    link: Rc<RefCell<Option<ComponentLink<Form<Key>>>>>,
}

impl<Key> PartialEq for FormFieldLink<Key>
where
    Key: Clone + PartialEq + FieldKey + Display + Hash + Eq + 'static,
{
    fn eq(&self, other: &FormFieldLink<Key>) -> bool {
        match *self.link.borrow() {
            Some(_) => match *other.link.borrow() {
                Some(_) => true,
                None => false,
            },
            None => other.link.borrow().is_none(),
        }
    }
}

impl<Key> FormFieldLink<Key>
where
    Key: Clone + PartialEq + FieldKey + Display + Hash + Eq + 'static,
{
    pub fn new() -> Self {
        Self {
            link: Rc::new(RefCell::new(None)),
        }
    }

    pub fn register(&self, link: ComponentLink<Form<Key>>) {
        *self.link.borrow_mut() = Some(link);
    }

    pub fn link(&self) -> Ref<ComponentLink<Form<Key>>> {
        Ref::map(self.link.borrow(), |l| match l {
            Some(l_value) => l_value,
            None => panic!("expected link to be registered"),
        })
    }
}
