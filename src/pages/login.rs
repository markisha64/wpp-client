use anyhow::{anyhow, Context};
use dioxus::prelude::*;
use jsonwebtoken::DecodingKey;
use shared::api::user::{AuthResponse, Claims, LoginRequest};

use crate::{components::navbar::Auth, route::Route, BACKEND_URL, CLAIMS, USER};

pub fn Login() -> Element {
    let mut email_signal = use_signal(|| "".to_string());
    let mut password_signal = use_signal(|| "".to_string());
    let mut error_signal = use_signal(|| Option::<String>::None);

    let email = email_signal();
    let password = password_signal();
    let error = error_signal();

    let is_logged_in = CLAIMS().zip(USER()).is_some();
    let navigator = use_navigator();

    use_effect(move || {
        if is_logged_in {
            navigator.replace(Route::Home);
        }
    });

    rsx! {
        div {
            class: "flex min-h-screen items-center justify-center bg-gray-50",
            div {
                class: "w-full max-w-md p-8 bg-white rounded shadow-md",
                h1 {
                    class: "text-2xl font-bold mb-6 text-center",
                    "Login"
                }
                form {
                    class: "space-y-4",
                    onsubmit: move |_| {
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

                                *CLAIMS.write() = Some(Auth {
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

                                navigator.replace(Route::Home);

                                Ok(())
                            }.await;

                            if let Err(err) = task {
                                error_signal.set(Some(err.to_string()));
                            }
                        });
                    },
                    div {
                        label {
                            class: "block mb-1 font-medium",
                            "Email"
                        },
                        input {
                            r#type: "email",
                            class: "w-full px-3 py-2 border rounded focus:outline-none focus:ring focus:border-blue-300",
                            onchange: move|evt| {
                                email_signal.set(evt.value());
                            },
                            required: "true"
                        }
                    },
                    div {
                        label {
                            class: "block mb-1 font-medium",
                            "Password"
                        },
                        input {
                            r#type: "password",
                            class: "w-full px-3 py-2 border rounded focus:outline-none focus:ring focus:border-blue-300",
                            onchange: move|evt| {
                                password_signal.set(evt.value());
                            },
                            required: "true"
                        }
                    },
                    if let Some(err) = error {
                        div {
                            class: "text-red-500 text-sm",
                            "{err}"
                        }
                    }
                    button {
                        r#type: "submit",
                        class: "w-full bg-blue-600 text-white py-2 rounded hover:bg-blue-700 transition",
                        "Login"
                    }
                }
                p {
                    class: "mt-4 text-center text-sm",
                    "Dont't have an account? ",
                    Link {
                        to: Route::Register {},
                        class: "text-blue-600 hover:underline",
                        "Register"
                    }
                }
            }
        }
    }
}
