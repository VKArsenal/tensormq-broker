use bytes::Bytes;
use dashmap::DashMap;
use tokio::sync::broadcast;

use crate::protocol::MsgType;

/// Parsed Tensor Message ready for routing.
#[derive(Debug, Clone)]
pub struct TensorMessage {
    pub msg_type: MsgType,
    pub flags: u8,
    pub stream_id: u64,
    pub topic: String,
    pub meta: Bytes,
    pub tensor: Bytes,
}

/// Routing Engine
pub struct Broker {
    exact_topics: DashMap<String, broadcast::Sender<TensorMessage>>,
    pattern_topics: DashMap<String, broadcast::Sender<TensorMessage>>,
    channel_capacity: usize,
}

/// Matching topics against patterns
fn matches_pattern(topic: &str, pattern: &str) -> bool {
    let t_parts: Vec<&str> = topic.split('/').collect();
    let p_parts: Vec<&str> = pattern.split('/').collect();

    if t_parts.len() != p_parts.len() {
        return false;
    }
    for (t, p) in t_parts.iter().zip(p_parts.iter()) {
        if *p != "*" && t != p {
            return false;
        }
    }
    return true;
}

impl Broker {
    pub fn new(channel_capacity: usize) -> Self {
        Self {
            exact_topics: DashMap::new(),
            pattern_topics: DashMap::new(),
            channel_capacity,
        }
    }

    pub fn subscribe(&self, topic_or_pattern: &str) -> broadcast::Receiver<TensorMessage> {
        if topic_or_pattern.contains('*') {
            let sender = self.pattern_topics.entry(topic_or_pattern.to_string()).or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(self.channel_capacity);
                tx
            });
            sender.subscribe()
        } else {
            let sender = self.exact_topics.entry(topic_or_pattern.to_string()).or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(self.channel_capacity);
                tx
            });
            sender.subscribe()
        }
    }

    pub fn publish(&self, msg: TensorMessage) {
        let mut sent_count = 0;

        if let Some(sender) = self.exact_topics.get(&msg.topic) {
            if sender.send(msg.clone()).is_ok() {
                sent_count += sender.receiver_count();
            }
        }

        for entry in self.pattern_topics.iter() {
            let pattern = entry.key();
            if matches_pattern(&msg.topic, pattern) {
                if entry.value().send(msg.clone()).is_ok() {
                    sent_count += entry.value().receiver_count();
                }
            }
        }

        if sent_count > 0 {
            println!("Routed {} to {} subscribers", msg.topic, sent_count);
        } else {
            println!("Dropped {} (no subscribers)", msg.topic);
        }
    }
}