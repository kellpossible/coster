use crate::{
    bulma::components::{form::field::Field, Select},
    validation::{ValidationErrors, Validatable},
};

use yew::{html, Callback, Component, ComponentLink, Html, Properties, ShouldRender};
use yewtil::NeqAssign;

use std::fmt::Display;

#[derive(Debug)]
pub struct SelectField<F, T>
where
    F: Clone + PartialEq + Field + 'static,
    T: Clone + PartialEq + Display + 'static,
{
    pub value: Option<T>,
    pub validation_errors: ValidationErrors<F>,
    pub props: Props<F, T>,
    link: ComponentLink<Self>,
}

pub enum Msg<T> {
    Update(T),
}

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props<F, T>
where
    F: Clone,
    T: Clone,
{
    pub field: F,
    #[prop_or_default]
    pub selected: Option<T>,
    pub options: Vec<T>,
    #[prop_or_default]
    pub onchange: Callback<T>,
}

impl<F, T> Component for SelectField<F, T>
where
    T: Clone + PartialEq + ToString + Display + 'static,
    F: Clone + PartialEq + Field + 'static,
{
    type Message = Msg<T>;
    type Properties = Props<F, T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        SelectField {
            value: None,
            validation_errors: ValidationErrors::default(),
            props,
            link,
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
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
                    <Select<T>
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

impl<F, T> Validatable<F> for SelectField<F, T>
where
    F: Clone + PartialEq + Field,
    T: Clone + PartialEq + Display,
{
    fn validate(&self) -> Result<(), crate::validation::ValidationErrors<F>> {
        todo!()
    }
}
