use bson::oid::ObjectId;
use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use tokio::sync::oneshot;

use shared::api::{
    message::{CreateRequest, GetRequest},
    websocket::{WebsocketClientMessageData, WebsocketServerResData},
};

use crate::{components, CHATS, USER};

#[derive(Clone)]
pub enum UpdateHeight {
    CheckNeed,
    GoDown,
    GoTo(f64),
}

pub fn Home() -> Element {
    // defined signals
    let selected_chat_id_signal = use_signal::<Option<ObjectId>>(|| None);
    let ws_channel = use_coroutine_handle::<(
        WebsocketClientMessageData,
        oneshot::Sender<Result<WebsocketServerResData, String>>,
    )>();
    let mut update_height_signal = use_signal(|| UpdateHeight::CheckNeed);
    let mut rerender_signal = use_signal(|| false);

    let ws_request = move |req| -> oneshot::Receiver<_> {
        let (tx, rx) = oneshot::channel();

        ws_channel.send((req, tx));

        rx
    };

    // dependant signals
    let selected_chat_id = selected_chat_id_signal();
    let chats = CHATS();

    let selected_chat = chats
        .iter()
        .find(|x| Some(x.id) == selected_chat_id)
        .map(|x| x.clone());

    let _ = use_resource(move || async move {
        // dependant signals
        let chats = CHATS();
        let user_o = USER().map(|x| x.claims.user);
        let selected_chat_id = selected_chat_id_signal();
        let update_height = update_height_signal();
        let rerender = rerender_signal();

        let selected_chat = chats.into_iter().find(|x| Some(x.id) == selected_chat_id);

        if let Some(user) = user_o {
            if let Some(chat) = selected_chat {
                let mut eval = document::eval(
                    r#"

                    const elt = document.getElementById("chat-messages")
                    const v = elt.scrollTop
                    const scroll_height = elt.scrollHeight

                    dioxus.send(v);
                    dioxus.send(scroll_height)
                     
                    "#,
                );

                let scroll_top_v = eval.recv::<f64>().await.unwrap();
                let current_height = eval.recv::<f64>().await.unwrap();

                let scroll_top = scroll_top_v <= 16.0;
                let scroll_bottom = (current_height - scroll_top_v) < 16.0;

                match update_height {
                    UpdateHeight::CheckNeed => {
                        // check if need update
                        if !scroll_top {
                            return;
                        }

                        if scroll_bottom {
                            if let Some(chat_user) = chat.users.iter().find(|x| x.id == user.id) {
                                if chat_user.last_message_seen_ts != chat.last_message_ts {
                                    let _ = ws_request(WebsocketClientMessageData::SetChatRead(
                                        chat.id,
                                    ))
                                    .await;
                                }
                            }
                        }

                        let ts = chat.messages.get(0).map(|x| x.created_at);

                        let rx = ws_request(WebsocketClientMessageData::GetMessages(GetRequest {
                            chat_id: chat.id,
                            last_message_ts: ts,
                        }));

                        let mut messages = match rx.await {
                            Ok(data) => match data {
                                Ok(WebsocketServerResData::GetMessages(messages)) => messages,
                                Err(e) => {
                                    info!("{}", e);

                                    Vec::new()
                                }
                                _ => Vec::new(),
                            },

                            Err(e) => {
                                info!("{}", e);

                                Vec::new()
                            }
                        };

                        if messages.len() == 0 {
                            return;
                        }

                        let mut chats = CHATS.write();
                        let chat_o = chats.iter_mut().find(|x| x.id == chat.id);

                        if let Some(chat_m) = chat_o {
                            messages.extend(chat.messages.into_iter());
                            chat_m.messages = messages;
                            update_height_signal.set(UpdateHeight::GoTo(current_height));
                        }
                    }
                    UpdateHeight::GoDown => {
                        if rerender {
                            let _ = document::eval(
                                r#"

                            const elt = document.getElementById("chat-messages")
                            elt.scrollTop = elt.scrollHeight
         
                            "#,
                            )
                            .await;

                            update_height_signal.set(UpdateHeight::CheckNeed);
                            rerender_signal.set(false);
                        }
                    }
                    UpdateHeight::GoTo(old_height) => {
                        let _ = document::eval(
                            format!(
                                r#"

                                const elt = document.getElementById("chat-messages")
                                elt.scrollTop = {}
                                console.log(elt.scrollTop)
                     
                                "#,
                                current_height - old_height
                            )
                            .as_str(),
                        )
                        .await;

                        update_height_signal.set(UpdateHeight::CheckNeed);
                    }
                }
            }
        }
    });

    rerender_signal.set(true);

    rsx! {
        components::sidebar::Sidebar {
            selected_chat_id_signal,
            update_height_signal
        },
        if let Some(chat) = selected_chat {
            main {
                class: "flex-1 flex flex-col",
                div {
                    class: "flex items-center justify-between p-4 border-b bg-white",
                    div {
                        class: "font-bold text-lg",
                        "{chat.name}({chat.id.to_string()})"
                    }
                    button {
                        class: "md:hidden px-3 py-1 border rounded text-sm",
                        onclick: move |_| {
                            // set show users
                        }
                    }
                },
                div {
                    onscroll: move |_| {
                        update_height_signal.set(UpdateHeight::CheckNeed);
                    },
                    for message in chat.messages {
                        div {
                            // img {
                            //     src: "",
                            //     alt: "user name",
                            //     class: "w-8 h-8 roudned-full"
                            // },
                            div {
                                if let Some(creator) = message.creator {
                                    if let Some(name) = chat.users.iter().find(|user| user.id == creator).map(|user| user.display_name.clone()) {
                                        div {
                                            class: "font-semibold text-blue-600",
                                            "{name}"
                                        }
                                    } else {
                                        div {
                                            class: "font-semibold text-blue-600",
                                            "Unknown({creator.to_string()})"
                                        }
                                    }
                                } else {
                                    div {
                                        class: "font-semibold text-blue-600 underline",
                                        "System"
                                    }
                                }
                                div {
                                    "{message.content}"
                                }
                            }
                        }
                    }
                }
                form {
                    class: "flex gap-2 p-4 border-t bg-white",
                    onsubmit: move |_| {
                        async move {
                            // get input value
                            let mut eval = document::eval(
                                r#"

                                const elt = document.getElementById("message")
                                dioxus.send(elt.value)

                                "#
                            );

                            let current_message = eval.recv::<String>().await.unwrap();

                            if current_message != "" {
                                let _ =  ws_request(WebsocketClientMessageData::NewMessage(CreateRequest {
                                    chat_id: selected_chat_id.unwrap(),
                                    content: current_message
                                })).await.unwrap();

                                update_height_signal.set(UpdateHeight::GoDown);

                                // clear input
                                let _ = document::eval(
                                    r#"

                                    const elt = document.getElementById("message")
                                    elt.value = ""

                                    "#
                                ).await;
                            }
                        }
                    },
                    input {
                        class: "flex-1 border rounded px-3 py-2 focus:outline-none focus:ring",
                        placeholder: "Type a message...",
                        id: "message",
                    },
                    button {
                        class: "bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700",
                        r#type: "submit",
                        "Send"
                    }
                }
            }

            aside {
                // translate logic here
                class: "w-64 bg-white border-l flex-col transition-transform duration-200 ease-in-out hidden md:flex translate-x-full",
                div {
                    class: "p-4 font-bold text-lg border-b flex justify-between items-center",
                    "Users",
                    button {
                        class: "md:hidden px-2 py-1 border rounded text-xs",
                        onclick: |_| {},
                        "×"
                    }
                },
                ul {
                    class: "flex-1 overflow-y-auto",
                    for user in chat.users {
                        // img {
                        //     src: "",
                        //     alt: user.display_name,
                        //     class: "w-8 h-8 rounded-full",
                        //     "{user.display_name}"
                        // },
                        {user.display_name}
                    }
                }
            }
        }
    }
}
