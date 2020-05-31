use crate::bulma::{
    components::{Form, FormFieldLink, InputField, SelectField},
    FieldKey, InputValue,
};
use crate::validation::{ValidationError, Validator};
use crate::{
    state::{
        middleware::{localize::LocalizeStore, route::RouteStore},
        StateCallback, StateStoreRef,
    },
    AppRoute,
};
use commodity::CommodityType;
use log::info;
use std::fmt::Display;
use tr::tr;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(PartialEq, Clone, Copy, Hash, Eq, Debug)]
enum FormFields {
    Name,
    WorkingCurrency,
    // Participant(u32),
}

impl FieldKey for FormFields {}

impl Display for FormFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct FormData {
    name: String,
    working_currency: Option<CommodityType>,
}

impl Default for FormData {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            working_currency: None,
        }
    }
}

pub struct NewCostingTab {
    form_data: FormData,
    props: Props,
    currencies: Vec<CommodityType>,
    link: ComponentLink<Self>,
    form_field_link: FormFieldLink<FormFields>,
    _language_changed_callback: StateCallback,
}

#[derive(Clone)]
pub enum Msg {
    UpdateName(String),
    UpdateWorkingCurrency(CommodityType),
    Create,
    Cancel,
    LanguageChanged,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub state_store: StateStoreRef,
}

impl Component for NewCostingTab {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let mut currencies = commodity::all_iso4217_currencies();
        currencies.sort_by(|a, b| a.id.cmp(&b.id));

        let callback = props
            .state_store
            .subscribe_language_changed(&link, Msg::LanguageChanged);

        NewCostingTab {
            form_data: FormData::default(),
            props,
            currencies,
            link,
            form_field_link: FormFieldLink::new(),
            _language_changed_callback: callback,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::UpdateName(name) => {
                self.form_data.name = name.trim().to_string();
                true
            }
            Msg::UpdateWorkingCurrency(working_currency) => {
                self.form_data.working_currency = Some(working_currency);
                true
            }
            Msg::Create => {
                info!("Creating Tab with data: {:?}", self.form_data);
                true
                // self.props.router.borrow_mut().set_route(AppRoute::Index);
            }
            Msg::Cancel => {
                self.props.state_store.change_route(AppRoute::Index);
                true
            }
            Msg::LanguageChanged => true,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let oncancel = self.link.callback(|_| Msg::Cancel);
        let onsubmit = self.link.callback(|_| Msg::Create);
        let onchange_working_currency = self.link.callback(Msg::UpdateWorkingCurrency);
        let onchange_name = self
            .link
            .callback(|name_value: InputValue| Msg::UpdateName(name_value.into_string()));

        let name_validator: Validator<InputValue, FormFields> =
            Validator::new().validation(|name_value: &InputValue, _| {
                if name_value.as_string().trim().is_empty() {
                    Err(ValidationError::new(FormFields::Name)
                        .with_message(|_| tr!("This field cannot be empty")))
                } else {
                    Ok(())
                }
            });

        let working_currency_validator: Validator<Option<CommodityType>, FormFields> =
            Validator::new().validation(|working_currency: &Option<CommodityType>, _| {
                if working_currency.is_none() {
                    Err(ValidationError::new(FormFields::WorkingCurrency)
                        .with_message(|_| tr!("Please select a working currency")))
                } else {
                    Ok(())
                }
            });

        let tab_name_label = tr!("Tab Name");

        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <div class="level-item">
                            <h3 class="title is-3">{ tr!("New Tab") }</h3>
                        </div>
                    </div>
                </nav>

                <div class="card">
                    <Form<FormFields>
                        field_link = self.form_field_link.clone()
                        oncancel = oncancel
                        onsubmit = onsubmit
                        submit_button_label = tr!("Create")
                        cancel_button_label = tr!("Cancel")>
                        <InputField<FormFields>
                            label = tab_name_label.clone()
                            field_key = FormFields::Name
                            form_link = self.form_field_link.clone()
                            placeholder = tab_name_label
                            validator = name_validator
                            onchange = onchange_name
                            />
                        <SelectField<CommodityType, FormFields>
                            label = tr!("Working Currency")
                            field_key = FormFields::WorkingCurrency
                            form_link = self.form_field_link.clone()
                            options = self.currencies.clone()
                            validator = working_currency_validator
                            onchange = onchange_working_currency
                            />
                    </Form<FormFields>>
                </div>
            </>
        }
    }
}
