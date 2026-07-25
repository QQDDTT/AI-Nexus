use crate::core::interfaces::{Channel, MessageContent};
use crate::utils::errors::AiNexusError;
use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::types::Message;
use tokio::sync::mpsc;
use tracing::{error, info};

pub struct TelegramChannel {
    bot: Bot,
    rx: tokio::sync::Mutex<mpsc::UnboundedReceiver<(String, String)>>, // (user_id/chat_id, text)
}

impl TelegramChannel {
    pub fn new(token: &str) -> Self {
        let bot = Bot::new(token);
        let (tx, rx) = mpsc::unbounded_channel();
        
        let bot_clone = bot.clone();
        tokio::spawn(async move {
            info!("Starting Telegram Bot Polling...");
            teloxide::repl(bot_clone, move |_: Bot, msg: Message| {
                let tx_clone = tx.clone();
                async move {
                    if let Some(text) = msg.text() {
                        let chat_id = msg.chat.id.to_string();
                        let _ = tx_clone.send((chat_id, text.to_string()));
                    }
                    Ok(())
                }
            }).await;
        });

        Self {
            bot,
            rx: tokio::sync::Mutex::new(rx),
        }
    }
}

#[async_trait]
impl Channel for TelegramChannel {
    fn channel_name(&self) -> &str {
        "Telegram"
    }

    async fn receive_input(&self) -> Result<Vec<MessageContent>, AiNexusError> {
        let mut rx_lock = self.rx.lock().await;
        match rx_lock.recv().await {
            Some((chat_id, text)) => {
                // To pass chat_id context back and forth, we might need a structured way.
                // For now, returning it as a special formatted text or relying on the agent to handle it.
                // In a real implementation, the Channel trait might need `receive_input` to return the `user_id` as well.
                // As a workaround for the current trait:
                Ok(vec![MessageContent::Text(format!("{}|{}", chat_id, text))])
            }
            None => Err(AiNexusError::General("Telegram channel closed".to_string())),
        }
    }

    async fn send_reply(&self, target_user: &str, contents: Vec<MessageContent>) -> Result<(), AiNexusError> {
        if let Ok(chat_id) = target_user.parse::<i64>() {
            let chat_id = teloxide::types::ChatId(chat_id);
            for content in contents {
                match content {
                    MessageContent::Text(text) => {
                        if let Err(e) = self.bot.send_message(chat_id, text).await {
                            error!("Failed to send telegram message: {}", e);
                        }
                    }
                    _ => {
                        info!("Unsupported message type for Telegram yet.");
                    }
                }
            }
        } else {
            error!("Invalid target_user for Telegram: {}", target_user);
        }
        Ok(())
    }
}
