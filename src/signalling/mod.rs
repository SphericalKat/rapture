use serde::{Deserialize, Serialize};

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
    CreateTransportRes {
        answer: String,
    }
}
