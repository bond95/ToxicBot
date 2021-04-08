use std::env;

use futures::StreamExt;
use telegram_bot::*;
use regex::Regex;
use rand::{self, Rng};
use if_chain::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let bot_name = env::var("TELEGRAM_BOT_USERNAME").expect("TELEGRAM_BOT_USERNAME not set");
    let re = Regex::new(r"(http|ftp|https)://([\w_-]+(?:(?:\.[\w_-]+)+))([\w.,@?^=%&:/~+#-]*[\w@?^=%&/~+#-])?").unwrap();
    let vs = vec!["Вилкой в глаз или в жопу раз?", "Когда на небе была очередь за умом или сиськами, ты стоял в очереди за пирожком", "Впервые вижу членовагину", "Do you like сосать?"];
    let api = Api::new(token);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update?;
        println!("{:?}", update);
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);
                if_chain! {
	                if let Some(ref reply_to_message) = message.reply_to_message;
	                if let MessageOrChannelPost::Message(mess) = &**reply_to_message;
	                if mess.from.username.as_ref().unwrap_or(&"".to_owned()) == &bot_name;
	                then {
			                api.send(message.text_reply(vs[rand::thread_rng().gen_range(0..vs.len())]))
			                .await?;
	        	 	} else {
		                if re.is_match(data) {
			                api.send(message.text_reply(vs[rand::thread_rng().gen_range(0..vs.len())]))
			                .await?;
		                }
	        	 	}
	            }
            }
        }
    }
    Ok(())
}
