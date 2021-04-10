use std::collections::HashMap;
use rand::prelude::ThreadRng;
use crate::constants::*;
use rand::seq::SliceRandom;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufRead};
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use lingua::Language::{English, Czech, Russian};

pub struct ToxicBot {
    insults: HashMap<Language, Vec<String>>,
    rng: ThreadRng,
    language_detector: LanguageDetector,
}

impl ToxicBot {
    pub fn new() -> ToxicBot {
        let language_detector = LanguageDetectorBuilder::from_languages(
            &vec![English, Czech, Russian],
        ).build();

        ToxicBot {
            insults: HashMap::new(),
            rng: rand::thread_rng(),
            language_detector,
        }
    }

    pub fn get_response(&mut self, message: &str) -> String {
        let lang = self.language_detector
            .detect_language_of(message)
            .map_or_else(|| {
                log::warn!(
                    "unable to detect language for message '{}', using the default language '{:?}'",
                    message, DEFAULT_INSULT_LANGUAGE,
                );
                DEFAULT_INSULT_LANGUAGE
            }, |l| {
                log::info!("detected language for message '{}' as '{:?}'", message, l);
                l
            });

        match self.insults.get(&lang) {
            // if the language exists, take a random message from its dataset
            Some(insults) => insults.choose(&mut self.rng).unwrap().clone(),
            None => {
                // choose random language if the requested language doesn't exist
                let keys: Vec<Language> = self.insults.keys().map(|l| l.clone()).collect();
                if keys.len() == 0 {
                    // no languages at all
                    return INSULT_NOT_FOUND_ANSWER.to_string();
                }

                let random_key = keys.choose(&mut self.rng).unwrap().clone();
                let insults = self.insults.get(&random_key).unwrap();

                insults.choose(&mut self.rng).unwrap().clone()
            }
        }
    }

    pub fn load_slice_with_insults<T: ToString>(&mut self, language: Language, data: &[T]) {
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

fn language_str_to_enum(language: &str) -> Language {
    match language {
        "eng" => English,
        "rus" => Russian,
        "ces" => Czech,
        _ => panic!("unknown language {}", language),
    }
}

#[cfg(test)]
mod tests {
    use crate::toxic_bot::ToxicBot;
    use lingua::Language;

    #[test]
    fn toxic_bot_test() {
        let mut bot = ToxicBot::new();

        let russian_insults = vec!["ты пидор", "мудак", "иди нахуй"];
        let english_insults = vec!["fuck you", "you're moron", "asshole!"];

        bot.load_slice_with_insults(Language::Russian, &russian_insults);
        bot.load_slice_with_insults(Language::English, &english_insults);

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
