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

    load_dir(&mut bot, DEFAULT_DATA_PATH).unwrap();

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

fn load_dir(bot: &mut ToxicBot, path: &str) -> Result<(), Box<dyn Error>> {
    for language_dir in fs::read_dir(path)? {
        let language_dir = language_dir?;
        let file_type = language_dir.file_type()?;

        // ignore files or other things that aren't dirs in ./insults_data/
        if !file_type.is_dir() {
            continue;
        }

        let language_name = language_dir.file_name();
        let language_name = language_name.to_str().unwrap();

        // ignore dirs starting with .
        if language_name.starts_with(".") {
            continue;
        }

        for data_file in fs::read_dir(language_dir.path())? {
            let data_file = data_file?;
            let data_file = data_file.path();
            let data_file = data_file.to_str().unwrap();
            load_file(bot, language_name, data_file)?
        }
    }

    Ok(())
}

fn load_file(bot: &mut ToxicBot, language: &str, file_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader.lines().map(|line| {
        let line = line.unwrap();

        if file_path.ends_with("base64") {
            let line = base64::decode(line.as_bytes()).unwrap();
            String::from_utf8(line).unwrap()
        } else {
            line
        }
    }).collect();

    bot.load_dataset_of_insults(language_str_to_enum(language), &lines);

    Ok(())
}

fn language_str_to_enum(language: &str) -> Lang {
    match language {
        "eng" => Lang::Eng,
        "rus" => Lang::Rus,
        "ces" => Lang::Ces,
        _ => panic!("unknown language {}", language),
    }
}