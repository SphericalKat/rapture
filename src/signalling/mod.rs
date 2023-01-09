use std::sync::Arc;

use anyhow::{anyhow};
use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::SFU;

#[derive(Serialize, Deserialize, Debug)]
pub enum Transport {
    PUBLISHER,
    SUBSCRIBER,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SocketMessage {
    CreateTransportReq {
        transport_type: Transport,
        offer: String,
    },
    RenegotiateReq {
        offer: String,
        publisher_id: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SocketResponse {
    CreateTransportRes {
        answer: RTCSessionDescription,
        publisher_id: String,
        msg_type: String,
    },
    RenegotiateRes {
        sdp: RTCSessionDescription,
        msg_type: String,
    },
}

pub type SendSocket = SplitSink<WebSocket, Message>;

pub async fn parse_message(socket: &mut SendSocket, message: String, session_id: &String) {
    if let Ok(d) = serde_json::from_str::<SocketMessage>(&message) {
        tracing::debug!("{:?}", d);
        let skt = Arc::new(Mutex::new(socket));
        match handle_socket_message(skt.clone(), d, session_id).await {
            Ok(res) => {
                let res_json = serde_json::to_string(&res).unwrap();
                if let Err(e) = skt.lock().await.send(Message::Text(res_json)).await {
                    tracing::error!("error sending socket message: {}", e)
                }
            }
            Err(e) => tracing::error!("error creating transport: {}", e),
        };
    }
}
async fn handle_socket_message(
    socket: Arc<Mutex<&mut SendSocket>>,
    msg: SocketMessage,
    session_id: &String,
) -> anyhow::Result<SocketResponse> {
    let mut locked_sfu = SFU.lock().await;
    match msg {
        SocketMessage::CreateTransportReq {
            transport_type,
            offer,
        } => match transport_type {
            Transport::PUBLISHER => locked_sfu.create_publisher(socket, offer, session_id).await,
            Transport::SUBSCRIBER => todo!(),
        },
        SocketMessage::RenegotiateReq {
            offer,
            publisher_id,
        } => {
            // locked_sfu.renegotiate(socket, offer, publisher_id, session_id)
            //     .await
            Err(anyhow!("asdasda"))
        }
    }
}
