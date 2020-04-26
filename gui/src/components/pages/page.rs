use crate::components::costing_tab_list::CostingTabList;
use crate::components::navbar::Navbar;

use std::cell::RefCell;
use std::{
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

use log::debug;

use commodity::CommodityType;
use costing::{Tab, TabID};
use i18n_embed::{
    language_loader, DefaultLocalizer, I18nEmbed, LanguageRequester, Localizer,
    WebLanguageRequester,
};
use tr::tr;
use yew::{html, Callback, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct Page<Content: Component> {
    rerender: AtomicBool,
    //TODO: do this properly
    // content: Content,
    props: Props,
    link: ComponentLink<Self>,
}

pub enum LanguageMsg {
    Select(unic_langid::LanguageIdentifier),
    Rerender,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub language_requester: Rc<RefCell<dyn LanguageRequester<'static>>>,
    pub localizer: Rc<Box<dyn Localizer<'static>>>,
}

impl<Content> Component for Page<Content>
where
    Content: Component,
{
    type Message = LanguageMsg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Page {
            props,
            link,
            rerender: AtomicBool::new(false),
        }
    }

    fn update(&mut self, msg: LanguageMsg) -> ShouldRender {
        match msg {
            LanguageMsg::Select(language) => {
                self.props
                    .language_requester
                    .borrow_mut()
                    .set_languge_override(Some(language))
                    .unwrap();
                self.props.language_requester.borrow_mut().poll().unwrap();
                self.change(self.props.clone());
                self.rerender.store(true, Ordering::Relaxed);
                true
            }
            LanguageMsg::Rerender => {
                self.rerender.store(false, Ordering::Relaxed);
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let navbar_brand = html! {
            <a class="navbar-item" href="/">
                { tr!("Coster") }
            </a>
        };

        let lang = self.props.localizer.language_loader().current_language();

        // TODO: implement content properly
        html! {
            <>
                <Navbar lang=lang.clone() brand=navbar_brand localizer=self.props.localizer.clone() on_language_change = self.link.callback(|selection| {
                    debug!("GUI Language Selection: {}", selection);
                    LanguageMsg::Select(selection)
                })/>
                <div class="section">
                    <div class="columns">
                        <div class="column is-one-quarter is-desktop"></div>
                        <div class="column">
                            <div class="container">
                                <CostingTabList lang=lang.clone()/>
                            </div>
                        </div>
                        <div class="column is-one-quarter is-desktop"></div>
                    </div>
                </div>
            </>
        }
    }
}
