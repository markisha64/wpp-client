use bson::oid::ObjectId;
use chrono::prelude::*;
use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use tokio::sync::oneshot;

use shared::api::{
    message::{CreateRequest, GetRequest},
    websocket::{WebsocketClientMessageData, WebsocketServerResData},
};

use crate::{components, CHATS};

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
        let selected_chat_id = selected_chat_id_signal();
        let update_height = update_height_signal();
        let rerender = rerender_signal();

        let selected_chat = chats.into_iter().find(|x| Some(x.id) == selected_chat_id);

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
                        // TODO: update last seen
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
    });

    rerender_signal.set(true);

    rsx! {
        components::sidebar::Sidebar {
            selected_chat_id_signal,
            update_height_signal
        },
        div {
            class: "p-4 sm:ml-40 pb-20 max-h-screen overflow-auto",
            id: "chat-messages",
            onscroll: move |_| {
                update_height_signal.set(UpdateHeight::CheckNeed);
            },
            if let Some(chat) = selected_chat {
                div {
                    class: "p-4 mt-14",
                    ul {
                        for message in chat.messages {
                            li {
                                class: "border-2 border-dashed rounded-lg border-gray-700 m-1 p-1",
                                div {
                                    class: "w-full text-right text-xs",
                                    span {
                                        "{DateTime::<Local>::from(DateTime::<Utc>::from(message.created_at)).format(\"%d/%m/%Y %T\")} "
                                    }
                                    if let Some(creator) = message.creator {
                                        if let Some(name) = chat.users.iter().find(|user| user.id == creator).map(|user| user.display_name.clone()) {
                                            span {
                                                class: "underline",
                                                "{name}"
                                            }
                                        } else {
                                            span {
                                                class: "underline",
                                                "Unknown({creator.to_string()})"
                                            }
                                        }
                                    } else {
                                        span {
                                            class: "underline",
                                            "System"

                                        }
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
                        onkeyup: move |evt: Event<KeyboardData>| {
                            async move {
                                let mut eval = document::eval(
                                    r#"

                                    const elt = document.getElementById("message")
                                    dioxus.send(elt.value)

                                    "#
                                );

                                let current_message = eval.recv::<String>().await.unwrap();

                                if evt.key() == Key::Enter && current_message != "" {
                                    let _ =  ws_request(WebsocketClientMessageData::NewMessage(CreateRequest {
                                        chat_id: selected_chat_id.unwrap(),
                                        content: current_message
                                    })).await.unwrap();

                                    update_height_signal.set(UpdateHeight::GoDown);

                                    let _ = document::eval(
                                        r#"

                                        const elt = document.getElementById("message")
                                        elt.value = ""
                                        
                                        "#
                                    ).await;
                                }
                            }
                        },
                        class: "border text-sm rounded-lg block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"
                    }
                }
            }
        }
    }
}
