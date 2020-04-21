use yew::{html, Component, ComponentLink, Html, ShouldRender, Properties};
use tr::tr;

#[derive(Debug)]
pub struct Navbar {
    props: Props,
    link: ComponentLink<Self>,
}

#[derive(PartialEq, Clone, Properties, Debug)]
pub struct Props {
    #[prop_or_default]
    pub brand: Option<Html>,
}

impl Component for Navbar {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Navbar { 
            props,
            link,
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <nav class="navbar" role="navigation" aria-label="main navigation">
                {
                    if self.props.brand.is_some() {
                        html!{
                            <div class="navbar-brand">
                                { self.props.brand.as_ref().unwrap().clone() }
                                <a role="button" class="navbar-burger" aria-label="menu" aria-expanded="false">
                                    <span aria-hidden="true"></span>
                                    <span aria-hidden="true"></span>
                                    <span aria-hidden="true"></span>
                                </a>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                <div id="navbarBasicExample" class="navbar-menu">
                    <div class="navbar-start">
                        <a class="navbar-item">
                            { tr!("Home") }
                        </a>

                        <a class="navbar-item">
                            { tr!("Documentation") }
                        </a>

                        <div class="navbar-item has-dropdown is-hoverable">
                            <a class="navbar-link">
                            { tr!("More") }
                            </a>

                            <div class="navbar-dropdown">
                                <a class="navbar-item">
                                { tr!("About") }
                                </a>
                                <a class="navbar-item">
                                { tr!("Jobs") }
                                </a>
                                <a class="navbar-item">
                                { tr!("Contact") }
                                </a>
                                <hr class="navbar-divider"/>
                                <a class="navbar-item">
                                { tr!("Report an issue") }
                                </a>
                            </div>
                        </div>
                    </div>

                    <div class="navbar-end">
                        <div class="navbar-item">
                            <div class="buttons">
                            <a class="button is-primary">
                                <strong>{ tr!("Documentation") }</strong>
                            </a>
                            <a class="button is-light">
                            { tr!("Documentation") }
                            </a>
                            </div>
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
}