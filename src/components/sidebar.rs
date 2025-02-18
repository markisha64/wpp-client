use std::str::FromStr;

use anyhow::anyhow;
use bson::oid::ObjectId;
use dioxus::prelude::*;
use shared::api::{
    chat::CreateRequest,
    websocket::{WebsocketClientMessageData, WebsocketServerResData},
};
use tokio::sync::oneshot;

use crate::{pages::home::UpdateHeight, CHATS, USER};

#[component]
pub fn Sidebar(
    selected_chat_id_signal: Signal<Option<ObjectId>>,
    update_height_signal: Signal<UpdateHeight>,
) -> Element {
    let selected_chat_id = selected_chat_id_signal();
    let chats = CHATS();
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
                    true => "bg-gray-700",
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

    rsx! {
        aside {
            class: "fixed top-0 left-0 z-40 w-40 h-screen pt-16 transition-transform -translate-x-full border-r sm:translate-x-0 bg-gray-800 border-gray-700",
            if user.is_some() {
                div {
                    class: "h-full px-3 py-4 overflow-y-auto bg-gray-800",
                    ul {
                        class: "space-y-2 font-medium",
                        if let Some(chat_id) = selected_chat_id {
                            li {
                                class: "text-white text-[0.6rem] text-center",
                                "{chat_id}"
                            }
                        }
                        li {
                            class: "items-center p-2 rounded-lg text-white hover:bg-gray-600 group",
                            onclick: move |_| {
                                async move {
                                    let _ = document::eval("document.getElementById('new-chat-modal').classList.remove('hidden')").await;
                                }
                            },
                            span {
                                class: "flex-1 text-sm ms-3 whitespace-nowrap",
                                "Create/Join Chat"
                            }
                        },
                    }
                    ul {
                        class: "pt-4 mt-4 space-y-2 font-medium border-t border-gray-200 dark:border-gray-700",
                        for (name, id, class) in chats_mapped {
                            li {
                                a {
                                    class: "items-center p-2 rounded-lg text-white hover:bg-gray-600 group {class}",
                                    onclick: move |_| {
                                        update_height_signal.set(UpdateHeight::GoDown);
                                        selected_chat_id_signal.set(Some(id));
                                    },
                                    span {
                                        class: "flex-1 ms-3 whitespace-nowrap",
                                        "{name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        div {
            id: "new-chat-modal",
            class: "fixed inset-0 gray-800 bg-opacity-50 flex items-center justify-center hidden",
            div {
                class: "bg-gray-800 p-6 rounded-lg w-96",
                div {
                    class: "flex flex-col items-center",
                    h2 {
                        class: "text-xl font-semibold text-white",
                        "New Chat"
                    }
                    div {
                        div {
                            label {
                                r#for: "chat_name",
                                class: "block mb-2 text-sm font-medium text-white",
                                "Chat Name"
                            }
                            input {
                                r#type: "text",
                                id: "chat_name",
                                class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500",
                                required: 1,
                            }
                        }
                        button {
                            r#type: "button",
                            class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center bg-blue-600 hover:bg-blue-700 focus:ring-blue-800 float-right m-2",
                            onclick: move |_| {
                                async move {
                                    let mut eval = document::eval(
                                        r#"

                                        const elt = document.getElementById("chat_name")
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

                                            let _ = document::eval("document.getElementById('new-chat-modal').classList.add('hidden')").await;
                                        }
                                    }
                                }
                            },
                            "Create"
                        }
                    }
                    div {
                        class: "pt-4 mt-4 border-t border-gray-700",
                        h2 {
                            class: "text-xl font-semibold text-white",
                            "Or join via code"
                        }
                        div {
                            label {
                                r#for: "join_code",
                                class: "block mb-2 text-sm font-medium text-white",
                                "Code"
                            }
                            input {
                                r#type: "text",
                                id: "join_code",
                                class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500",
                                required: 1,
                            }
                        }
                        button {
                            r#type: "button",
                            class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center bg-blue-600 hover:bg-blue-700 focus:ring-blue-800 float-right m-2",
                            onclick: move |_| {
                                async move {
                                    let mut eval = document::eval(
                                        r#"

                                        const elt = document.getElementById("join_code")
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
                                            let _ = document::eval("document.getElementById('new-chat-modal').classList.add('hidden')").await;
                                        }
                                    }
                                }
                            },
                            "Join"
                        }
                    }
                }
            }
        }
    }
}
