use rust_stemmers::{Algorithm, Stemmer};

extern crate csv;
extern crate serde;

use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use crate::structs::{Review, PreprocessedReview};

pub fn remove_punctuation(text: &str) -> String {
    text.chars().filter(|c| !c.is_ascii_punctuation()).collect()
}

pub fn to_lowercase(text: &str) -> String {
    text.to_lowercase()
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|word| word.to_string())
        .collect()
}

pub fn remove_stopwords(tokens: &[String], stopwords: &HashSet<String>) -> Vec<String> {
    tokens
        .iter()
        .filter(|token| !stopwords.contains(token.as_str()))
        .cloned()
        .collect()
}

pub fn stem(tokens: &[String]) -> Vec<String> {
    let stemmer = Stemmer::create(Algorithm::English);
    tokens
        .iter()
        .map(|token| stemmer.stem(token))
        .map(|stem| stem.to_string())
        .collect()
}

pub fn preprocess() -> Result<(), Box<dyn Error>> {
    let file_path = "data.csv";
    let mut file = File::open(file_path)?;

    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(file_content.as_bytes());

    let mut reviews = Vec::new();
    for result in reader.deserialize() {
        let review: Review = result?;
        reviews.push(review);
    }

    let mut preprocessed_reviews = Vec::new();

    for review in reviews.iter_mut() {
        let text = &review.review_text;
        let cleaned_text = remove_punctuation(text);
        let lowercased_text = to_lowercase(&cleaned_text);
        let tokens = tokenize(&lowercased_text);

        // Load or define your list of stopwords as a HashSet<String>.

        let stopwords: HashSet<String> = std::fs::read_to_string("stopwords.txt")?
            .lines()
            .map(|line| line.to_string())
            .collect();

        let tokens_without_stopwords = remove_stopwords(&tokens, &stopwords);
        // let stemmed_tokens = stem(&tokens_without_stopwords);

        let preprocessed_review_text = tokens_without_stopwords.join(" ");
        let preprocessed_review = PreprocessedReview {
            review_text: preprocessed_review_text,
            review_stars: review.review_stars,
        };
        preprocessed_reviews.push(preprocessed_review);
    }

    let mut writer = csv::Writer::from_writer(File::create("preprocessed_data.csv")?);
    for preprocessed_review in preprocessed_reviews {
        writer.serialize(preprocessed_review)?;
    }
    writer.flush()?;

    Ok(())
}
