use bson::oid::ObjectId;
use dioxus::prelude::*;

use crate::components::navbar::CHATS;

pub fn Home() -> Element {
    let mut selected_chat_signal = use_signal::<Option<ObjectId>>(|| None);
    let selected_chat = selected_chat_signal.read().clone();

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

    rsx! {
        aside {
            class: "fixed top-0 left-0 z-40 w-64 h-screen pt-20 transition-transform -translate-x-full border-r sm:translate-x-0 bg-gray-800 border-gray-700",
            div {
                class: "h-full px-3 py-4 overflow-y-auto bg-gray-800",
                ul {
                    class: "space-y-2 font-medium",
                    for (chat, id, class) in chats {
                        li {
                            a {
                                class: "items-center p-2 rounded-lg text-white hover:bg-gray-600 group {class}",
                                onclick: move |_| {
                                    selected_chat_signal.set(Some(id));
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
    }
}
