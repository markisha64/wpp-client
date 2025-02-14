use dioxus::prelude::*;
use shared::api::user::Claims;

use crate::{route::Route, USER};

#[derive(Clone)]
pub struct Auth {
    pub claims: Claims,
    pub token: String,
}

#[component]
pub fn NavBar() -> Element {
    let user_r = USER();
    let display_login = user_r.is_none();
    let display_name = user_r.clone();

    rsx! {
        nav {
            class: "fixed top-0 z-50 w-full border-b bg-gray-800 border-gray-700 text-white",
            div {
                class: "w-full flex flex-wrap items-center justify-between px-3 py-3 lg:px-5 lg:pl-3",
                "CHET",
                if display_login {
                    div {
                        class: "flex md:order-2 space-x-3 md:space-x-0 rtl:space-x-reverse",
                        button {
                            r#type: "button",
                            class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center bg-blue-600 hover:bg-blue-700 focus:ring-blue-800",
                            Link {
                                to: Route::Login {}, "Login"
                            }
                        }
                    }
                }
                else {
                    div {
                        "{display_name.as_ref().unwrap().claims.user.display_name.clone()}"
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}
