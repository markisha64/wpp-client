use std::str::FromStr;

use anyhow::anyhow;
use bson::oid::ObjectId;
use dioxus::prelude::*;
use shared::api::{
    chat::CreateRequest,
    websocket::{WebsocketClientMessageData, WebsocketServerResData},
};
use tokio::sync::oneshot;

use crate::{components, pages::home::UpdateHeight, route::Route, CHATS, CLAIMS, USER};

#[component]
pub fn Sidebar(
    selected_chat_id_signal: Signal<Option<ObjectId>>,
    update_height_signal: Signal<UpdateHeight>,
) -> Element {
    let mut new_modal_signal = use_signal(|| false);
    let selected_chat_id = selected_chat_id_signal();
    let chats = CHATS();
    let claims = CLAIMS();
    let user = USER();

    let ws_channel = use_coroutine_handle::<(
        WebsocketClientMessageData,
        oneshot::Sender<Result<WebsocketServerResData, String>>,
    )>();

    let ws_request = move |req| -> oneshot::Receiver<_> {
        let (tx, rx) = oneshot::channel();

        ws_channel.send((req, tx));

        rx
    };

    let chats_mapped = chats
        .into_iter()
        .map(|x| {
            (
                x.name,
                x.id,
                match Some(x.id) == selected_chat_id {
                    true => "bg-blue-100 font-semibold",
                    _ => "",
                },
            )
        })
        .collect::<Vec<_>>();

    use_effect(|| {
        document::eval(
            r#"

            let elt = document.getElementById("new-chat-modal")
            window.onclick = function(event) {
                if (event.target === elt) {
                    elt.classList.add("hidden")
                }
            }
                
            "#,
        );
    });

    let logged_in = claims.is_some();
    let new_modal = new_modal_signal();

    rsx! {
        aside {
            class: "w-64 bg-white border-r flex flex-col",
            div {
                class: "p-4 font-bold text-lg border-b",
                "Chats"
            },
            ul {
                class: "flex-1 overflow-y-auto",
                for (name, id, cls) in chats_mapped {
                    li {
                        class: "px-4 py-3 cursor-pointer hover:bg-blue-50 {cls}",
                        onclick: move |_| {
                            update_height_signal.set(UpdateHeight::GoDown);
                            selected_chat_id_signal.set(Some(id));
                        },
                        "{name}"
                    }
                },
                if logged_in {
                    li {
                        class: "px-4 py-3 cursor-pointer hover:bg-blue-50 flex items-center justify-center",
                        onclick: move |_| {
                            new_modal_signal.set(true);
                        },
                        svg {
                            class: "w-5 h-5 text-blue-600",
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 24 24",
                            fill: "none",
                            aria_label: "Create or join chat",
                            path { d: "M12 5v14M5 12h14", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round" }
                        }
                    }
                }
            },
            div {
                class: "p-4 border-t flex flex-col items-center gap-2 text-sm text-gray-500",
                if let Some((_, user)) = claims.zip(user) {
                    components::avatar::Avatar {
                        src: Some(user.profile_image.clone()),
                        alt: user.display_name.clone(),
                        size: components::avatar::Size::Medium,
                    }
                    div {
                        "{user.display_name}"
                    }
                    Link {
                        to: Route::Profile,
                        class: "text-blue-600 hover:text-blue-800 text-xs",
                        "Edit Profile"
                    }
                } else {
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
            }
        },

        if new_modal {
            div {
                class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                div {
                    class: "bg-white rounded-lg shadow-xl w-full max-w-4xl mx-4",
                    div {
                        class: "flex items-center justify-between p-6 border-b",
                        h2 {
                            class: "text-xl font-bold",
                            "New Chat"
                        },
                        button {
                            class: "text-gray-500 hover:text-gray-700 text-2xl",
                            onclick: move |_| {
                                new_modal_signal.set(false);
                            },
                            "×"
                        }
                    },
                    div {
                        class: "flex",
                        div {
                            class: "flex-1 p-6 border-r",
                            h3 {
                                class: "text-lg font-semibold mb-4",
                                "Create New Chat"
                            }
                            form {
                                class: "space-y-4",
                                onsubmit: move |_| {
                                    async move {
                                        let mut eval = document::eval(
                                            r#"

                                            const elt = document.getElementById("chatName")
                                            dioxus.send(elt.value)

                                            "#
                                        );

                                        let current_name = eval.recv::<String>().await.unwrap();

                                        if current_name.len() > 0 {
                                            let new_chat_r = ws_request(WebsocketClientMessageData::CreateChat(CreateRequest {
                                                name: current_name
                                            })).await;

                                            let new_chat = new_chat_r
                                                .map_err(|err| anyhow!(err))
                                                .and_then(|data| match data {
                                                    Ok(WebsocketServerResData::CreateChat(chat)) => Ok(chat),
                                                    Ok(_) => Err(anyhow!("unexpected response")),
                                                    Err(e) => Err(anyhow!(e))
                                                });

                                            if let Ok(chat) = new_chat {
                                                let chat_id = chat.id;
                                                // borrow both here to avoid race conditions
                                                let chats = &mut (*CHATS.write());

                                                let selected_chat_id = &mut (*selected_chat_id_signal.write());

                                                *selected_chat_id = Some(chat_id);

                                                chats.push(chat);
                                                chats.sort_by(|a, b| {
                                                    a.last_message_ts
                                                        .cmp(&b.last_message_ts)
                                                        .reverse()
                                                });

                                                new_modal_signal.set(false);
                                            }
                                        }
                                    }
                                },
                                div {
                                    label {
                                        r#for: "chatName",
                                        class: "block text-sm font-medium text-gray-700 mb-2",
                                        "Chat Name"
                                    }
                                    input {
                                        id: "chatName",
                                        r#type: "text",
                                        class: "w-full border rounded px-3 py-2 focus:outline-none focus:ring focus:ring-blue-500",
                                        placeholder: "Enter chat name...",
                                        required: "true"
                                    }
                                },
                                button {
                                    r#type: "submit",
                                    class: "w-full bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors",
                                    "Create Chat"
                                }
                            }
                        },

                        div {
                            class: "flex-1 p-6",
                            h3 {
                                class: "text-lg font-semibold mb-4",
                                "Join Existing Chat"
                            },
                            form {
                                onsubmit: move |_| {
                                    async move {
                                        let mut eval = document::eval(
                                            r#"

                                            const elt = document.getElementById("chatCode")
                                            dioxus.send(elt.value)

                                            "#
                                        );

                                        let code = eval.recv::<String>().await.unwrap();

                                        let id_r = ObjectId::from_str(code.as_str());

                                        if let Ok(id) = id_r {
                                            let join_r = ws_request(WebsocketClientMessageData::JoinChat(id)).await;

                                            let r = join_r
                                                .map_err(|err| anyhow!(err))
                                                .and_then(|data| match data {
                                                    Ok(WebsocketServerResData::JoinChat(res)) => Ok(res),
                                                    Ok(_) => Err(anyhow!("unexpected response")),
                                                    Err(e) => Err(anyhow!(e))
                                                });

                                            if r.is_ok() {
                                                new_modal_signal.set(false);

                                                let _ = ws_request(WebsocketClientMessageData::GetChats).await;
                                            }
                                        }
                                    }
                                },
                                class: "space-y-4",
                                div {
                                    label {
                                        r#for: "chatCode",
                                        class: "block text-sm font-medium text-gray-700 mb-2",
                                        "Chat Code"
                                    },
                                    input {
                                        id: "chatCode",
                                        r#type: "text",
                                        class: "w-full border rounded px-3 py-2 focus:outline-none focus:ring focus:ring-blue-500",
                                        placeholder: "Enter chat code...",
                                        required: "true"
                                    }
                                },
                                button {
                                    r#type: "submit",
                                    class: "w-full bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors",
                                    "Join Chat"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
