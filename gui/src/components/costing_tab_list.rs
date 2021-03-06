use crate::state::middleware::localize::LocalizeStore;
use crate::{
    state::{CosterEvent, StateCallback, StateStoreRef},
    AppRoute,
};

use tr::tr;
use yew::MouseEvent;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use switch_router_middleware::RouteStore;

pub struct CostingTabList {
    props: Props,
    link: ComponentLink<Self>,
    _language_changed_callback: StateCallback,
    _tabs_changed_callback: StateCallback,
}

#[derive(Clone)]
pub enum Msg {
    NewCostingTab,
    LanguageChanged,
    TabsChanged,
    TestGraphQL,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub state_store: StateStoreRef,
}

impl Component for CostingTabList {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let language_changed_callback = props
            .state_store
            .subscribe_language_changed(&link, Msg::LanguageChanged);

        let tabs_changed_callback = link.callback(|(_store, _event)| Msg::TabsChanged).into();

        props
            .state_store
            .subscribe_event(&tabs_changed_callback, CosterEvent::TabsChanged);

        CostingTabList {
            props,
            link,
            _language_changed_callback: language_changed_callback,
            _tabs_changed_callback: tabs_changed_callback,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NewCostingTab => {
                self.props.state_store.change_route(AppRoute::NewCostingTab);
                true
            }
            Msg::LanguageChanged => true,
            Msg::TabsChanged => true,
            Msg::TestGraphQL => {
                crate::graphql::addtest::add_test();
                false
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
        let state = self.props.state_store.state();
        let new_tab_handler = self.link.callback(|_msg: MouseEvent| Msg::NewCostingTab);
        let test_graphql_handler = self.link.callback(|_msg: MouseEvent| Msg::TestGraphQL);

        let tabs_html_iter = state.tabs.iter().map(|tab| {
            html! {
                <tr>
                    <td>{ &tab.name }</td>
                </tr>
            }
        });

        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <div class="level-item">
                            <h3 class="title is-3">{ tr!("Your Tabs") }</h3>
                        </div>
                    </div>
                    <div class="level-right">
                        <div class="level-item">
                            <button class="button is-success" onclick = new_tab_handler>{ tr!("New Tab") }</button>
                        </div>
                    </div>
                </nav>
                <table class="table is-striped is-fullwidth">
                    <thead>
                        <tr>
                            <th>{ tr!("Tab Name") }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for tabs_html_iter }
                    </tbody>
                </table>
                <button class="button is-success" onclick = test_graphql_handler>{ "Test GraphQL" }</button>
            </>
        }
    }
}
