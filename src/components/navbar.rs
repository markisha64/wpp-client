use dioxus::prelude::*;
use shared::api::user::Claims;

use crate::route::Route;

pub static USER: GlobalSignal<Option<Claims>> = Signal::global(|| None);

#[component]
pub fn NavBar() -> Element {
    let user_r = USER.read();
    let display_login = user_r.is_none();
    let display_name = user_r.clone();

    rsx! {
        nav {
            class: "mx-auto border-gray-200 bg-gray-900 text-sky-100",
            div {
                class: "w-full flex flex-wrap items-center justify-between p-4",
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
                        "{display_name.as_ref().unwrap().user.display_name.clone()}"
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}
