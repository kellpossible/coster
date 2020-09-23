use crate::{
    state::{
        middleware::localize::LocalizeStore,
        ChangeLastSelectedCurrency, CosterAction, StateCallback, StateStoreRef,
    },
    AppRoute,
};
use anyhow::anyhow;
use commodity::CommodityType;
use costing::Tab;
use form_validation::{
    concat_results, Validatable, Validation, ValidationError, ValidationErrors, Validator, ValidatorFn, AsyncValidator,
};
use log::error;
use std::{fmt::Display, rc::Rc};
use tr::tr;
use uuid::Uuid;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_bulma::components::form::{Form, FormFieldLink, FieldKey, input_field::TextInput, select_field::SelectField, FieldMsg};
use switch_router_middleware::RouteStore;

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
    pub name: String,
    pub working_currency: Option<CommodityType>,
}

impl FormData {
    pub fn create_tab(&self) -> Result<Tab, anyhow::Error> {
        self.validate()
            .map_err(|e| anyhow!("error validating FormData: {}", e))?;
        let working_currency_id = match &self.working_currency {
            Some(working_currency) => working_currency.id,
            None => return Err(anyhow!("empty working_currency in FormData")),
        };
        Ok(Tab::new(
            Uuid::new_v4(),
            self.name.clone(),
            working_currency_id,
            Vec::new(),
            Vec::new(),
        ))
    }
}

impl FormData {
    fn name_validator() -> Validator<String, FormFields> {
        Validator::new().validation(ValidatorFn::new(|name_value: &String, _| {
            if name_value.trim().is_empty() {
                Err(ValidationError::new(FormFields::Name, "coster::costing_tab::field_is_empty")
                    .with_message(|key| tr!("{0} cannot be empty", key)).into())
            } else {
                Ok(())
            }
        }))
    }

    fn working_currency_validator() -> Validator<Option<CommodityType>, FormFields> {
        Validator::new().validation(ValidatorFn::new(|working_currency: &Option<CommodityType>, _| {
            if working_currency.is_none() {
                Err(ValidationError::new(FormFields::WorkingCurrency, "coster::costing_tab::working_currency_not_selected")
                    .with_message(|_| tr!("Please select a working currency")).into())
            } else {
                Ok(())
            }
        }))
    }
}

impl Validatable<FormFields> for FormData {
    fn validate(&self) -> Result<(), ValidationErrors<FormFields>> {
        concat_results(vec![
            Self::name_validator()
                .validate_value(&self.name, &FormFields::Name),
            Self::working_currency_validator()
                .validate_value(&self.working_currency, &FormFields::WorkingCurrency),
        ])
    }
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
    form_is_valid: bool,
    _language_changed_callback: StateCallback,
}

#[derive(Clone)]
pub enum Msg {
    UpdateName(String),
    UpdateWorkingCurrency(CommodityType),
    UpdateFormIsValid(bool),
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

        let mut form_data = FormData::default();
        form_data.working_currency = props.state_store.state().last_selected_currency.clone();

        NewCostingTab {
            form_data,
            props,
            currencies,
            link,
            // Form is displayed as valid (until validations arrive)
            form_is_valid: true,
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
                // Trigger all the fields to display their validations.
                self.form_field_link.send_all_fields_message(FieldMsg::Validate);
                self.form_is_valid = self.form_data.validate().is_ok();

                if self.form_is_valid {
                    self.props.state_store.dispatch(ChangeLastSelectedCurrency {
                        last_selected_currency: self.form_data.working_currency.clone(),
                        write_to_database: true,
                    });
                    let tab = match self.form_data.create_tab() {
                        Ok(tab) => tab,
                        Err(err) => {
                            error!("{}", err);
                            return false;
                        }
                    };

                    self.props.state_store.dispatch(CosterAction::CreateTab {
                        tab: Rc::new(tab),
                        write_to_database: true,
                    });

                    self.props.state_store.change_route(AppRoute::Index);
                }
                true
            }
            Msg::Cancel => {
                self.props.state_store.change_route(AppRoute::Index);
                true
            }
            Msg::LanguageChanged => true,
            Msg::UpdateFormIsValid(is_valid) => {
                self.form_is_valid = is_valid;
                true
            }
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
        let onclick_cancel = self.link.callback(|_| Msg::Cancel);
        let onclick_submit = self.link.callback(|_| Msg::Create);
        let onupdate_working_currency = self.link.callback(Msg::UpdateWorkingCurrency);
        let onupdate_name = self
            .link
            .callback(Msg::UpdateName);

        let onvalidateupdate = self
            .link
            .callback(|errors: ValidationErrors<FormFields>| Msg::UpdateFormIsValid(errors.is_empty()));

        let tab_name_label = tr!("Tab Name");

        let name_validator: AsyncValidator<String, FormFields> = FormData::name_validator().into();
        let working_currency_validator: AsyncValidator<Option<CommodityType>, FormFields> = FormData::working_currency_validator().into();

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
                        form_link = self.form_field_link.clone()
                        onvalidateupdate=onvalidateupdate
                        >
                        <TextInput<FormFields>
                            label = tab_name_label.clone()
                            field_key = FormFields::Name
                            form_link = self.form_field_link.clone()
                            placeholder = tab_name_label
                            validator = name_validator
                            onupdate = onupdate_name
                            />
                        <SelectField<CommodityType, FormFields>
                            label = tr!("Working Currency")
                            field_key = FormFields::WorkingCurrency
                            form_link = self.form_field_link.clone()
                            options = self.currencies.clone()
                            validator = working_currency_validator
                            onupdate = onupdate_working_currency
                            selected = self.form_data.working_currency.clone()
                            />
                    </Form<FormFields>>
                    <div class="field is-grouped">
                        <div class="control">
                            <button
                                class="button is-link"
                                onclick=onclick_submit
                                disabled=!self.form_is_valid>
                                { tr!("Submit") }
                            </button>
                        </div>
                        <div class="control">
                            <button 
                                class="button is-link is-light" 
                                onclick=onclick_cancel>
                                { tr!("Cancel") }
                            </button>
                        </div>
                    </div>
                </div>
            </>
        }
    }
    fn rendered(&mut self, _first_render: bool) {}
}
