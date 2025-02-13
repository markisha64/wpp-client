use bson::oid::ObjectId;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::navbar::CHATS;
use shared::api::{
    message::CreateRequest,
    message::GetRequest,
    websocket::{WebsocketClientMessage, WebsocketClientMessageData},
};

pub fn Home() -> Element {
    let mut selected_chat_signal = use_signal::<Option<ObjectId>>(|| None);
    let selected_chat = selected_chat_signal();
    let ws_channel = use_coroutine_handle::<WebsocketClientMessage>();
    let mut current_message_signal = use_signal(|| "".to_string());
    let mut scroll_signal = use_signal(|| ());
    let mut scroll_height_signal = use_signal(|| 0.0);

    let current_message = current_message_signal();

    let chats = CHATS()
        .iter()
        .map(|x| {
            (
                x.name.clone(),
                x.id,
                match Some(x.id) == selected_chat {
                    true => "bg-gray-700",
                    _ => "",
                },
            )
        })
        .collect::<Vec<_>>();

    let selected_chat = selected_chat
        .map(|x| {
            CHATS()
                .iter()
                .find(|chat| chat.id == x)
                .map(|chat| chat.clone())
        })
        .flatten();

    let selected_chat_id = selected_chat.as_ref().map(|x| x.id);

    let _ = use_resource(move || async move {
        let selected_chat_id = selected_chat_signal();
        let selected_chat = selected_chat_id
            .as_ref()
            .map(|x| CHATS().iter().find(|y| y.id == *x).map(|y| y.clone()))
            .flatten();
        let _ = scroll_signal();
        let scroll_height = scroll_height_signal();

        if let Some(chat) = selected_chat {
            let mut eval = document::eval(
                r#"

                const elt = document.getElementById("chat-messages")
                const v = elt
                    ? elt.scrollTop == 0
                    : false
                const vv = elt
                    ? elt.scrollHeight > elt.offsetHeight
                    : false
                const scroll_height = elt
                    ? elt.scrollHeight
                    : 0.0

                dioxus.send(v)
                dioxus.send(scroll_height)
                     
                "#,
            );

            let mut scroll_top = eval.recv::<bool>().await.unwrap();
            let current_scroll_height = eval.recv::<f64>().await.unwrap();

            if scroll_height != current_scroll_height {
                scroll_height_signal.set(current_scroll_height);
                scroll_top = false;
                let _ = document::eval(
                    format!(
                        r#"

                        document.getElementById("chat-messages").scrollTop = {}
                    
                        "#,
                        current_scroll_height - scroll_height
                    )
                    .as_str(),
                );
            }

            if scroll_top {
                let ts = chat
                    .messages
                    .get(0)
                    .map(|x| x.created_at)
                    .unwrap_or(chat.last_message_ts);

                if scroll_top && ts != chat.first_message_ts {
                    ws_channel.send(WebsocketClientMessage {
                        id: Uuid::new_v4(),
                        data: WebsocketClientMessageData::GetMessages(GetRequest {
                            chat_id: chat.id,
                            last_message_ts: ts,
                        }),
                    });
                }
            }
        }
    });

    rsx! {
        aside {
            class: "fixed top-0 left-0 z-40 w-40 h-screen pt-20 transition-transform -translate-x-full border-r sm:translate-x-0 bg-gray-800 border-gray-700",
            div {
                class: "h-full px-3 py-4 overflow-y-auto bg-gray-800",
                ul {
                    class: "space-y-2 font-medium",
                    for (name, id, class) in chats {
                        li {
                            a {
                                class: "items-center p-2 rounded-lg text-white hover:bg-gray-600 group {class}",
                                onclick: move |_| async move {
                                    selected_chat_signal.set(Some(id));
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
        div {
            class: "p-4 sm:ml-40 pb-20 max-h-screen overflow-auto",
            id: "chat-messages",
            onscroll: move |_x| async move {
                scroll_signal.set(());
            },
            if let Some(chat) = selected_chat {
                div {
                    class: "p-4 border-2 border-dashed rounded-lg border-gray-700 mt-14 ",
                    ul {
                        for message in chat.messages {
                            li {
                                div {
                                    class: "w-full text-right text-xs",
                                    if let Some(creator) = message.creator {
                                        "{creator.to_string()}"
                                    } else {
                                        "System"
                                    }
                                }
                                div {
                                    class: "w-full text-left",
                                    "{message.content}"
                                }
                            }
                        }
                    }
                }
            }
        }
        if selected_chat_id.is_some() {
            div {
                class: "fixed bottom-0 left-0 z-30 w-screen h-16 border-t bg-gray-700 border-gray-600 sm:pl-40 p-2",
                div {
                    input {
                        r#type: "text",
                        id: "message",
                        value: "{current_message}",
                        onchange: move |evt| {
                            current_message_signal.set(evt.value());
                        },
                        onkeyup: move |evt| {
                            if evt.key() == Key::Enter && current_message != "" {
                                ws_channel.send(WebsocketClientMessage { id: Uuid::new_v4(), data: WebsocketClientMessageData::NewMessage(CreateRequest {
                                    chat_id: selected_chat_id.unwrap(),
                                    content: current_message.clone()
                                }) });

                                current_message_signal.set("".to_string());
                            }
                        },
                        class: "border text-sm rounded-lg block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"
                    }
                }
            }
        }
    }
}
