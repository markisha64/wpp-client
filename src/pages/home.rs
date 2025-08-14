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
    let mut show_users_signal = use_signal(|| true);

    let ws_request = move |req| -> oneshot::Receiver<_> {
        let (tx, rx) = oneshot::channel();

        ws_channel.send((req, tx));

        rx
    };

    // dependant signals
    let selected_chat_id = selected_chat_id_signal();
    let chats = CHATS();
    let show_users = show_users_signal();

    let selected_chat = chats
        .iter()
        .find(|x| Some(x.id) == selected_chat_id)
        .map(|x| x.clone());

    let _ = use_resource(move || async move {
        // dependant signals
        let chats = CHATS();
        let user_o = USER();
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

                let scroll_top = scroll_top_v <= 1.0;
                let scroll_bottom = (current_height - scroll_top_v) < 1.0;

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
        div {
            class: "flex h-screen bg-gray-100",
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
                            "{chat.name} ({chat.id.to_string()})"
                        }
                        button {
                            class: "lg:hidden px-3 py-1 border rounded text-sm",
                            onclick: move |_| {
                                show_users_signal.set(!show_users);
                            },
                            match show_users {
                                true => "Hide Users",
                                false => "Show Users"
                            }
                        }
                    },
                    div {
                        class: "flex-1 overflow-y-auto p-4 space-y-4",
                        onscroll: move |_| {
                            update_height_signal.set(UpdateHeight::CheckNeed);
                        },
                        id: "chat-messages",
                        for message in chat.messages {
                            div {
                                class: "flex items-start gap-3",
                                if let Some(creator) = message.creator {
                                    if let Some(chat_user) = chat.users.iter().find(|user| user.id == creator) {
                                        img {
                                            src: chat_user.profile_image.clone(),
                                            alt: chat_user.display_name.clone(),
                                            class: "w-8 h-8 rounded-full"
                                        },
                                        div {
                                            class: "font-semibold text-blue-600",
                                            "{chat_user.display_name}"
                                        }
                                    } else {
                                        img {
                                            src: "",
                                            alt: creator.to_string(),
                                            class: "w-8 h-8 rounded-full"
                                        },
                                        div {
                                            class: "font-semibold text-blue-600",
                                            "Unknown({creator.to_string()})"
                                        }
                                    }
                                } else {
                                    img {
                                        src: "",
                                        alt: "System",
                                        class: "w-8 h-8 rounded-full"
                                    },
                                    div {
                                        class: "font-semibold text-blue-600 underline",
                                        "System"
                                    }
                                }
                                div {
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
                    class: "w-64 bg-white border-l flex-col transition-transform duration-200 ease-in-out hidden lg:flex",
                    div {
                        class: "p-4 font-bold text-lg border-b flex justify-between items-center",
                        "Users",
                        button {
                            class: "md:hidden px-2 py-1 border rounded text-xs",
                            onclick: move |_| {
                                show_users_signal.set(false);
                            },
                            "×"
                        }
                    },
                    ul {
                        class: "flex-1 overflow-y-auto",
                        for user in &chat.users {
                            li {
                                class: "px-4 py-3 border-b last:border-b-0 flex items-center gap-3",
                                img {
                                    src: user.profile_image.clone(),
                                    alt: user.display_name.clone(),
                                    class: "w-8 h-8 rounded-full",
                                    "{user.display_name.clone()}"
                                },
                                {user.display_name.clone()}
                            }
                        }
                    }
                }

                if show_users {
                    div {
                        class: "lg:hidden fixed inset-0 z-50",
                        div {
                            class: "absolute inset-0 bg-black/50",
                            onclick: move |_| {
                                show_users_signal.set(false);
                            }
                        },
                        div {
                            class: "absolute right-0 top-0 h-full w-64 bg-white border-l shadow-lg transform transition-transform duration-200 ease-in-out",
                            div {
                                class: "p-4 font-bold text-lg border-b flex justify-between items-center",
                                "Users",
                                button {
                                    class: "px-2 py-1 border rounded text-xs hover:bg-gray-50",
                                    onclick: move |_| {
                                        show_users_signal.set(false);
                                    },
                                    "×"
                                }
                            },
                            ul {
                                class: "flex-1 overflow-y-auto",
                                for user in &chat.users {
                                    li {
                                        class: "px-4 py-3 border-b last:border-b-0 flex items-center gap-3",
                                        img {
                                            src: user.profile_image.clone(),
                                            alt: user.display_name.clone(),
                                            class: "w-8 h-8 rounded-full",
                                            "{user.display_name.clone()}"
                                        },
                                        {user.display_name.clone()}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
