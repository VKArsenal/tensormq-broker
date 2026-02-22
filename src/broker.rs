use bytes::Bytes;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Parsed Tensor Message ready for routing.
#[derive(Debug, Clone)]
pub struct TensorMessage {
    pub topic: String,
    pub meta: Bytes,
    pub tensor: Bytes,
}

/// Routing Engine
pub struct Broker {
    topics: DashMap<String, broadcast::Sender<TensorMessage>>,
    channel_capacity: usize,
}

impl Broker {
    pub fn new(channel_capacity: usize) -> Self {
        Self {
            topics: DashMap::new(),
            channel_capacity,
        }
    }

    pub fn subscribe(&self, topic: &str) -> broadcast::Receiver<TensorMessage> {
        let sender = self.topics.entry(topic.to_string()).or_insert_with(|| {
            let (tx, _rx) = broadcast::channel(self.channel_capacity);
            tx
        });
        sender.subscribe()
    }

    pub fn publish(&self, msg: TensorMessage) {
        if let Some(sender) = self.topics.get(&msg.topic) {
            let active_subscribers = sender.receiver_count();
            if sender.send(msg.clone()).is_ok() {
                println!("Broadcasted to {} subscriber(s)", active_subscribers);
            } else {
                println!("Dropped message (no active subscribers)");
            }
        } else {
            println!("Dropped message (Topic {} does not exist)", msg.topic);
        }
    }
}