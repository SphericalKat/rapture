use std::{collections::HashMap, sync::Arc};

use anyhow::Ok;
use axum::extract::ws::Message;
use futures::sink::SinkExt;

use tokio::sync::Mutex;
use tracing::Level;
use uuid::Uuid;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::signalling::{SendSocket, SocketResponse};

use self::{publisher::Publisher, session::Session};

pub mod publisher;
pub mod session;
pub mod subscriber;

pub struct SFU<'a> {
    sessions: Mutex<HashMap<String, Arc<Mutex<Session<'a, >>>>>,
}

impl<'a> SFU<'a> {
    pub fn new() -> SFU<'a> {
        return SFU {
            sessions: Mutex::new(HashMap::new()),
        };
    }

    async fn get_session(&'a mut self, id: &String) -> anyhow::Result<Arc<Mutex<Session>>> {
        let mut sessions = self.sessions.lock().await;
        let s = Arc::new(Mutex::new(Session::new()));
        sessions.entry(id.clone()).or_insert(s.clone());
        Ok(s)
    }

    pub async fn create_publisher(
        &'a mut self,
        socket: Arc<Mutex<&mut SendSocket>>,
        offer: String,
        session_id: &String,
    ) -> anyhow::Result<SocketResponse> {
        let p = Publisher::new().await?;
        let pub_id = Uuid::new_v4();

        // set remote desc
        let peer_conn = p.pc.clone();
        let pc = p.pc.clone();
        let locked_pc = pc.lock().await;

        locked_pc
            .set_remote_description(RTCSessionDescription::offer(offer)?)
            .await?;

        // create an answer
        let answer = locked_pc.create_answer(None).await?;

        locked_pc.on_negotiation_needed(Box::new(move || {
            let pc = peer_conn.clone();
            // let skt = socket.clone();

            Box::pin(async move {
                let locked_pc = pc.lock().await;
                let offer = locked_pc
                    .create_offer(None)
                    .await
                    .expect("on_negotiation_needed::could not create offer");
                locked_pc
                    .set_local_description(offer.clone())
                    .await
                    .expect("on_negotiation_needed::could not set local desc");

                // let mut locked_socket = skt.lock().await;
                // let msg = serde_json::to_string(&SocketResponse::RenegotiateRes {
                //     sdp: offer,
                //     msg_type: "ON_NEGOTIATION_NEEDED".to_owned(),
                // })
                // .expect("on_negotiation_needed::could not serialize socket response");
                // locked_socket.send(Message::Text(msg)).await;
            })
        }));

        let s = self.get_session(session_id).await?;

        s.lock().await.publishers.insert(pub_id.to_string(), p);

        Ok(SocketResponse::CreateTransportRes {
            answer,
            publisher_id: pub_id.to_string(),
            msg_type: "PUBLISHER_CREATED".to_owned(),
        })
    }

    pub async fn renegotiate(
        &'a mut self,
        socket: Arc<Mutex<&mut SendSocket>>,
        offer: String,
        publisher_id: String,
        session_id: &String,
    ) -> anyhow::Result<SocketResponse> {
        tracing::event!(
            Level::INFO,
            publisher_id = publisher_id,
            session_id = session_id,
            "Renegotiating"
        );
        let s = self.get_session(session_id).await?;
        let locked_session = s.lock().await;
        let p = locked_session.get_publisher(publisher_id)?;
        let pc = p.pc.lock().await;

        pc.set_remote_description(RTCSessionDescription::offer(offer)?)
            .await?;

        let ans = pc.create_answer(None).await?;

        Ok(SocketResponse::RenegotiateRes {
            sdp: ans,
            msg_type: "NEGOTIATION_DONE".to_owned(),
        })
    }

    async fn renegotiate_server(
        &'a mut self,
        publisher_id: String,
        session_id: &String,
    ) -> anyhow::Result<SocketResponse> {
        tracing::event!(
            Level::INFO,
            publisher_id = publisher_id,
            session_id = session_id,
            "Renegotiating"
        );
        let s = self.get_session(session_id).await?;
        let locked_session = s.lock().await;
        let p = locked_session.get_publisher(publisher_id)?;
        let pc = p.pc.lock().await;

        let offer = pc.create_offer(None).await?;

        Ok(SocketResponse::RenegotiateRes {
            sdp: offer,
            msg_type: "NEGOTIATION_DONE".to_owned(),
        })
    }
}
