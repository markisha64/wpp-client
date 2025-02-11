use dioxus::prelude::*;

use crate::{
    components::navbar::NavBar, pages::home::Home, pages::login::Login, pages::register::Register,
};

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(NavBar)]
    #[route("/")]
    Home,
    #[route("/login")]
    Login,
    #[route("/register")]
    Register,

}
