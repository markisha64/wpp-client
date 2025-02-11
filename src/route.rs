use dioxus::prelude::*;

use crate::pages::home::Home;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home,
}
