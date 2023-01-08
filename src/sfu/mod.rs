use std::{collections::HashMap, sync::Arc};

use anyhow::Ok;
use tokio::sync::Mutex;
use uuid::Uuid;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::signalling::SocketResponse;

use self::{publisher::Publisher, session::Session};

pub mod publisher;
pub mod session;
pub mod subscriber;

pub struct SFU {
    sessions: Mutex<HashMap<String, Arc<Mutex<Session>>>>,
}

impl SFU {
    pub fn new() -> SFU {
        return SFU {
            sessions: Mutex::new(HashMap::new()),
        };
    }

    async fn get_session(&mut self, id: &String) -> anyhow::Result<Arc<Mutex<Session>>> {
        let mut sessions = self.sessions.lock().await;
        let s = Arc::new(Mutex::new(Session::new()));
        sessions.entry(id.clone()).or_insert(s.clone());
        Ok(s)
    }

    pub async fn create_publisher(
        &mut self,
        offer: String,
        session_id: &String,
    ) -> anyhow::Result<SocketResponse> {
        let p = Publisher::new().await?;
        let pub_id = Uuid::new_v4();

        // set remote desc
        p.pc.set_remote_description(RTCSessionDescription::offer(offer)?)
            .await?;

        // create an answer
        let answer = p.pc.create_answer(None).await?;

        let s = self.get_session(session_id).await?;

        s.lock().await.publishers.insert(pub_id.to_string(), p);

        Ok(SocketResponse::CreateTransportRes {
            answer,
            publisher_id: pub_id.to_string(),
            msg_type: "PUBLISHER_CREATED".to_owned(),
        })
    }

    pub async fn renegotiate(
        &mut self,
        offer: String,
        publisher_id: String,
        session_id: &String,
    ) -> anyhow::Result<SocketResponse> {
        let s = self.get_session(session_id).await?;
        let locked_session = s.lock().await;
        let p = locked_session.get_publisher(publisher_id)?;

        p.pc.set_remote_description(RTCSessionDescription::offer(offer)?)
            .await?;

        let ans = p.pc.create_answer(None).await?;

        Ok(SocketResponse::NegotiateRes {
            sdp: ans,
            msg_type: "NEGOTIATION_DONE".to_owned(),
        })
    }
}
