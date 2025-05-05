use crossbeam::channel::{Receiver, Sender, TrySendError};
use std::collections::VecDeque;

pub struct BroadcastPool<T: Clone> {
    senders: VecDeque<Sender<T>>,
    receivers: VecDeque<Receiver<T>>,
}

impl<T: Clone> Default for BroadcastPool<T> {
    fn default() -> Self {
        Self {
            senders: VecDeque::with_capacity(8),
            receivers: VecDeque::with_capacity(8),
        }
    }
}

impl<T: Clone> BroadcastPool<T> {
    pub fn create(&mut self) {
        let (tx, rx) = crossbeam::channel::unbounded::<T>();
        self.senders.push_back(tx);
        self.receivers.push_back(rx);
    }

    pub fn last_receiver(&mut self) -> Option<Receiver<T>> {
        self.receivers.pop_front()
    }

    pub fn is_receiver_ready(&self) -> bool {
        !self.receivers.is_empty()
    }

    pub fn last_sender(&mut self) -> Option<Sender<T>> {
        self.senders.pop_back()
    }

    pub fn is_sender_ready(&self) -> bool {
        !self.senders.is_empty()
    }
}

pub struct BroadcastChannel<T: Clone> {
    senders: Vec<Sender<T>>,
}

impl<T: Clone> Default for BroadcastChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> BroadcastChannel<T> {
    pub fn new() -> Self {
        Self {
            senders: Vec::with_capacity(8),
        }
    }

    pub fn add_sender(&mut self, sender: Sender<T>) {
        self.senders.push(sender);
    }

    pub fn send(&mut self, msg: T) {
        let mut disconnected: Vec<usize> = Vec::new();
        for (index, sender) in self.senders.iter().enumerate() {
            let result = sender.try_send(msg.clone());
            if let Err(TrySendError::Disconnected(_)) = result {
                disconnected.push(index);
            }
        }
        if !disconnected.is_empty() {
            disconnected.reverse();
            for index in disconnected {
                self.senders.remove(index);
            }
        }
    }

    pub fn subscribers(&self) -> usize {
        self.senders.len()
    }
}
