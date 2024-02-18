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

pub struct Replies(Cache<u64, Reply>);

impl Replies {
    pub fn new() -> Self {
        Self(
            Cache::builder()
                .max_capacity(1000)
                .time_to_idle(Duration::from_secs(60 * 5))
                .build(),
        )
    }

    pub fn insert(&self, id: u64, reply: Reply) {
        self.0.insert(id, reply);
    }

    pub fn remove(&self, id: u64) -> Option<Reply> {
        self.0.remove(&id)
    }

    pub fn get(&self, id: u64) -> Option<Reply> {
        self.0.get(&id)
    }
}
