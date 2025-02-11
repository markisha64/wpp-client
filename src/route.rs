use dioxus::prelude::*;

use crate::{components::navbar::NavBar, pages::home::Home};

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(NavBar)]
        #[route("/")]
        Home,
}
