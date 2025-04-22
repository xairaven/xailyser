use crossbeam::channel::{Receiver, Sender, TrySendError};

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

    pub fn add_receiver(&mut self) -> Receiver<T> {
        let (tx, rx) = crossbeam::channel::unbounded::<T>();
        self.senders.push(tx);
        rx
    }

    pub fn send(&mut self, msg: T) {
        let mut disconnected: Vec<usize> = Vec::new();
        for (index, sender) in self.senders.iter().enumerate() {
            let result = sender.try_send(msg.clone());
            if let Err(TrySendError::Disconnected(_)) = result {
                disconnected.push(index);
            }
        }
        disconnected.reverse();
        for index in disconnected {
            self.senders.remove(index);
        }
    }

    pub fn subscribers(&self) -> usize {
        self.senders.len()
    }
}
