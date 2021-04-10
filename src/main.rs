#[macro_use]
extern crate lazy_static;
use std::env;

use futures::StreamExt;
use telegram_bot::*;
use regex::Regex;
use rand::{self, Rng};
use if_chain::*;

async fn check_message(api: Api, message: Message, data: &str, bot_name: String) -> Result<(), Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(http://www\.|https://www\.|http://|https://)?[a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,5}(:[0-9]{1,5})?(/.*)?").unwrap();
        static ref VS: Vec<&'static str> = vec!["Вилкой в глаз или в жопу раз?", "Когда на небе была очередь за умом или сиськами, ты стоял в очереди за пирожком", "Впервые вижу членовагину", "Do you like сосать?"];
    }
    // Print received text message to stdout.
    println!("<{}>: {}", &message.from.first_name, data);
    if_chain! {
        if let Some(ref reply_to_message) = message.reply_to_message;
        if let MessageOrChannelPost::Message(mess) = &**reply_to_message;
        if mess.from.username.as_ref().unwrap_or(&"".to_owned()) == &bot_name;
        then {
                api.send(message.text_reply(VS[rand::thread_rng().gen_range(0..VS.len())]))
                .await?;
        } else {
            if RE.is_match(data) {
                api.send(message.text_reply(VS[rand::thread_rng().gen_range(0..VS.len())]))
                .await?;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let bot_name = env::var("TELEGRAM_BOT_USERNAME").expect("TELEGRAM_BOT_USERNAME not set");
    let api = Api::new(token);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update?;
        println!("{:?}", update);
        if let UpdateKind::Message(message) = update.kind {
            let api_copy = api.clone();
            let message_copy = message.clone();
            let bot_name_clone = bot_name.clone();
            match message.kind {
                MessageKind::Text { ref data, .. } => check_message(api_copy, message_copy, data, bot_name_clone).await?,
                MessageKind::Photo { ref caption, .. } => {
                    if let Some(ref data) = caption {
                        check_message(api_copy, message_copy, data, bot_name_clone).await?
                    }
                },
                MessageKind::Video { ref caption, .. } => {
                    if let Some(ref data) = caption {
                        check_message(api_copy, message_copy, data, bot_name_clone).await?
                    }
                },
                _ => continue,
            }
        }
    }
    Ok(())
}
