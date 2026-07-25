use crate::core::interfaces::{Channel, MessageContent};
use crate::utils::errors::AiNexusError;
use async_trait::async_trait;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tracing::info;

use tokio::sync::mpsc;

pub struct LocalTerminalChannel {
    pub user_id: String,
    pub tx: mpsc::UnboundedSender<String>,
    pub rx: tokio::sync::Mutex<mpsc::UnboundedReceiver<String>>,
}

impl LocalTerminalChannel {
    pub fn new(user_id: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { 
            user_id,
            tx,
            rx: tokio::sync::Mutex::new(rx),
        }
    }
}


#[async_trait]
impl Channel for LocalTerminalChannel {
    fn channel_name(&self) -> &str {
        "LocalTerminal"
    }

    async fn receive_input(&self) -> Result<Vec<MessageContent>, AiNexusError> {
        info!("[LocalTerminal] Enter your message (type /exit to quit):");
        
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        let mut rx_lock = self.rx.lock().await;

        tokio::select! {
            result = reader.read_line(&mut line) => {
                result.map_err(|e| {
                    AiNexusError::General(format!("Failed to read stdin: {}", e))
                })?;
            }
            Some(intent) = rx_lock.recv() => {
                info!("[Scheduler Intent Received]: {}", intent);
                return Ok(vec![MessageContent::Text(intent)]);
            }
        }

        let input = line.trim().to_string();
        
        // 预留 /attach 附件支持
        if input.starts_with("/attach ") {
            info!("Attachment mock recognized, but not fully implemented.");
        }

        Ok(vec![MessageContent::Text(input)])
    }

    async fn send_reply(&self, _target_user: &str, contents: Vec<MessageContent>) -> Result<(), AiNexusError> {
        for content in contents {
            match content {
                MessageContent::Text(text) => {
                    info!("[LocalTerminal Reply]: {}", text);
                }
                MessageContent::Image { .. } => {
                    info!("[LocalTerminal Reply]: <Image received>");
                }
                MessageContent::Audio { .. } => {
                    info!("[LocalTerminal Reply]: <Audio received>");
                }
                MessageContent::Document { .. } => {
                    info!("[LocalTerminal Reply]: <Document received>");
                }
            }
        }
        Ok(())
    }
}
