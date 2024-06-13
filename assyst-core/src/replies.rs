use std::time::{Duration, Instant};

use moka::sync::Cache;

#[derive(Copy, Clone, Debug)]
pub struct ReplyInUse {
    /// The message ID of this reply
    pub message_id: u64,
    /// Whether the reply has any attachments.
    pub has_attachments: bool,
}

#[derive(Debug, Clone)]
pub enum ReplyState {
    Processing,
    InUse(ReplyInUse),
}

#[derive(Debug, Clone)]
pub struct Reply {
    pub state: ReplyState,
    pub created: Instant,
}

impl Reply {
    pub fn in_use(&self) -> Option<ReplyInUse> {
        if let ReplyState::InUse(reply) = self.state {
            Some(reply)
        } else {
            None
        }
    }
}

/// Cached command replies. First cache is for "raw" messages, second is for interaction messages.
pub struct Replies(Cache<u64, Reply>, Cache<u64, ()>);

impl Replies {
    pub fn new() -> Self {
        Self(
            Cache::builder()
                .max_capacity(1000)
                .time_to_idle(Duration::from_secs(60 * 5))
                .build(),
            Cache::builder()
                .max_capacity(1000)
                .time_to_idle(Duration::from_secs(60 * 5))
                .build(),
        )
    }

    pub fn insert_raw_message(&self, id: u64, reply: Reply) {
        self.0.insert(id, reply);
    }

    pub fn remove_raw_message(&self, id: u64) -> Option<Reply> {
        self.0.remove(&id)
    }

    pub fn get_raw_message(&self, id: u64) -> Option<Reply> {
        self.0.get(&id)
    }

    pub fn insert_interaction_command(&self, id: u64) {
        self.1.insert(id, ());
    }

    pub fn remove_interaction_command(&self, id: u64) -> Option<()> {
        self.1.remove(&id)
    }

    pub fn get_interaction_command(&self, id: u64) -> Option<()> {
        self.1.get(&id)
    }
}
