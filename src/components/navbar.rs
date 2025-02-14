use std::collections::HashMap;

use chrono::Utc;
use dioxus::prelude::*;
use dioxus_logger::tracing;
use jsonwebtoken::DecodingKey;
use shared::api::user::Claims;
use shared::api::websocket::{
    WebsocketClientMessage, WebsocketClientMessageData, WebsocketServerMessage,
    WebsocketServerResData,
};
use shared::models::chat::ChatSafe;
use tokio::sync::oneshot;
use ws_stream_wasm::WsMessage::Text;

use crate::route::Route;

#[derive(Clone)]
pub struct Auth {
    pub claims: Claims,
    pub token: String,
}
use futures_util::{
    future::{select, Either},
    SinkExt, StreamExt,
};
use uuid::Uuid;
use {pharos::*, ws_stream_wasm::*};

pub static USER: GlobalSignal<Option<Auth>> = Signal::global(|| {
    let storage = web_sys::window()?.local_storage().ok()??;
    let jwt_token = storage.get_item("jwt_token").ok()??;

    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.insecure_disable_signature_validation();

    let key = DecodingKey::from_secret(&[]);
    let payload = jsonwebtoken::decode::<Claims>(jwt_token.as_str(), &key, &validation).ok()?;

    if payload.claims.exp <= Utc::now().timestamp() as usize {
        storage.remove_item("jwt_token").ok()?;

        return None;
    }

    Some(Auth {
        claims: payload.claims,
        token: jwt_token,
    })
});

pub static CHATS: GlobalSignal<Vec<ChatSafe>> = Signal::global(|| Vec::new());

#[component]
pub fn NavBar() -> Element {
    let user_r = USER();
    let display_login = user_r.is_none();
    let display_name = user_r.clone();

    use_coroutine(
        move |mut rx: UnboundedReceiver<(
            WebsocketClientMessageData,
            oneshot::Sender<Result<WebsocketServerResData, String>>,
        )>| async move {
            let mut message_requests: HashMap<Uuid, _> = HashMap::new();

            let token = USER().clone().map(|x| x.token);

            if let Some(token) = token {
                let (mut ws, mut wsio) =
                    WsMeta::connect(format!("ws://localhost:3030/ws/?jwt_token={}", token), None)
                        .await
                        .unwrap();

                let mut evts = ws.observe(ObserveConfig::default()).await.unwrap();

                let chats_request_id = Uuid::new_v4();
                wsio.send(WsMessage::Text(
                    serde_json::to_string(&WebsocketClientMessage {
                        id: chats_request_id,
                        data: WebsocketClientMessageData::GetChats,
                    })
                    .unwrap(),
                ))
                .await
                .unwrap();

                loop {
                    let rrx = rx.next();
                    let evtx = select(evts.next(), wsio.next());

                    match select(rrx, evtx).await {
                        Either::Left((x, _)) => {
                            if let Some((data, responder)) = x {
                                let id = Uuid::new_v4();

                                let request = WebsocketClientMessage { id, data };

                                if let Ok(_) = wsio
                                    .send(WsMessage::Text(serde_json::to_string(&request).unwrap()))
                                    .await
                                {
                                    message_requests.insert(id, responder);
                                }
                            }
                        }

                        Either::Right((Either::Left((x, _)), _)) => {
                            tracing::info!("websocket event {:?}", x);
                            break;
                        }

                        Either::Right((Either::Right((x, _)), _)) => {
                            if let Some(Text(payload)) = x {
                                if let Ok(message) =
                                    serde_json::from_str::<WebsocketServerMessage>(&payload)
                                {
                                    match &message {
                                        WebsocketServerMessage::RequestResponse { id, data } => {
                                            match data {
                                                Ok(WebsocketServerResData::GetChats(chats)) => {
                                                    *CHATS.write() = chats.clone();
                                                }

                                                result => {
                                                    if let Some(x) = message_requests.remove(id) {
                                                        let _ = x.send(result.clone());
                                                    }
                                                }
                                            }
                                        }

                                        WebsocketServerMessage::NewMessage(message) => {
                                            let chats = &mut (*CHATS.write());
                                            let chat_o =
                                                chats.iter_mut().find(|x| x.id == message.chat_id);

                                            if let Some(chat) = chat_o {
                                                let ts = message.created_at;

                                                chat.messages.push(message.clone());
                                                chat.last_message_ts = ts;
                                            }

                                            chats.sort_by(|a, b| {
                                                a.last_message_ts.cmp(&b.last_message_ts).reverse()
                                            })
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    );

    rsx! {
        nav {
            class: "fixed top-0 z-50 w-full border-b bg-gray-800 border-gray-700 text-white",
            div {
                class: "w-full flex flex-wrap items-center justify-between px-3 py-3 lg:px-5 lg:pl-3",
                "CHET",
                if display_login {
                    div {
                        class: "flex md:order-2 space-x-3 md:space-x-0 rtl:space-x-reverse",
                        button {
                            r#type: "button",
                            class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center bg-blue-600 hover:bg-blue-700 focus:ring-blue-800",
                            Link {
                                to: Route::Login {}, "Login"
                            }
                        }
                    }
                }
                else {
                    div {
                        "{display_name.as_ref().unwrap().claims.user.display_name.clone()}"
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}
