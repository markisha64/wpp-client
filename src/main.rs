#![allow(non_snake_case)]

use components::navbar::Auth;
use dioxus::document::EvalError;
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, warn, Level};
use route::Route;

use gloo_timers::future::TimeoutFuture;
use shared::models::user::UserSafe;
use std::collections::HashMap;

use chrono::Utc;
use dioxus_logger::tracing;
use jsonwebtoken::DecodingKey;
use shared::api::user::Claims;
use shared::api::websocket::{
    MediaSoupMessage, WebsocketClientMessage, WebsocketClientMessageData, WebsocketServerMessage,
    WebsocketServerResData,
};
use shared::models::chat::ChatSafe;
use tokio::sync::oneshot;
use ws_stream_wasm::WsMessage::Text;

use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;
use {pharos::*, ws_stream_wasm::*};

mod components;
mod pages;
mod route;

pub static BACKEND_URL: &str = match option_env!("BACKEND_URL") {
    Some(x) => x,
    None => "http://localhost:3030",
};
pub static BACKEND_URL_WS: &str = match option_env!("BACKEND_URL_WS") {
    Some(x) => x,
    None => "ws://localhost:3030",
};

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

pub static USER: GlobalSignal<Option<UserSafe>> = Signal::global(|| None);
pub static CLAIMS: GlobalSignal<Option<Auth>> = Signal::global(|| {
    let storage = web_sys::window()?.local_storage().ok()??;
    let jwt_token = storage.get_item("jwt_token").ok()??;
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.insecure_disable_signature_validation();

    let key = DecodingKey::from_secret(&[]);
    let payload = match jsonwebtoken::decode::<Claims>(jwt_token.as_str(), &key, &validation) {
        Ok(e) => Some(e),
        Err(_) => {
            storage.remove_item("jwt_tooken").ok()?;
            None
        }
    }?;

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

fn App() -> Element {
    use_coroutine(
        move |mut ws_channel: UnboundedReceiver<(
            WebsocketClientMessageData,
            oneshot::Sender<Result<WebsocketServerResData, String>>,
        )>| async move {
            loop {
                let mut message_requests: HashMap<Uuid, _> = HashMap::new();
                let user_o = CLAIMS();
                let token = user_o.map(|x| (x.token, x.claims.user_id));

                if let Some((token, user_id)) = token {
                    if let Ok((mut ws, mut wsio)) =
                        WsMeta::connect(format!("{}/ws/?jwt_token={}", BACKEND_URL_WS, token), None)
                            .await
                    {
                        let mut ms_js = document::eval(include_str!("../js/mediasoup.js"));

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

                        let get_self_request_id = Uuid::new_v4();
                        wsio.send(WsMessage::Text(
                            serde_json::to_string(&WebsocketClientMessage {
                                id: get_self_request_id,
                                data: WebsocketClientMessageData::GetSelf,
                            })
                            .unwrap(),
                        ))
                        .await
                        .unwrap();

                        loop {
                            tokio::select! {
                                Some((data, responder)) = ws_channel.next() => {
                                    let id = Uuid::new_v4();

                                    let request = WebsocketClientMessage { id, data };

                                    if let Ok(_) = wsio
                                        .send(WsMessage::Text(
                                            serde_json::to_string(&request).unwrap(),
                                        ))
                                        .await
                                    {
                                        message_requests.insert(id, responder);
                                    }
                                }

                                x = evts.next() => {
                                    tracing::info!("websocket event {:?}", x);

                                    // break here connection probably dead
                                    break;
                                }

                                Some(Text(payload)) = wsio.next() => {
                                    if let Ok(message) =
                                        serde_json::from_str::<WebsocketServerMessage>(&payload)
                                    {
                                        match message {
                                            WebsocketServerMessage::RequestResponse {
                                                id,
                                                data,
                                            } => match data {
                                                Ok(WebsocketServerResData::GetChats(chats)) => {
                                                    *CHATS.write() = chats.clone();
                                                }

                                                Ok(WebsocketServerResData::GetSelf(user)) => {
                                                    USER.write().replace(user);
                                                }

                                                Ok(WebsocketServerResData::NewMessage(message)) => {
                                                    let chats = &mut (*CHATS.write());
                                                    let chat_o = chats
                                                        .iter_mut()
                                                        .find(|x| x.id == message.chat_id);

                                                    if let Some(chat) = chat_o {
                                                        let ts = message.created_at;

                                                        chat.messages.push(message.clone());
                                                        chat.last_message_ts = ts;
                                                    }

                                                    chats.sort_by(|a, b| {
                                                        a.last_message_ts
                                                            .cmp(&b.last_message_ts)
                                                            .reverse()
                                                    });

                                                    if let Some(x) =
                                                        message_requests.remove(&id)
                                                    {
                                                        let _ = x.send(Ok(WebsocketServerResData::NewMessage(message)));
                                                    }
                                                }

                                                Ok(WebsocketServerResData::MS(media_soup)) => {
                                                    let _ = ms_js.send(WebsocketServerMessage::RequestResponse {
                                                        id,
                                                        data: Ok(WebsocketServerResData::MS(media_soup.clone()))
                                                    });

                                                    if let Some(x) =
                                                        message_requests.remove(&id)
                                                    {
                                                        let _ = x.send(Ok(WebsocketServerResData::MS(media_soup)));
                                                    }
                                                }

                                                result => {
                                                    if let Some(x) =
                                                        message_requests.remove(&id)
                                                    {
                                                        let _ = x.send(result);
                                                    }
                                                }
                                            },

                                            WebsocketServerMessage::NewMessage(message) => {
                                                if message.creator == Some(user_id) {
                                                    continue;
                                                }

                                                let chats = &mut (*CHATS.write());
                                                let chat_o = chats
                                                    .iter_mut()
                                                    .find(|x| x.id == message.chat_id);

                                                if let Some(chat) = chat_o {
                                                    let ts = message.created_at;

                                                    chat.messages.push(message.clone());
                                                    chat.last_message_ts = ts;
                                                }

                                                chats.sort_by(|a, b| {
                                                    a.last_message_ts
                                                        .cmp(&b.last_message_ts)
                                                        .reverse()
                                                })
                                            }

                                            WebsocketServerMessage::UserJoined {
                                                chat_id,
                                                user,
                                            } => {
                                                let chats = &mut (*CHATS.write());
                                                let chat_o =
                                                    chats.iter_mut().find(|x| x.id == chat_id);

                                                if let Some(chat) = chat_o {
                                                    if let None = chat
                                                        .users
                                                        .iter()
                                                        .find(|x| x.id == user.id)
                                                    {
                                                        chat.users.push(user.clone())
                                                    }
                                                }
                                            }

                                            WebsocketServerMessage::SetChatRead {
                                                chat_id,
                                                last_message_ts,
                                            } => {
                                                let chats = &mut (*CHATS.write());
                                                let chat_o =
                                                    chats.iter_mut().find(|x| x.id == chat_id);

                                                if let Some(chat) = chat_o {
                                                    if let Some(chat_user) = chat
                                                        .users
                                                        .iter_mut()
                                                        .find(|x| x.id == user_id)
                                                    {
                                                        chat_user.last_message_seen_ts =
                                                            last_message_ts;
                                                    }
                                                }
                                            }

                                            WebsocketServerMessage::ProfileUpdated(user) => {
                                                USER.write().replace(user);
                                            }

                                            WebsocketServerMessage::ProducerAdded {
                                                participant_id,
                                                producer_id,
                                            } => {
                                                let _ = ms_js.send(WebsocketServerMessage::ProducerAdded { participant_id, producer_id  });
                                            },

                                            WebsocketServerMessage::ProducerRemove {
                                                participant_id,
                                                producer_id,
                                            } => {
                                                let _ = ms_js.send(WebsocketServerMessage::ProducerRemove { participant_id, producer_id  });
                                            },
                                        }
                                    }
                                }

                                ms_r = ms_js.recv::<MediaSoupMessage>() => {
                                    let ms = match ms_r {
                                        Ok(ms) => ms,
                                        Err(e) => {
                                            match e {
                                                EvalError::Finished => {
                                                    warn!("re running document::eval");
                                                    ms_js = document::eval(include_str!("../js/mediasoup.js"));
                                                }
                                                e => {
                                                    info!("err {:?}", e);
                                                }
                                            }

                                            continue;
                                        }
                                    };

                                    let _ = wsio.send(WsMessage::Text(
                                        serde_json::to_string(&WebsocketClientMessage {
                                            id: uuid::Uuid::nil(),
                                            data: WebsocketClientMessageData::MS(ms)
                                        }).unwrap()
                                    )).await;
                                }
                            }
                        }
                    }

                    // wait for reconnect
                    TimeoutFuture::new(10000).await;
                }

                // wait for recheck token
                TimeoutFuture::new(1000).await;
            }
        },
    );

    rsx! {
        document::Stylesheet {
            href: asset!("/assets/tailwind.css")
        }
        document::Script {
            src: asset!("/assets/mediasoup-client.bundle.js")
        }
        div {
            id: "toast",
            class: "fixed top-5 right-5 z-70 hidden",
            div {
                class: "bg-red-500 p-4 rounded-lg shadow-lg w-72 flex justify-between items-center",
                p {
                    id: "toast-content",
                    ""
                }
            }
        }
        Router::<Route> {}
    }
}
