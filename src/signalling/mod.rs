use axum::extract::ws::{Message, WebSocket};
use serde::{Deserialize, Serialize};

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
    CreateTransportRes { answer: String, publisher_id: String, msg_type: String },
    NegotiateRes { sdp: String, msg_type: String },
}

pub async fn handle_text_message(socket: &mut WebSocket, t: String) {
    if let Ok(d) = serde_json::from_str::<SocketMessage>(&t) {
        tracing::debug!("{:?}", d);
        match handle_socket_message(d).await {
            Ok(res) => {
                let res_json = serde_json::to_string(&res).unwrap();
                if let Err(e) = socket.send(Message::Text(res_json)).await {
                    tracing::error!("error sending socket message: {}", e)
                }
            }
            Err(e) => tracing::error!("error creating transport: {}", e),
        };
    }
}
async fn handle_socket_message(msg: SocketMessage) -> anyhow::Result<SocketResponse> {
    match msg {
        SocketMessage::CreateTransportReq {
            transport_type,
            offer,
        } => match transport_type {
            Transport::PUBLISHER => SFU.lock().await.create_publisher(offer).await,
            Transport::SUBSCRIBER => todo!(),
        },
        SocketMessage::RenegotiateReq {
            offer,
            publisher_id,
        } => SFU.lock().await.renegotiate(offer, publisher_id).await,
    }
}
