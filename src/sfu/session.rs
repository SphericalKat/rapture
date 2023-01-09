use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use tokio::sync::Mutex;

use crate::signalling::SendSocket;

use super::{publisher::Publisher, subscriber::Subscriber};

pub struct Session<'session> {
    pub publishers: HashMap<String, Publisher>,
    pub subscribers: HashMap<String, Subscriber>,
    pub socket: HashMap<String, Arc<Mutex<&'session mut SendSocket>>>
}

impl<'session> Session<'session> {
    pub fn new() -> Session<'session> {
        Session {
            publishers: HashMap::new(),
            subscribers: HashMap::new(),
            socket: HashMap::new(),
        }
    }

    pub fn get_publisher(&self, id: String) -> anyhow::Result<&Publisher> {
        self.publishers.get(&id).context("publisher not found")
    }

    pub fn get_subscriber(&self, id: String) -> anyhow::Result<&Subscriber> {
        self.subscribers.get(&id).context("subscriber not found")
    }
}
