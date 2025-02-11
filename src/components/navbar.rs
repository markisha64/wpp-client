use dioxus::prelude::*;
use shared::api::user::Claims;

use crate::route::Route;

static USER: GlobalSignal<Option<Claims>> = Signal::global(|| None);

#[component]
pub fn NavBar() -> Element {
    let user = USER.read();
    let display_login = user.is_none();

    rsx! {
        nav {
            class: "bg-white border-gray-200 dark:bg-gray-900 text-sky-100",
            div {
                class: "max-w-screen-xl flex flex-wrap items-center justify-between mx-auto p-4",
                "CHET",
                if display_login {
                    div {
                        class: "flex md:order-2 space-x-3 md:space-x-0 rtl:space-x-reverse",
                        button {
                            r#type: "button",
                            class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                            Link {
                                to: Route::Login {}, "Login"
                            }
                        }
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}
