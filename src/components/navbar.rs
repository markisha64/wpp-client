use dioxus::prelude::*;
use shared::api::user::Claims;

use crate::route::Route;

#[derive(Clone)]
pub struct Auth {
    pub claims: Claims,
    pub token: String,
}

#[component]
pub fn NavBar() -> Element {
    rsx! {
        Outlet::<Route> {}
    }
}
