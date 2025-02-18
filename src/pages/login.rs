use anyhow::{anyhow, Context};
use dioxus::prelude::*;
use jsonwebtoken::DecodingKey;
use shared::api::user::{AuthResponse, Claims, LoginRequest};

use crate::{components::navbar::Auth, route::Route, BACKEND_URL, USER};

pub fn Login() -> Element {
    let mut email_signal = use_signal(|| "".to_string());
    let mut password_signal = use_signal(|| "".to_string());
    let toast_handle = use_coroutine_handle::<anyhow::Error>();

    let email = email_signal();
    let password = password_signal();

    let is_logged_in = USER().is_some();
    let navigator = use_navigator();

    use_effect(move || {
        if is_logged_in {
            navigator.replace(Route::Home);
        }
    });

    rsx! {
        div {
            class: "flex w-screen mt-20",
            div {
                class: "min-w-sm p-6 border border-gray-200 rounded-lg shadow-sm bg-gray-800 border-gray-700 flex-col mx-auto mt-2",
                h5 {
                    class: "mb-2 text-2xl font-bold tracking-tight text-gray-900 text-white",
                    "Login"
                }
                form {
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
                                email_signal.set(evt.value());
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
                                password_signal.set(evt.value());
                            }
                        }
                    }
                    div {
                        class: "flex flex-wrap justify-between gap-6 align-middle",
                        Link {
                            class: "block text-white text-sm py-2 px-4 underline",
                            to: Route::Register {}, "Or, alternatively, register!"
                        },
                        button {
                            r#type: "button",
                            class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center bg-blue-600 hover:bg-blue-700 focus:ring-blue-800 float-right m-2",
                            onclick: move |_| {
                                to_owned![email, password];

                                spawn(async move {
                                    let task: Result<(), anyhow::Error> = async move {
                                        let client = reqwest::Client::new();
                                        let res = client.post(format!("{}/user/login", BACKEND_URL))
                                            .json(&LoginRequest {
                                                email,
                                                password,
                                            })
                                            .send()
                                            .await?
                                            .error_for_status()?
                                            .json::<AuthResponse>()
                                            .await?;

                                        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
                                        validation.insecure_disable_signature_validation();

                                        let key = DecodingKey::from_secret(&[]);
                                        let payload = jsonwebtoken::decode::<Claims>(res.token.as_str(), &key, &validation)?;

                                        *USER.write() = Some(Auth {
                                            claims: payload.claims,
                                            token: res.token.clone(),
                                        });

                                        web_sys::window()
                                            .context("failed to get window")?
                                            .local_storage()
                                            .map_err(|_| anyhow!("failed to get local storage"))?
                                            .context("failed to get storage")?
                                            .set_item("jwt_token", res.token.as_str())
                                            .map_err(|_| anyhow!("failed to get local storage"))?;

                                        navigator.replace(Route::Home)
                                            .context("failed navigation")?;

                                        Ok(())
                                    }.await;

                                    if let Err(err) = task {
                                        toast_handle.send(err);
                                    }
                                });
                            },
                            "Login"
                        }
                    }
                }
            }
        }
    }
}
