use dioxus::prelude::*;
use jsonwebtoken::DecodingKey;
use shared::api::user::{AuthResponse, Claims, RegisterRequest};

use crate::{components::navbar::USER, route::Route};

pub fn Register() -> Element {
    let mut email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut display_name = use_signal(|| "".to_string());

    let is_logged_in = USER.read().is_some();
    let navigator = use_navigator();

    use_effect(move || {
        if is_logged_in {
            navigator.replace(Route::Home);
        }
    });

    rsx! {
        div {
            class: "flex w-screen",
            div {
                class: "min-w-sm p-6 border border-gray-200 rounded-lg shadow-sm bg-gray-800 border-gray-700 flex-col mx-auto mt-2",
                h5 {
                    class: "mb-2 text-2xl font-bold tracking-tight text-gray-900 text-white",
                    "Register"
                }
                form {
                    div {
                        label {
                            r#for: "display_name",
                            class: "block mb-2 text-sm font-medium text-gray-900 text-white",
                            "Display name"
                        }
                        input {
                            r#type: "text",
                            id: "display_name",
                            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500",
                            required: 1,
                            onchange: move |evt| {
                                display_name.set(evt.value());
                            }
                        }
                    }
                    div {
                        label {
                            r#for: "email",
                            class: "block mb-2 text-sm font-medium text-gray-900 text-white",
                            "Email"
                        }
                        input {
                            r#type: "text",
                            id: "email",
                            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500",
                            required: 1,
                            onchange: move |evt| {
                                email.set(evt.value());
                            }
                        }
                    }
                    div {
                        label {
                            r#for: "password",
                            class: "block mb-2 text-sm font-medium text-gray-900 text-white",
                            "Password"
                        }
                        input {
                            r#type: "password",
                            id: "password",
                            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500",
                            required: 1,
                            onchange: move |evt| {
                                password.set(evt.value());
                            }
                        }
                    }
                    div {
                        class: "flex flex-wrap justify-between gap-6 align-middle",
                        Link {
                            class: "block text-white text-sm py-2 px-4 underline",
                            to: Route::Login {}, "Or, if you have an account, login!"
                        },
                        button {
                            r#type: "button",
                            class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center bg-blue-600 hover:bg-blue-700 focus:ring-blue-800 m-2",
                            onclick: move |_| async move {
                                let client = reqwest::Client::new();
                                let res = client.post("http://localhost:3030/user/register")
                                    .json(&RegisterRequest{
                                        display_name: display_name.read().clone(),
                                        email: email.read().clone(),
                                        password: password.read().clone(),
                                    })
                                    .send()
                                    .await
                                    .unwrap()
                                    .json::<AuthResponse>()
                                    .await
                                    .unwrap();

                                let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
                                validation.insecure_disable_signature_validation();

                                let key = DecodingKey::from_secret(&[]);
                                let payload = jsonwebtoken::decode::<Claims>(res.token.as_str(), &key, &validation).unwrap();

                                *USER.write() = Some(payload.claims);
                                web_sys::window()
                                    .unwrap()
                                    .local_storage()
                                    .unwrap()
                                    .unwrap()
                                    .set_item("jwt_token", res.token.as_str())
                                    .unwrap();

                                navigator.replace(Route::Home);
                            },
                            "Register"
                        }
                    }
                }
            }
        }
    }
}
