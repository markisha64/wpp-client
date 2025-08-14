use crate::{route::Route, BACKEND_URL, CLAIMS, USER};
use anyhow::{anyhow, Context};
use base64::{engine::general_purpose, Engine as _};
use dioxus::{document::eval, prelude::*};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    multipart,
};
use shared::api::{
    media::UploadFileResponse,
    user::UpdateRequest,
    websocket::{WebsocketClientMessageData, WebsocketServerResData},
};
use tokio::sync::oneshot;

pub fn Profile() -> Element {
    let user = USER();
    let navigator = use_navigator();
    let ws_channel = use_coroutine_handle::<(
        WebsocketClientMessageData,
        oneshot::Sender<Result<WebsocketServerResData, String>>,
    )>();

    let ws_request = move |req| -> oneshot::Receiver<_> {
        let (tx, rx) = oneshot::channel();

        ws_channel.send((req, tx));

        rx
    };

    use_effect(move || {
        if CLAIMS().is_none() {
            navigator.replace(Route::Login);
        }
    });

    let (user, token) = match user.zip(CLAIMS()) {
        Some((u, a)) => (u, a.token),
        None => return rsx! {},
    };

    let mut message_signal = use_signal(|| Option::<(String, bool)>::None);
    let mut is_loading_signal = use_signal(|| false);

    let mut display_name_signal = use_signal(|| user.display_name);
    let mut profile_image_signal = use_signal(|| (user.profile_image, false, String::new()));

    let message = message_signal();
    let is_loading = is_loading_signal();

    let display_name = display_name_signal();
    let display_name_m = display_name.clone();
    let (profile_image, profile_image_changed, file_name) = profile_image_signal();
    let profile_image_m = profile_image.clone();

    rsx! {
        div {
            class: "min-h-screen bg-gray-50 py-8",
            div {
                class: "max-w-2xl mx-auto px-4",
                div {
                    class: "mb-8",
                    div {
                        class: "flex items-center justify-between",
                        h1 {
                            class: "text-3xl font-bold text-gray-900",
                            "Profile Settings"
                        }
                        Link {
                            to: Route::Home,
                            class: "text-blue-600 hover:text-blue-800 font-medium",
                            "← Back to Chat"
                        }
                    }
                    p {
                        class: "text-gray-600 mt-2",
                        "Update your profile informaation and settings"
                    }
                }

                div {
                    class: "bg-white rounded-lg shadow-md p-6",
                    form {
                        onsubmit: move |_| {
                            to_owned![display_name_m, profile_image_m, token, file_name];

                            is_loading_signal.set(true);
                            message_signal.set(None);

                            spawn(async move {
                                let task: Result<(), anyhow::Error> = async move {
                                    let mut request = UpdateRequest {
                                        display_name: Some(display_name_m),
                                        profile_image: None
                                    };

                                    if profile_image_changed {
                                        let data_o = profile_image_m.strip_prefix("data:image;base64,");

                                        let data = data_o.map(|x| general_purpose::STANDARD.decode(x))
                                            .ok_or(anyhow!("failed to get data"))??;

                                        let file_part = multipart::Part::bytes(data).file_name(file_name);

                                        let form = multipart::Form::new()
                                            .part("file", file_part);

                                        let mut headers = HeaderMap::new();

                                        headers.insert(HeaderName::from_static("authorization"), HeaderValue::from_str(format!("Bearer {}", token).as_str())?);

                                        let client = reqwest::Client::new();
                                        let res = client.post(format!("{}/media/upload", BACKEND_URL))
                                            .multipart(form)
                                            .headers(headers)
                                            .send()
                                            .await?
                                            .error_for_status()?
                                            .json::<UploadFileResponse>()
                                            .await?;

                                        let url = format!("{}{}", BACKEND_URL, res.path);

                                        *profile_image_signal.write() = (url.clone(), false, String::new());
                                        request.profile_image = Some(url);
                                    }

                                    ws_request(WebsocketClientMessageData::ProfileUpdate(request)).await?
                                        .map_err(|e| anyhow!(e))?;

                                    Ok(())
                                }.await;

                                message_signal.write().replace(match task {
                                    Ok(_) => ("Profile updated successfully".to_string(), true),
                                    Err(e) => (e.to_string(), false)
                                });
                                is_loading_signal.set(false);
                            });
                        },
                        div {
                            class: "mb-8",
                            h2 {
                                class: "text-xl font-semibold mb-4",
                                "Profile Image"
                            }
                            div {
                                class: "flex items-center space-x-6",
                                div {
                                    class: "relative",
                                    div {
                                        class: "w-24 h-24 rounded-full overflow-hidden bg-gray-200",
                                        img {
                                            src: profile_image,
                                            alt: "Profile",
                                            width: 96,
                                            height: 96,
                                            class: "w-full h-full object-cover"
                                        }
                                    }
                                    button {
                                        r#type: "button",
                                        onclick: |_| {
                                            eval(r"
                                                let e = document.getElementById('profile-image-input');    
                                                e.click();
                                            ");
                                        },
                                        class: "absolute -bottom-2 -right-2 bg-blue-600 text-white rounded-full p-2 hover:bg-blue-700 transition-colors",
                                        svg {
                                            class: "w-4 h-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: 2,
                                                d: "M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"
                                            }
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: 2,
                                                d: "M15 13a3 3 0 11-6 0 3 3 0 016 0z"
                                            }
                                        }
                                    }
                                }

                                div {
                                    class: "flex-1",
                                    p {
                                        class: "text-sm text-gray-600 mb-2",
                                        "Upload a new profile image. Supported formats: JPG, PNG, GIFUpload "
                                    }
                                    button {
                                        r#type: "button",
                                        onclick: |_| {
                                            eval(r"
                                                let e = document.getElementById('profile-image-input');    
                                                e.click();
                                            ");
                                        },
                                        class: "text-blue-600 hover:text-blue-800 font-medium text-sm",
                                        "Choose Image"
                                    }
                                    input {
                                        r#type: "file",
                                        accept: "image/*",
                                        id: "profile-image-input",
                                        onchange: move |evt| {
                                            async move {
                                                if let Some(file_engine) = &evt.files() {
                                                    let files = file_engine.files();

                                                    let data_o = file_engine.read_file(files[0].as_str()).await;

                                                    if let Some(data) = data_o {
                                                        let encoded = general_purpose::STANDARD.encode(data);

                                                        *profile_image_signal.write() = (format!("data:image;base64,{}", encoded), true, files[0].clone());
                                                    }
                                                }
                                            }
                                        },
                                        class: "hidden"
                                    }
                                }
                            }
                        }

                        div {
                            class: "mb-8",
                            h2 {
                                class: "text-xl font-semibold mb-4",
                                "Display Name"
                            }
                            div {
                                label {
                                    r#for: "display-name",
                                    class: "block text-sm font-medium text-gray-700 mb-2",
                                    "Display Name"
                                }
                                input {
                                    r#type: "text",
                                    id: "display-name",
                                    value: display_name,
                                    onchange: move |evt| {
                                        display_name_signal.set(evt.value());
                                    },
                                    maxlength: 50,
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                }
                                p {
                                    class: "text-xs text-gray-500 mt-1",
                                    "This is the name that will be displayed to other users"
                                }
                            }
                        }

                        if let Some((message, success)) = message {
                            if success {
                                div {
                                    class: "bg-green-50 text-green-800 border border-green-200",
                                    "{message}"
                                }
                            } else {
                                div {
                                    class: "bg-red-50 text-red-800 border border-red-200",
                                    "{message}"
                                }
                            }
                        }

                        div {
                            class: "flex items-center justify-between pt-6 border-t",
                            button {
                                r#type: "button",
                                onclick: move |_| {
                                    *CLAIMS.write() = None;
                                    *USER.write() = None;



                                    let task = move || -> Result<(), anyhow::Error> {
                                        web_sys::window()
                                            .context("failed to get window")?
                                            .local_storage()
                                            .map_err(|_| anyhow!("failed to get local storage"))?
                                            .context("failed to get storage")?
                                            .remove_item("jwt_token")
                                            .map_err(|_| anyhow!("failed to get local storage"))?;

                                        Ok(())
                                    };

                                    match task() {
                                        Ok(_) => {
                                            navigator.replace(Route::Login);
                                        },
                                        Err(e) => {
                                            message_signal.write().replace((e.to_string(), false));
                                        }
                                    }
                                },
                                class: "px-6 py-2 border border-red-300 text-red-700 rounded-md hover:bg-red-50 transition-colors",
                                "Logout"
                            }
                            button {
                                r#type: "submit",
                                disabled: is_loading,
                                class: "px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors",
                                if is_loading {
                                    div {
                                        class: "flex items-center",
                                        svg {
                                            class: "animate-spin -ml-1 mr-2 h-4 w-4 text-white",
                                            fill: "none",
                                            view_box: "0 0 24 24",
                                            circle {
                                                class: "opacity-25",
                                                cx: 12,
                                                cy: 12,
                                                r: 10,
                                                stroke: "currentColor",
                                                stroke_width: 4
                                            }
                                            path {
                                                class: "opacity-75",
                                                fill: "currentColor",
                                                d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                            }
                                        }
                                        "Saving..."
                                    }
                                } else {
                                    "Save Changes"
                                }
                            }
                        }
                    }
                }

                div {
                    class: "mt-6 bg-white rounded-lg shadow-md p-6",
                    h2 {
                        class: "text-xl font-semibold mb-4",
                        "Account Details"
                    }
                    div {
                        class: "space-y-4",
                        div {
                            class: "flex items-center justify-between py-3 border-b",
                            div {
                                h3 {
                                    class: "font-medium",
                                    "Email"
                                },
                                p {
                                    class: "text-sm text-gray-600",
                                    "{user.email}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
