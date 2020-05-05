use crate::{
    bulma::components::{form::field::FieldKey, Select},
    validation::{Validatable, Validation, ValidationErrors, Validator},
};

use yew::{html, Callback, Component, ComponentLink, Html, Properties, ShouldRender};
use yewtil::NeqAssign;

use super::{
    field::{FieldLink, FieldMsg, FormField},
    form::{self, FormFieldLink},
};
use form::FormMsg;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    rc::Rc,
};

#[derive(Debug)]
pub struct SelectField<Value, Key>
where
    Value: Clone + PartialEq + Display + 'static,
    Key: FieldKey + 'static,
{
    pub value: Option<Value>,
    pub validation_errors: ValidationErrors<Key>,
    pub props: Props<Value, Key>,
    link: ComponentLink<Self>,
}

pub enum Msg<Value> {
    Update(Value),
    Validate,
}

pub struct SelectFieldLink<Value, Key>
where
    Value: Clone + PartialEq + Display + 'static,
    Key: FieldKey + 'static,
{
    pub field_key: Key,
    pub link: ComponentLink<SelectField<Value, Key>>,
}

impl<Value, Key> Debug for SelectFieldLink<Value, Key>
where
    Value: Clone + PartialEq + Display + 'static,
    Key: FieldKey + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SelectFieldLink<{0:?}>", self.field_key())
    }
}

impl<T> Into<Msg<T>> for FieldMsg {
    fn into(self) -> Msg<T> {
        match self {
            FieldMsg::Validate => Msg::Validate,
        }
    }
}

impl<Value, Key> FieldLink<Key> for SelectFieldLink<Value, Key>
where
    Value: Clone + PartialEq + Display + 'static,
    Key: FieldKey + 'static,
{
    fn field_key(&self) -> &Key {
        &self.field_key
    }
    fn send_message(&self, msg: FieldMsg) {
        self.link.send_message(msg)
    }
}

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props<Value, Key>
where
    Key: FieldKey + 'static,
    Value: Clone,
{
    pub field_key: Key,
    pub form_link: FormFieldLink<Key>,
    #[prop_or_default]
    pub selected: Option<Value>,
    pub options: Vec<Value>,
    #[prop_or_default]
    pub validator: Validator<Option<Value>, Key>,
    #[prop_or_default]
    pub onchange: Callback<Value>,
}

impl<Value, Key> Component for SelectField<Value, Key>
where
    Value: Clone + PartialEq + ToString + Display + 'static,
    Key: FieldKey + 'static,
{
    type Message = Msg<Value>;
    type Properties = Props<Value, Key>;

    fn create(props: Props<Value, Key>, link: ComponentLink<Self>) -> Self {
        let field_link = SelectFieldLink {
            field_key: props.field_key.clone(),
            link: link.clone(),
        };
        props.form_link.register_field(Rc::new(field_link));

        SelectField {
            value: None,
            validation_errors: ValidationErrors::default(),
            props,
            link,
        }
    }

    fn update(&mut self, msg: Msg<Value>) -> ShouldRender {
        match msg {
            Msg::Update(value) => {
                self.value = Some(value.clone());
                self.props.onchange.emit(value);
                self.props
                    .form_link
                    .send_form_message(FormMsg::FieldValueUpdate(self.props.field_key.clone()));
                self.update(Msg::Validate);
            }
            Msg::Validate => {
                self.validation_errors = self.validate_or_empty();
                self.props
                    .form_link
                    .send_form_message(FormMsg::FieldValidationUpdate(
                        self.props.field_key.clone(),
                        self.validation_errors.clone(),
                    ))
            }
        }
        true
    }

    fn view(&self) -> Html {
        let mut classes = vec![];
        let validation_error =
            if let Some(errors) = self.validation_errors.get(&self.props.field_key) {
                classes.push("is-danger".to_string());
                let error_message = errors.to_string();
                html! {<p class="help is-danger">{ error_message }</p>}
            } else {
                html! {}
            };

        let select_onchange = self.link.callback(Msg::Update);

        html! {
            <div class="field">
                <label class="label">{ self.props.field_key.field_label() }</label>
                <div class="control">
                    <Select<Value>
                        selected=self.props.selected.clone()
                        options=self.props.options.clone()
                        div_classes=classes
                        onchange=select_onchange
                        />
                </div>
                { validation_error }
            </div>
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }
}

impl<Value, Key> Validatable<Key> for SelectField<Value, Key>
where
    Key: FieldKey,
    Value: Clone + PartialEq + Display,
{
    fn validate(&self) -> Result<(), ValidationErrors<Key>> {
        self.props
            .validator
            .validate_value(&self.value, &self.props.field_key)
    }
}

impl<Value, Key> FormField<Key> for SelectField<Value, Key>
where
    Key: FieldKey + 'static,
    Value: Clone + PartialEq + Display,
{
    fn validation_errors(&self) -> &ValidationErrors<Key> {
        &self.validation_errors
    }
    fn field_key(&self) -> &Key {
        &self.props.field_key
    }
}