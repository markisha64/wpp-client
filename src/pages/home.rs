use bson::{oid::ObjectId, DateTime};
use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use uuid::Uuid;

use crate::components::navbar::CHATS;
use shared::api::{
    message::CreateRequest,
    message::GetRequest,
    websocket::{WebsocketClientMessage, WebsocketClientMessageData},
};

pub fn Home() -> Element {
    let mut selected_chat_signal = use_signal::<Option<ObjectId>>(|| None);
    let selected_chat = selected_chat_signal.read().clone();
    let ws_channel = use_coroutine_handle::<WebsocketClientMessage>();
    let mut current_message_signal = use_signal(|| "".to_string());

    let current_message = current_message_signal.read().clone();

    let chats = CHATS
        .read()
        .iter()
        .map(|x| {
            (
                x.clone(),
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
            chats
                .iter()
                .find(|(_, id, _)| *id == x)
                .map(|(chat, _, _)| chat.clone())
        })
        .flatten();
    let selected_chat_id = selected_chat.as_ref().map(|x| x.id);

    rsx! {
        aside {
            class: "fixed top-0 left-0 z-40 w-40 h-screen pt-20 transition-transform -translate-x-full border-r sm:translate-x-0 bg-gray-800 border-gray-700",
            div {
                class: "h-full px-3 py-4 overflow-y-auto bg-gray-800",
                ul {
                    class: "space-y-2 font-medium",
                    for (chat, id, class) in chats {
                        li {
                            a {
                                class: "items-center p-2 rounded-lg text-white hover:bg-gray-600 group {class}",
                                onclick: move |_| async move {
                                    selected_chat_signal.set(Some(id));

                                    ws_channel.send(WebsocketClientMessage { id: Uuid::new_v4(), data: WebsocketClientMessageData::GetMessages(GetRequest {
                                        chat_id: id,
                                        last_message_ts: chat.last_message_ts
                                    })});
                                },
                                span {
                                    class: "flex-1 ms-3 whitespace-nowrap",
                                    "{chat.name}"
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(chat) = selected_chat {
            div {
                class: "p-4 sm:ml-40 overflow-y-auto pb-20",
                div {
                    class: "p-4 border-2 border-dashed rounded-lg border-gray-700 mt-14",
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
            div {
                class: "fixed bottom-0 left-0 z-30 w-screen h-16 border-t bg-gray-700 border-gray-600 sm:pl-40 p-2",
                div {
                    class: "overflow-x-auto ",
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
}
