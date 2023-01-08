use std::collections::HashMap;

use anyhow::{Context, Ok};
use uuid::Uuid;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::signalling::SocketResponse;

use self::publisher::Publisher;

pub mod publisher;
pub mod subscriber;

pub struct SFU {
    publishers: HashMap<String, Publisher>,
}

impl SFU {
    pub fn new() -> SFU {
        return SFU {
            publishers: HashMap::new(),
        };
    }

    fn get_publisher_by_id(&self, id: String) -> anyhow::Result<&Publisher> {
        self.publishers.get(&id).context("publisher not found")
    }

    pub async fn create_publisher(&mut self, offer: String, session_id: &String) -> anyhow::Result<SocketResponse> {
        let p = Publisher::new().await?;
        let pub_id = Uuid::new_v4();

        // set remote desc
        p.pc.set_remote_description(RTCSessionDescription::offer(offer)?)
            .await?;

        // create an answer
        let answer = p.pc.create_answer(None).await?;

        self.publishers.insert(pub_id.to_string(), p);

        Ok(SocketResponse::CreateTransportRes {
            answer,
            publisher_id: pub_id.to_string(),
            msg_type: "PUBLISHER_CREATED".to_owned(),
        })
    }

    pub async fn renegotiate(
        &self,
        offer: String,
        publisher_id: String,
        session_id: &String
    ) -> anyhow::Result<SocketResponse> {
        let p = self.get_publisher_by_id(publisher_id)?;

        p.pc.set_remote_description(RTCSessionDescription::offer(offer)?)
            .await?;

        let ans = p.pc.create_answer(None).await?;

        Ok(SocketResponse::NegotiateRes {
            sdp: ans,
            msg_type: "NEGOTIATION_DONE".to_owned(),
        })
    }
}
