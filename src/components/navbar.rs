use dioxus::prelude::*;

use crate::route::Route;

#[component]
pub fn NavBar() -> Element {
    rsx! {
        nav {
            ul {
                li {
                    "login"
                }
                li {
                    "register"
                }
            }
        }
        Outlet::<Route> {}
    }
}
