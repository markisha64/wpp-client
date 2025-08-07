#![allow(non_snake_case)]

use components::navbar::Auth;
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use route::Route;

use gloo_timers::future::TimeoutFuture;
use std::collections::HashMap;

use chrono::Utc;
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

use futures_util::{
    future::{select, Either},
    SinkExt, StreamExt,
};
use uuid::Uuid;
use {pharos::*, ws_stream_wasm::*};

mod components;
mod pages;
mod route;

pub static BACKEND_URL: &str = match option_env!("BACKEND_URL") {
    Some(x) => x,
    None => "http://localhost:3030",
};
pub static BACKEND_URL_WS: &str = match option_env!("BACKEND_URL") {
    Some(x) => x,
    None => "ws://localhost:3030",
};

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

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

fn App() -> Element {
    use_coroutine(move |mut rx: UnboundedReceiver<anyhow::Error>| async move {
        let _ = document::eval(
            r#"

            let e = document.getElementById('toast');    
            e.onclick = () => {
                e.classList.add('hidden')
            }
                
            "#,
        )
        .await;

        while let Some(msg) = rx.next().await {
            let _ = document::eval(
                format!(
                    r#"

                let ec = document.getElementById('toast-content');    
                ec.innerHTML = "{}"

                let e = document.getElementById('toast');    
                e.classList.remove('hidden')
                
                "#,
                    msg
                )
                .as_str(),
            )
            .await;
        }
    });

    use_coroutine(
        move |mut rx: UnboundedReceiver<(
            WebsocketClientMessageData,
            oneshot::Sender<Result<WebsocketServerResData, String>>,
        )>| async move {
            loop {
                let mut message_requests: HashMap<Uuid, _> = HashMap::new();
                let user_o = USER();
                let token = user_o.map(|x| (x.token, x.claims.user.id));

                if let Some((token, user_id)) = token {
                    if let Ok((mut ws, mut wsio)) =
                        WsMeta::connect(format!("{}/ws/?jwt_token={}", BACKEND_URL_WS, token), None)
                            .await
                    {
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
                                            .send(WsMessage::Text(
                                                serde_json::to_string(&request).unwrap(),
                                            ))
                                            .await
                                        {
                                            message_requests.insert(id, responder);
                                        }
                                    }
                                }

                                Either::Right((Either::Left((x, _)), _)) => {
                                    tracing::info!("websocket event {:?}", x);

                                    // break here connection probably dead
                                    break;
                                }

                                Either::Right((Either::Right((x, _)), _)) => {
                                    if let Some(Text(payload)) = x {
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

                                                    result => {
                                                        if let Some(x) =
                                                            message_requests.remove(&id)
                                                        {
                                                            let _ = x.send(result);
                                                        }
                                                    }
                                                },

                                                WebsocketServerMessage::NewMessage(message) => {
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

                                                _ => todo!("mediasoup"),
                                            }
                                        }
                                    }
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
