use clap::{App, SubCommand};
use crate::toxic_bot::ToxicBot;
use std::{fs, env};
use std::fs::{File};
use crate::constants::*;
use std::error::Error;
use std::io::{BufReader, BufRead};
use whatlang::Lang;
use crate::telegram_bot::TelegramBot;

mod toxic_bot;
mod cmd_bot;
mod constants;
mod telegram_bot;

fn main() {
    env_logger::init();

    let matches = App::new("ToxicBot")
        .about("A bot that will roast you")
        .author("Bohdan Iakymets <cyberbond95@gmail.com>, Serhii Zakharov <serhii@zahar.pro>")
        .subcommand(
            SubCommand::with_name("cmd")
                .about("just for testing, or if you're feeling \
                        like you want to be roasted and don't have Telegram")
        )
        .subcommand(
            SubCommand::with_name("telegram")
                .about("to run telegram bot. \
                        You'll also need to set TELEGRAM_BOT_TOKEN env variable")
        )
        .get_matches();

    let mut bot = ToxicBot::new();

    bot.load_dir_with_insults(DEFAULT_DATA_PATH).unwrap();

    match matches.subcommand_name() {
        Some("cmd") => cmd_bot::run(&mut bot),
        Some("telegram") => {
            let token = env::var("TELEGRAM_BOT_TOKEN")
                .map_err(|_| "telegram bot token is not set")
                .unwrap();
            TelegramBot::new(&mut bot, token).unwrap().run().unwrap()
        }
        _ => println!("{}", matches.usage()),
    }
}
