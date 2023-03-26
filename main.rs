use crossbeam::channel;
use rayon::prelude::*;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, error::Error, fs::File, io::Read};
use tf_idf::calculate_tfidf;

use crate::structs::PreprocessedReview;

mod preprocess;
mod structs;
mod tf_idf;

extern crate csv;
extern crate serde;

fn main() {
    print!("Hello world...");
}

fn get_tfidf() {
    let num_rows = 5;
    let dictionary_file_path = "bag_of_words.csv";
    let reviews_file_path = "preprocessed_data.csv";

    println!("Loading dictionary...");
    let dictionary = load_dictionary(dictionary_file_path).unwrap();
    println!("Loading preprocessed reviews...");
    let preprocessed_reviews = load_csv(reviews_file_path).unwrap();

    let stdout_mutex = Arc::new(Mutex::new(io::stdout()));

    println!("Converting reviews to vectors...");
    let review_vectors: Vec<Vec<u32>> = preprocessed_reviews
        .par_iter()
        .map(|review| {
            let mut stdout = stdout_mutex.lock().unwrap();
            writeln!(
                stdout,
                "Converting review to vector: {}",
                review.review_text
            )
            .unwrap();
            drop(stdout);

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

    let filtered_dictionary = dictionary;
    let filtered_review_vectors = review_vectors;

    println!("Calculating tf-idf...");
    let tfidf_vectors = calculate_tfidf(&filtered_review_vectors);

    println!("Writing tf-idf output to file...");
    let output_file_path = "tfidf_output.csv";
    let output_file = std::fs::File::create(output_file_path).expect("Unable to create file");

    let stdout_mutex = Arc::new(Mutex::new(io::stdout()));

    crossbeam::scope(|s| {
        let (tx, rx) = channel::bounded(tfidf_vectors.len());

        tfidf_vectors
            .par_iter()
            .enumerate()
            .for_each_with(tx, |tx, (i, tfidf_vector)| {
                let mut buf = String::new();
                buf.push_str(&format!("Review {}\n", i));

                for (j, tfidf_value) in tfidf_vector.iter().enumerate() {
                    buf.push_str(&format!("{},{}\n", filtered_dictionary[j], tfidf_value));
                }

                buf.push('\n');
                tx.send((i, buf)).expect("Unable to send data");

                let mut stdout = stdout_mutex.lock().unwrap();
                writeln!(
                    stdout,
                    "Writing tf-idf for review {}... ({}%)",
                    i,
                    (i as f64 / tfidf_vectors.len() as f64) * 100.0
                )
                .unwrap();
                drop(stdout);
            });

        let mut output_file = BufWriter::new(output_file);
        let mut collected_data: Vec<(usize, String)> = rx.iter().collect();
        collected_data.sort_by_key(|x| x.0);

        for (_, data) in collected_data.into_iter() {
            output_file
                .write_all(data.as_bytes())
                .expect("Unable to write data");
        }
    })
    .unwrap();

    println!("Tf-idf output saved to {}", output_file_path);
}

fn get_bag_of_words() {
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

fn filter_top_n_words(
    dictionary: &Vec<String>,
    review_vectors: &Vec<Vec<u32>>,
    n: usize,
) -> (Vec<String>, Vec<Vec<u32>>) {
    let mut indices: Vec<usize> = (0..dictionary.len()).collect();
    indices
        .sort_by_key(|&i| std::cmp::Reverse(review_vectors.par_iter().map(|v| v[i]).sum::<u32>()));
    println!("Finished sorting indices.");

    indices.truncate(n);
    println!("Finished truncating indices.");

    let new_dictionary: Vec<String> = indices.par_iter().map(|&i| dictionary[i].clone()).collect();
    println!("Finished creating new dictionary.");

    let new_review_vectors: Vec<Vec<u32>> = review_vectors
        .par_iter()
        .map(|review_vector| indices.par_iter().map(|&i| review_vector[i]).collect())
        .collect();

    (new_dictionary, new_review_vectors)
}

fn load_dictionary(file_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut dictionary = Vec::new();

    for line in reader.lines().skip(1) {
        let line = line?;
        let columns: Vec<&str> = line.split(',').collect();
        if columns.len() == 2 {
            dictionary.push(columns[1].to_string());
        }
    }

    Ok(dictionary)
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
