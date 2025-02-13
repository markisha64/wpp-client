use bson::oid::ObjectId;
use chrono::prelude::*;
use dioxus::prelude::*;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::components::navbar::CHATS;
use shared::api::{
    message::{CreateRequest, GetRequest},
    websocket::{
        WebsocketClientMessage, WebsocketClientMessageData, WebsocketServerMessage,
        WebsocketServerResData,
    },
};

#[derive(Clone)]
enum UpdateHeight {
    CheckNeed,
    GoDown,
    GoTo(f64),
}

pub fn Home() -> Element {
    let mut selected_chat_signal = use_signal::<Option<ObjectId>>(|| None);
    let selected_chat = selected_chat_signal();
    let ws_channel = use_coroutine_handle::<(
        WebsocketClientMessage,
        oneshot::Sender<WebsocketServerMessage>,
    )>();
    let mut current_message_signal = use_signal(|| "".to_string());
    let mut update_height_signal = use_signal(|| UpdateHeight::CheckNeed);

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
    let selected_chat_2 = selected_chat.clone();

    let selected_chat_id = selected_chat.as_ref().map(|x| x.id);

    let _ = use_resource(move || async move {
        let selected_chat = selected_chat_signal()
            .map(|id| CHATS().iter().find(|x| x.id == id).map(|x| x.clone()))
            .flatten();
        let update_height = update_height_signal();

        if let Some(chat) = selected_chat {
            let mut eval = document::eval(
                r#"

                const elt = document.getElementById("chat-messages")
                const v = elt.scrollTop <= 16
                const scroll_height = elt.scrollHeight

                dioxus.send(v);
                dioxus.send(scroll_height)
                     
                "#,
            );

            let scroll_top = eval.recv::<bool>().await.unwrap();
            let current_height = eval.recv::<f64>().await.unwrap();

            match update_height {
                UpdateHeight::CheckNeed => {
                    // check if need update
                    if !scroll_top {
                        return;
                    }

                    let ts = chat
                        .messages
                        .get(0)
                        .map(|x| x.created_at)
                        .unwrap_or(chat.last_message_ts);

                    let (tx, rx) = oneshot::channel();

                    ws_channel.send((
                        WebsocketClientMessage {
                            id: Uuid::new_v4(),
                            data: WebsocketClientMessageData::GetMessages(GetRequest {
                                chat_id: chat.id,
                                last_message_ts: ts,
                            }),
                        },
                        tx,
                    ));

                    let mut messages = match rx.await {
                        Ok(WebsocketServerMessage::RequestResponse { id, data, error }) => {
                            match data {
                                Some(WebsocketServerResData::GetMessages(messages)) => messages,
                                _ => Vec::new(),
                            }
                        }

                        _ => Vec::new(),
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
                    let _ = document::eval(
                        r#"

                        const elt = document.getElementById("chat-messages")
                        elt.scrollTop = elt.scrollHeight
                     
                        "#,
                    )
                    .await;

                    update_height_signal.set(UpdateHeight::CheckNeed);
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
                                    onclick: move |_| {
                                        async move {
                                            selected_chat_signal.set(Some(id));
                                            update_height_signal.set(UpdateHeight::GoDown);
                                        }
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
                onscroll: move |_| {
                    update_height_signal.set(UpdateHeight::CheckNeed);
                },
                if let Some(chat) = selected_chat_2 {
                    div {
                        class: "p-4 mt-14",
                        ul {
                            for message in chat.messages {
                                li {
                                    class: "border-2 border-dashed rounded-lg border-gray-700 m-1 p-1",
                                    div {
                                        class: "w-full text-right text-xs",
                                        if let Some(creator) = message.creator {
                                            span {
                                                "{DateTime::<Local>::from(DateTime::<Utc>::from(message.created_at)).format(\"%d/%m/%Y %T\")} "
                                            }
                                            span {
                                                class: "underline",
                                                "{creator.to_string()}"
                                            }
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
                                to_owned![current_message];

                                async move {
                                    if evt.key() == Key::Enter && current_message != "" {
                                        let (tx, rx) = oneshot::channel();

                                        ws_channel.send((WebsocketClientMessage { id: Uuid::new_v4(), data: WebsocketClientMessageData::NewMessage(CreateRequest {
                                            chat_id: selected_chat_id.unwrap(),
                                            content: current_message
                                        }) }, tx));

                                        let _ = rx.await.unwrap();

                                        update_height_signal.set(UpdateHeight::GoDown);
                                        current_message_signal.set("".to_string());
                                    }
                                }
                            }
    ,
                            class: "border text-sm rounded-lg block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"
                        }
                    }
                }
            }
        }
}
