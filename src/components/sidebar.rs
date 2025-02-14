use bson::oid::ObjectId;
use dioxus::prelude::*;

use crate::{pages::home::UpdateHeight, CHATS};

#[component]
pub fn Sidebar(
    selected_chat_id_signal: Signal<Option<ObjectId>>,
    update_height_signal: Signal<UpdateHeight>,
) -> Element {
    let selected_chat_id = selected_chat_id_signal();
    let chats = CHATS();

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

    rsx! {
        aside {
            class: "fixed top-0 left-0 z-40 w-40 h-screen pt-20 transition-transform -translate-x-full border-r sm:translate-x-0 bg-gray-800 border-gray-700",
            div {
                class: "h-full px-3 py-4 overflow-y-auto bg-gray-800",
                ul {
                    class: "space-y-2 font-medium",
                    for (name, id, class) in chats_mapped {
                        li {
                            a {
                                class: "items-center p-2 rounded-lg text-white hover:bg-gray-600 group {class}",
                                onclick: move |_| {
                                    async move {
                                        selected_chat_id_signal.set(Some(id));
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
    }
}
