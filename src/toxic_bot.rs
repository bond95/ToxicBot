use whatlang::Lang;
use std::collections::HashMap;
use rand::prelude::ThreadRng;
use crate::constants::*;
use rand::seq::SliceRandom;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufRead};

pub struct ToxicBot {
    insults: HashMap<Lang, Vec<String>>,
    rng: ThreadRng,
}

impl ToxicBot {
    pub fn new() -> ToxicBot {
        ToxicBot {
            insults: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }

    pub fn get_response(&mut self, message: &str) -> String {
        let lang = whatlang::detect(message).map_or_else(|| {
            log::warn!("unable to detect language for message '{}'", message);
            DEFAULT_INSULT_LANGUAGE
        }, |i| {
            log::info!("detected language for message '{}' as '{:?}'", message, i);
            i.lang()
        });

        match self.insults.get(&lang) {
            // if the language exists, take a random message from its dataset
            Some(insults) => insults.choose(&mut self.rng).unwrap().clone(),
            None => {
                // choose random language if the requested language doesn't exist
                let keys: Vec<Lang> = self.insults.keys().map(|l| *l).collect();
                if keys.len() == 0 {
                    // no languages at all
                    return INSULT_NOT_FOUND_ANSWER.to_string();
                }

                let random_key = *keys.choose(&mut self.rng).unwrap();
                let insults = self.insults.get(&random_key).unwrap();

                insults.choose(&mut self.rng).unwrap().clone()
            }
        }
    }

    pub fn load_slice_with_insults<T: ToString>(&mut self, language: Lang, data: &[T]) {
        let data = data.iter().map(|x| x.to_string());
        match self.insults.get_mut(&language) {
            Some(insults) => insults.extend(data),
            None => { self.insults.insert(language, data.collect()); }
        }
    }

    pub fn load_dir_with_insults(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
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
                self.load_file_with_insults(language_name, data_file)?
            }
        }

        Ok(())
    }

    fn load_file_with_insults(&mut self, language: &str, file_path: &str) -> Result<(), Box<dyn Error>> {
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

        self.load_slice_with_insults(language_str_to_enum(language), &lines);

        Ok(())
    }
}

fn language_str_to_enum(language: &str) -> Lang {
    match language {
        "eng" => Lang::Eng,
        "rus" => Lang::Rus,
        "ces" => Lang::Ces,
        _ => panic!("unknown language {}", language),
    }
}

#[cfg(test)]
mod tests {
    use whatlang::Lang;
    use crate::toxic_bot::ToxicBot;

    #[test]
    fn toxic_bot_test() {
        let mut bot = ToxicBot::new();

        let russian_insults = vec!["ты пидор", "мудак", "иди нахуй"];
        let english_insults = vec!["fuck you", "you're moron", "asshole!"];

        bot.load_dataset_of_insults(Lang::Rus, &russian_insults);
        bot.load_dataset_of_insults(Lang::Eng, &english_insults);

        let resp = bot.get_response("привет");
        assert!(
            russian_insults.iter().any(|x| *x == resp),
            "message '{}' should be from russian dataset", resp,
        );

        let resp = bot.get_response("Hi there");
        assert!(
            english_insults.iter().any(|x| *x == resp),
            "message '{}' should be from english dataset", resp,
        );
    }
}
