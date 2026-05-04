use crate::types::ScalingMode;
use parking_lot::Mutex;
use std::collections::VecDeque;

pub enum Command {
    Start {
        name: String,
        interval_seconds: f64,
        shuffle: bool,
        loop_enabled: bool,
        image_paths: Vec<String>,
    },
    Stop,
    Pause,
    Resume,
    Next,
    Previous,
    SetScalingMode(ScalingMode),
    Shutdown,
}

pub struct CommandQueue {
    queue: Mutex<VecDeque<Command>>,
    capacity: usize,
}

impl CommandQueue {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn push(&self, cmd: Command) -> Option<()> {
        let mut queue = self.queue.lock();
        if queue.len() >= self.capacity {
            return None;
        }
        queue.push_back(cmd);
        drop(queue);
        Some(())
    }

    pub fn pop(&self) -> Option<Command> {
        self.queue.lock().pop_front()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.lock().len()
    }
}
