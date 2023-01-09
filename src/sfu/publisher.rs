use std::sync::Arc;

use anyhow::Ok;
use tokio::sync::Mutex;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
    },
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration,
        RTCPeerConnection,
    },
};

pub struct Publisher {
    pub pc: Arc<Mutex<RTCPeerConnection>>,
}

impl Publisher {
    pub async fn new() -> anyhow::Result<Publisher> {
        // set up webrtc stuff
        let mut m = MediaEngine::default();

        m.register_default_codecs()?;

        let mut registry = Registry::new();

        registry = register_default_interceptors(registry, &mut m)?;

        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let pc = Arc::new(Mutex::new(api.new_peer_connection(config).await?));

        let p = Publisher { pc };

        Ok(p)
    }
}
