use crate::toxic_bot::ToxicBot;
use std::error::Error;
use telegram_bot::*;
use futures::StreamExt;
use tokio;
use regex::Regex;

pub struct TelegramBot<'a> {
    bot: &'a mut ToxicBot,
    api: Api,
    bot_id: UserId,
    url_regex: Regex,
}

impl TelegramBot<'_> {
    pub fn new(bot: &mut ToxicBot, token: String) -> Result<TelegramBot, Box<dyn Error>> {
        let api = Api::new(token);
        let url_regex = Regex::new(r"(http://www\.|https://www\.|http://|https://)?[a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,5}(:[0-9]{1,5})?(/.*)?")?;

        Ok(TelegramBot {
            bot,
            api,
            bot_id: UserId::new(0),
            url_regex,
        })
    }

    #[tokio::main]
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.bot_id = self.api.send(GetMe).await?.id;
        let mut stream = self.api.stream();

        while let Some(update) = stream.next().await {
            let update = update?;
            if let UpdateKind::Message(message) = update.kind {
                match message.kind {
                    MessageKind::Text { ref data, .. } => {
                        self.handle_message(&message, &data).await?
                    }
                    MessageKind::Photo { ref caption, .. } => {
                        self.handle_message(
                            &message,
                            caption.as_ref().unwrap_or(&"".to_string()),
                        ).await?
                    }
                    MessageKind::NewChatMembers { .. } => {
                        self.send_response(&message, "").await?;
                    }
                    kind => log::info!("update of kind {:?} was ignored", kind)
                }
            }
        }

        Ok(())
    }

    async fn handle_message(
        &mut self, message: &Message, text: &str,
    ) -> Result<(), Box<dyn Error>> {
        log::info!("got message {:?}", message);

        if let Some(reply) = &message.reply_to_message {
            if let MessageOrChannelPost::Message(reply) = *reply.clone() {
                // if it's a reply to bot's message
                if reply.from.id == self.bot_id {
                    self.send_response(message, text).await?;
                    return Ok(());
                }
            }
        }

        if self.is_triggered_by_message(text) {
            self.send_response(message, text).await?;
            return Ok(());
        }

        Ok(())
    }

    async fn send_response(
        &mut self, message: &Message, text: &str,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.bot.get_response(text);
        self.api.send(message.text_reply(response)).await?;
        Ok(())
    }

    fn is_triggered_by_message(&self, text: &str) -> bool {
        self.url_regex.is_match(text)
    }
}
