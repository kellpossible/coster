use crate::{
    bulma::components::{form::field::Field, Select},
    validation::{Validatable, ValidationErrors, Validator, Validation},
};

use yew::{html, Callback, Component, ComponentLink, Html, Properties, ShouldRender};
use yewtil::NeqAssign;

use std::fmt::Display;

#[derive(Debug)]
pub struct SelectField<Value, Key = &'static str>
where
    Value: Clone + PartialEq + Display + 'static,
    Key: Clone + PartialEq + Field + Display + 'static,
{
    pub value: Option<Value>,
    pub validation_errors: ValidationErrors<Key>,
    pub props: Props<Value, Key>,
    link: ComponentLink<Self>,
}

pub enum Msg<T> {
    Update(T),
}

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props<Value, Key>
where
    Key: Clone,
    Value: Clone,
{
    pub field: Key,
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
    Key: Clone + PartialEq + Display + Field + 'static,
{
    type Message = Msg<Value>;
    type Properties = Props<Value, Key>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        SelectField {
            value: None,
            validation_errors: ValidationErrors::default(),
            props,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Update(value) => {
                self.value = Some(value);
                self.validation_errors = self.validate_or_empty();
            }
        }
        true
    }

    fn view(&self) -> Html {
        let mut classes = vec![];
        let validation_error = if let Some(errors) = self.validation_errors.get(&self.props.field) {
            classes.push("is-danger".to_string());
            let error_message = errors.to_string();
            html! {<p class="help is-danger">{ error_message }</p>}
        } else {
            html! {}
        };

        html! {
            <div class="field">
                <label class="label">{ self.props.field.label() }</label>
                <div class="control">
                    <Select<Value>
                        selected=self.props.selected.clone()
                        options=self.props.options.clone()
                        div_classes=classes
                        onchange=self.props.onchange.clone()
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
    Key: Clone + Display + PartialEq + Field,
    Value: Clone + PartialEq + Display,
{
    fn validate(&self) -> Result<(), ValidationErrors<Key>> {
        self.props.validator.validate_value(&self.value, &self.props.field)
    }
}
