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
        // nav {
        //     class: "fixed top-0 z-50 w-full border-b bg-gray-800 border-gray-700 text-white",
        //     div {
        //         class: "w-full flex flex-wrap items-center justify-between px-3 py-3 lg:px-5 lg:pl-3",
        //         svg {
        //             class: "w-7 h-7",
        //             xmlns: "http://www.w3.org/2000/svg",
        //             view_box: "0 0 200 200",
        //             path {
        //                 d: "M100 30 A 70 70 0 0 0 100 170",
        //                 fill: "none",
        //                 stroke: "#808080",
        //                 stroke_width: 17
        //             }
        //             text {
        //                 x: 130,
        //                 y: 130,
        //                 font_family: "Arial, sans-serif",
        //                 font_size: 75,
        //                 font_weight: "bold",
        //                 fill: "#2563eb",
        //                 text_anchor: "middle",
        //                 "HET"
        //             }
        //         },
        //         if display_login {
        //             div {
        //                 class: "flex md:order-2 space-x-3 md:space-x-0 rtl:space-x-reverse",
        //                 button {
        //                     r#type: "button",
        //                     class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center bg-blue-600 hover:bg-blue-700 focus:ring-blue-800",
        //                     Link {
        //                         to: Route::Login {}, "Login"
        //                     }
        //                 }
        //             }
        //         }
        //         else {
        //             div {
        //                 "{display_name.as_ref().unwrap().claims.user.display_name.clone()}"
        //                 button {
        //                     r#type: "button",
        //                     class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center bg-blue-600 hover:bg-blue-700 focus:ring-blue-800 ml-2",
        //                     onclick: move |_| {
        //                         let user = &mut (*USER.write());
        //                         *user = None;

        //                         let _ = web_sys::window()
        //                             .unwrap()
        //                             .local_storage()
        //                             .unwrap()
        //                             .unwrap()
        //                             .remove_item("jwt_token");
        //                     },
        //                     "Log Out"
        //                 }
        //             }
        //         }
        //     }
        // }
        Outlet::<Route> {}
    }
}
