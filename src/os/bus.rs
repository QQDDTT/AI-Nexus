use tokio::sync::mpsc;
use crate::core::interfaces::AcpMessage;

pub struct NexusBus {
    sender: mpsc::Sender<AcpMessage>,
    receiver: Option<mpsc::Receiver<AcpMessage>>,
}

impl NexusBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: Some(receiver),
        }
    }

    pub fn get_sender(&self) -> mpsc::Sender<AcpMessage> {
        self.sender.clone()
    }

    pub fn take_receiver(&mut self) -> Option<mpsc::Receiver<AcpMessage>> {
        self.receiver.take()
    }
}
