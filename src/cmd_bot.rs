use crate::toxic_bot::ToxicBot;
use std::io::BufRead;

pub fn run(bot: &mut ToxicBot) {
    println!("Write something and press Enter: ");
    for line in std::io::stdin().lock().lines() {
        let resp = bot.get_response(&line.unwrap());
        println!("- {}", resp);
    }
}
