use rayon::prelude::*;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, error::Error, fs::File, io::Read};

use crate::structs::PreprocessedReview;

mod preprocess;
mod structs;

extern crate csv;
extern crate serde;

fn main() {
    let num_rows = 20000;
    let (dictionary, review_vectors) = bag_of_words(num_rows);

    let mut mapped_dictionary =
        map_dictionary_to_review_vectors(&dictionary, &review_vectors, None);

    // Sort the mapped_dictionary by count in descending order
    mapped_dictionary.sort_by(|a, b| b.1.cmp(&a.1));

    // Write the sorted mapped_dictionary to a file
    let file_path = "bag_of_words.csv";
    let mut file = std::fs::File::create(file_path).expect("Unable to create file");

    // Write the column labels
    writeln!(&mut file, "weight,word").expect("Unable to write column labels");

    // Write the data
    for (word, count) in mapped_dictionary.iter() {
        writeln!(&mut file, "{},{}", count, word).expect("Unable to write data");
    }

    println!("Bag of words data saved to {}", file_path);
}

fn bag_of_words(num_rows: usize) -> (Vec<String>, Vec<Vec<u32>>) {
    let file_path = "preprocessed_data.csv";
    let preprocessed_reviews = load_csv(file_path).unwrap();

    let word_counts: Arc<Mutex<HashMap<String, u32>>> = Arc::new(Mutex::new(HashMap::new()));

    preprocessed_reviews
        .par_iter()
        .take(num_rows)
        .enumerate()
        .for_each(|(_, review)| {
            let mut local_word_counts = HashMap::new();
            let words = review.review_text.split_whitespace();
            for word in words {
                *local_word_counts.entry(word.to_string()).or_insert(0) += 1;
            }

            let mut word_counts = word_counts.lock().unwrap();
            for (word, count) in local_word_counts {
                *word_counts.entry(word).or_insert(0) += count;
            }
        });

    let dictionary: Vec<String> = word_counts.lock().unwrap().keys().cloned().collect();

    let review_vectors: Vec<Vec<u32>> = preprocessed_reviews
        .par_iter()
        .take(num_rows)
        .enumerate()
        .map(|(_, review)| {
            let mut review_vector: Vec<u32> = vec![0; dictionary.len()];
            let words = review.review_text.split_whitespace();
            for word in words {
                if let Some(index) = dictionary.iter().position(|w| w == word) {
                    review_vector[index] += 1;
                }
            }
            review_vector
        })
        .collect();

    (dictionary, review_vectors)
}

fn map_dictionary_to_review_vectors(
    dictionary: &[String],
    review_vectors: &[Vec<u32>],
    num_reviews_to_process: Option<usize>,
) -> Vec<(String, u32)> {
    let num_reviews = num_reviews_to_process.unwrap_or_else(|| review_vectors.len());
    let mut combined_counts: Vec<u32> = vec![0; dictionary.len()];

    for review_vector in review_vectors.iter().take(num_reviews) {
        for (j, count) in review_vector.iter().enumerate() {
            combined_counts[j] += count;
        }
    }

    dictionary
        .iter()
        .cloned()
        .zip(combined_counts.into_iter())
        .collect()
}

fn load_csv(file_path: &str) -> Result<Vec<PreprocessedReview>, Box<dyn Error>> {
    let mut file = File::open(file_path)?;

    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(file_content.as_bytes());

    let mut preprocessed_reviews = Vec::new();
    for result in reader.deserialize() {
        let preprocessed_review: PreprocessedReview = result?;
        preprocessed_reviews.push(preprocessed_review);
    }

    Ok(preprocessed_reviews)
}
