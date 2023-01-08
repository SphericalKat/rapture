use std::collections::HashMap;

use anyhow::Context;

use super::{publisher::Publisher, subscriber::Subscriber};

pub struct Session {
    pub publishers: HashMap<String, Publisher>,
    pub subscribers: HashMap<String, Subscriber>,
}

impl Session {
    pub fn new() -> Session {
        Session {
            publishers: HashMap::new(),
            subscribers: HashMap::new(),
        }
    }

    pub fn get_publisher(&self, id: String) -> anyhow::Result<&Publisher> {
        self.publishers.get(&id).context("publisher not found")
    }

    pub fn get_subscriber(&self, id: String) -> anyhow::Result<&Subscriber> {
        self.subscribers.get(&id).context("subscriber not found")
    }
}
