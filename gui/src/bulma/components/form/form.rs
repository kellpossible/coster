use crate::{bulma::components::form::field::FieldKey, validation::ValidationErrors};

use yew::html::Renderable;
use yew::{html, Children, Component, ComponentLink, Html, Properties, ShouldRender, Callback};

use super::{
    field::{FieldLink, FieldMsg},
    SelectField,
};
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};
use tr::tr;

#[derive(Debug)]
pub struct Form<Key>
where
    Key: FieldKey + 'static,
{
    validation_errors: HashMap<Key, ValidationErrors<Key>>,
    pub props: Props<Key>,
    link: ComponentLink<Self>,
}

impl<Key> Form<Key>
where
    Key: FieldKey + 'static,
{
    pub fn validation_errors(&self) -> ValidationErrors<Key> {
        let mut errors = ValidationErrors::default();
        for errors_for_key in self.validation_errors.values() {
            errors.extend(errors_for_key.clone())
        }
        errors
    }
}

#[derive(Clone)]
pub enum FormMsg<Key> {
    FieldValueUpdate(Key),
    FieldValidationUpdate(Key, ValidationErrors<Key>),
    Submit,
    Cancel,
}

#[derive(Clone, Properties, Debug)]
pub struct Props<Key>
where
    Key: FieldKey + 'static,
{
    pub field_link: FormFieldLink<Key>,
    pub children: Children,
    #[prop_or_default]
    pub oncancel: Callback<()>,
    #[prop_or_default]
    pub onsubmit: Callback<()>,
}

impl<Key> Component for Form<Key>
where
    Key: FieldKey + 'static,
{
    type Message = FormMsg<Key>;
    type Properties = Props<Key>;

    fn create(props: Props<Key>, link: ComponentLink<Self>) -> Self {
        props.field_link.register_form(link.clone());
        Form {
            validation_errors: HashMap::new(),
            props,
            link,
        }
    }

    fn update(&mut self, msg: FormMsg<Key>) -> ShouldRender {
        match msg {
            FormMsg::FieldValueUpdate(key) => {}
            FormMsg::Submit => {
                self.props
                    .field_link
                    .send_all_fields_message(FieldMsg::Validate);

                if self.validation_errors.is_empty() {
                    self.props.onsubmit.emit(());
                }
            }
            FormMsg::Cancel => {
                self.props.oncancel.emit(());
            }
            FormMsg::FieldValidationUpdate(key, errors) => {
                self.validation_errors.insert(key, errors);
            }
        }
        true
    }

    fn view(&self) -> Html {
        let onclick_submit = self.link.callback(|_| FormMsg::Submit);
        let onclick_cancel = self.link.callback(|_| FormMsg::Cancel);

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
    Key: FieldKey + 'static,
{
    form_link: Rc<RefCell<Option<ComponentLink<Form<Key>>>>>,
    field_links: Rc<RefCell<HashMap<Key, Rc<dyn FieldLink<Key>>>>>,
}

impl<Key> PartialEq for FormFieldLink<Key>
where
    Key: FieldKey + 'static,
{
    fn eq(&self, other: &FormFieldLink<Key>) -> bool {
        match *self.form_link.borrow() {
            Some(_) => match *other.form_link.borrow() {
                Some(_) => true,
                None => false,
            },
            None => other.form_link.borrow().is_none(),
        }
    }
}

impl<Key> FormFieldLink<Key>
where
    Key: FieldKey + 'static,
{
    pub fn new() -> Self {
        Self {
            form_link: Rc::new(RefCell::new(None)),
            field_links: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn register_form(&self, link: ComponentLink<Form<Key>>) {
        *self.form_link.borrow_mut() = Some(link);
    }

    pub fn register_field(&self, link: Rc<dyn FieldLink<Key>>) {
        self.field_links
            .borrow_mut()
            .insert(link.field_key().clone(), link);
    }

    pub fn send_field_message(&self, key: &Key, msg: FieldMsg) {
        self.field_links
            .borrow()
            .get(key)
            .expect(&format!(
                "expected there to be a FieldLink matching the FieldKey {0:?}",
                key
            ))
            .send_message(msg);
    }

    pub fn send_all_fields_message(&self, msg: FieldMsg) {
        for field in self.field_links.borrow().values() {
            field.send_message(msg);
        }
    }

    pub fn send_form_message(&self, msg: FormMsg<Key>) {
        self.form_link
            .borrow()
            .as_ref()
            .expect("expected form ComponentLink to be registered")
            .send_message(msg);
    }

    pub fn form_link(&self) -> Ref<ComponentLink<Form<Key>>> {
        Ref::map(self.form_link.borrow(), |l| match l {
            Some(l_value) => l_value,
            None => panic!("expected form ComponentLink to be registered"),
        })
    }
}
