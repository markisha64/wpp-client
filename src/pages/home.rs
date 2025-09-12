use std::collections::HashMap;

use bson::oid::ObjectId;
use dioxus::prelude::*;
use dioxus_logger::tracing::{self, info};
use tokio::sync::oneshot;

use shared::api::{
    message::{CreateRequest, GetRequest},
    websocket::{MediaSoupMessage, WebsocketClientMessageData, WebsocketServerResData},
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
    let mut show_users_signal = use_signal(|| true);

    let mut show_media_signal = use_signal(|| (false, None));

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
        .map(|x| {
            (
                x.clone(),
                x.users
                    .iter()
                    .map(|x| (x.id.to_string(), x.display_name.clone()))
                    .collect::<HashMap<_, _>>(),
            )
        });

    let _ = use_effect(move || {
        // dependant signals
        let chats = CHATS();
        let user_o = USER();
        let selected_chat_id = selected_chat_id_signal();
        let update_height = update_height_signal();

        spawn(async move {
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
                                if let Some(chat_user) = chat.users.iter().find(|x| x.id == user.id)
                                {
                                    if chat_user.last_message_seen_ts != chat.last_message_ts {
                                        let _ = ws_request(
                                            WebsocketClientMessageData::SetChatRead(chat.id),
                                        )
                                        .await;
                                    }
                                }
                            }

                            let ts = chat.messages.get(0).map(|x| x.created_at);

                            let rx =
                                ws_request(WebsocketClientMessageData::GetMessages(GetRequest {
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
    });

    let (show_media_v, vc_chat) = show_media_signal();
    let show_media = show_media_v && vc_chat == selected_chat_id;

    let media_sources_class = match show_media {
        true => "",
        false => "hidden",
    };
    let messages_class = match show_media {
        true => "basis-[40%]",
        false => "flex-1",
    };

    rsx! {
        div {
            class: "flex h-screen bg-gray-100",
            components::sidebar::Sidebar {
                selected_chat_id_signal,
                update_height_signal
            },
            if let Some((chat, user_map)) = selected_chat {
                main {
                    class: "flex-1 flex flex-col",
                    div {
                        class: "flex items-center justify-between p-4 border-b bg-white",
                        div {
                            class: "font-bold text-lg",
                            "{chat.name}"
                        }
                        div {
                            if !show_media {
                                button {
                                    class: "px-3 py-1 bg-green-600 text-white rounded text-sm hover:bg-green-700 m-2",
                                    onclick: move |_| {
                                        to_owned![user_map];

                                        async move {
                                            let res = ws_request(WebsocketClientMessageData::MS(MediaSoupMessage::SetRoom(chat.id)));

                                            match res.await {
                                                Ok(_) => {
                                                    *show_media_signal.write() = (true, selected_chat_id);

                                                    let js = document::eval(r"
                                                        const value = await dioxus.recv()

                                                        document.getElementById('media-sources').dataset.users = JSON.stringify(value)
                                                    ");

                                                    let _ = js.send(user_map);
                                                },
                                                Err(e) => tracing::error!("{}", e)
                                            };
                                        }
                                    },
                                    "Join Call"
                                }
                            }
                            button {
                                class: "lg:hidden px-3 py-1 border rounded text-sm m-2",
                                onclick: move |_| {
                                    show_users_signal.set(!show_users);
                                },
                                match show_users {
                                    true => "Hide Users",
                                    false => "Show Users"
                                }
                            }
                            button {
                                class: "px-3 py-1 border rounded text-sm bg-gray-200 hover:bg-gray-100 m-2",
                                id: "copy-code-button",
                                onclick: move |_| {
                                    let eval = document::eval(r#"
                                        const msg = await dioxus.recv();   

                                        await navigator.clipboard.writeText(msg);

                                        const elt = document.getElementById("copy-code-button")

                                        elt.classList.remove("animate-copyCodeSuccess")
                                        void elt.offsetWidth;
                                        elt.classList.add("animate-copyCodeSuccess")
                                    "#);

                                    let _ = eval.send(chat.id.to_string());
                                },
                                "Copy Code"
                            }
                        }
                    },
                    div {
                        class: "basis-[60%] relative bg-gray-800 text-white overflow-hidden {media_sources_class}",
                        div {
                            class: "h-full w-full grid grid-cols-[repeat(auto-fit,minmax(140px,1fr))] sm:grid-cols-[repeat(auto-fit,minmax(180px,1fr))] md:grid-cols-[repeat(auto-fit,minmax(240px,1fr))] auto-rows-max items-start justify-items-center gap-2 md:gap-4 p-4 overflow-auto",
                            id: "media-sources",
                            figure {
                                class: "w-full max-w-[480px] min-w-0 rounded-xl bg-black/50",
                                figcaption {
                                    class: "mt-2 text-center text-sm text-white/70",
                                    "You"
                                }
                                div {
                                    class: "relative rounded-xl overflow-hidden bg-black ring-1 ring-white/10 shadow-lg",
                                    div {
                                        class: "absolute inset-0 z-0 flex items-center justify-center text-white/60 select-none pointer-events-none",
                                        svg {
                                            class: "w-16 h-16 md:w-20 md:h-20",
                                            xmlns: "http://www.w3.org/2000/svg",
                                            view_box: "0 0 24 24",
                                            fill: "currentColor",
                                            path { d: "M12 12c2.761 0 5-2.239 5-5s-2.239-5-5-5-5 2.239-5 5 2.239 5 5 5zm0 2c-4.418 0-8 2.239-8 5v1h16v-1c0-2.761-3.582-5-8-5z" }
                                        }
                                    }
                                    video {
                                        class: "relative z-10 block w-full aspect-video object-cover",
                                        id: "preview-send",
                                        muted: true,
                                        controls: false,
                                    }
                                }
                            }
                        }
                        // Controls bar moved to parent container (outside of #media-sources)
                        div {
                            class: "sticky bottom-0 z-30 w-full flex justify-center mt-2",
                            div {
                                class: "flex items-center gap-3 p-2 rounded-full bg-black/40 backdrop-blur ring-1 ring-white/10",
                                // Leave Call
                                button {
                                    class: "w-12 h-12 rounded-full bg-red-600 hover:bg-red-700 text-white flex items-center justify-center shadow",
                                    aria_label: "Leave call",
                                    onclick: move |_| {
                                        async move {
                                            let res = ws_request(WebsocketClientMessageData::MS(MediaSoupMessage::LeaveRoom));

                                            match res.await {
                                                Ok(_) => {
                                                    *show_media_signal.write() = (false, selected_chat_id);
                                                },
                                                Err(e) => tracing::error!("{}", e)
                                            };
                                        }
                                    },
                                    svg {
                                        class: "w-6 h-6",
                                        xmlns: "http://www.w3.org/2000/svg",
                                        view_box: "0 0 24 24",
                                        fill: "currentColor",
                                        path { d: "M3.51 14.88c-.31-.31-.48-.74-.48-1.18 0-.45.18-.88.5-1.19 4.55-4.51 11.9-4.51 16.45 0 .32.31.5.74.5 1.19 0 .44-.17.87-.48 1.18l-1.24 1.24c-.66.66-1.73.62-2.34-.1l-1.02-1.21c-.51-.6-.56-1.47-.12-2.12l.23-.35c-2.33-.94-4.97-.94-7.3 0l.23.35c.44.65.39 1.52-.12 2.12l-1.02 1.21c-.61.72-1.68.76-2.34.1L3.51 14.88z" }
                                    }
                                }
                                // Mute/Unmute
                                button {
                                    class: "w-12 h-12 rounded-full bg-gray-700 hover:bg-gray-600 text-white flex items-center justify-center shadow",
                                    aria_label: "Mute microphone",
                                    onclick: move |_| {
                                    },
                                    svg {
                                        class: "w-6 h-6",
                                        xmlns: "http://www.w3.org/2000/svg",
                                        view_box: "0 0 24 24",
                                        fill: "currentColor",
                                        path { d: "M12 14a3 3 0 0 0 3-3V6a3 3 0 1 0-6 0v5a3 3 0 0 0 3 3zm5-3a5 5 0 0 1-10 0H5a7 7 0 0 0 6 6.92V21h2v-3.08A7 7 0 0 0 19 11h-2z" }
                                    }
                                }
                                // Hide/Show Video
                                button {
                                    class: "w-12 h-12 rounded-full bg-gray-700 hover:bg-gray-600 text-white flex items-center justify-center shadow",
                                    aria_label: "Hide video",
                                    onclick: move |_| {},
                                    svg {
                                        class: "w-6 h-6",
                                        xmlns: "http://www.w3.org/2000/svg",
                                        view_box: "0 0 24 24",
                                        fill: "currentColor",
                                        path { d: "M2.81 2.81 1.39 4.22l3.03 3.03C3.57 8.32 2.27 9.58 1 11c2.73 3.18 6.11 5 11 5 1.47 0 2.82-.18 4.04-.51l3.74 3.74 1.41-1.41L2.81 2.81zM12 8c1.1 0 2 .9 2 2 0 .36-.1.69-.27.98l-2.71-2.71c.29-.17.62-.27.98-.27zm9-2-5 3v2.09l-2-2V7c0-1.1-.9-2-2-2-1.09 0-1.99.89-2 1.98V7.1l-1.94-1.94C8.77 3.88 10.25 3 12 3c2.76 0 5 2.24 5 5v.18L21 11V6z" }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "{messages_class} overflow-y-auto p-4 space-y-4",
                        onscroll: move |_| {
                            update_height_signal.set(UpdateHeight::CheckNeed);
                        },
                        id: "chat-messages",
                        for message in &chat.messages {
                            div {
                                class: "flex items-start gap-3",
                                if let Some(creator) = message.creator {
                                    if let Some(chat_user) = chat.users.iter().find(|user| user.id == creator) {
                                        components::avatar::Avatar {
                                            src: Some(chat_user.profile_image.clone()),
                                            alt: chat_user.display_name.clone(),
                                            size: components::avatar::Size::Small,
                                        }
                                        div {
                                            class: "font-semibold text-blue-600",
                                            "{chat_user.display_name}"
                                        }
                                    } else {
                                        components::avatar::Avatar {
                                            src: None,
                                            alt: "U K".to_string(),
                                            size: components::avatar::Size::Small,
                                        }
                                        div {
                                            class: "font-semibold text-blue-600",
                                            "Unknown({creator.to_string()})"
                                        }
                                    }
                                } else {
                                    components::avatar::Avatar {
                                        src: None,
                                        alt: "S Y".to_string(),
                                        size: components::avatar::Size::Small,
                                    }
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
                                    })).await;

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
                                components::avatar::Avatar {
                                    src: Some(user.profile_image.clone()),
                                    alt: user.display_name.clone(),
                                    size: components::avatar::Size::Small,
                                }
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
                                        components::avatar::Avatar {
                                            src: Some(user.profile_image.clone()),
                                            alt: user.display_name.clone(),
                                            size: components::avatar::Size::Small,
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
