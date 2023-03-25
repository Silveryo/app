use std::{error::Error, fs::File, io::Read};

use crate::structs::PreprocessedReview;

mod preprocess;
mod structs;

extern crate csv;
extern crate serde;

fn main() {
    print_csv().unwrap();
}

fn print_csv() -> Result<(), Box<dyn Error>> {
    let file_path = "preprocessed_data.csv";

    let preprocessed_reviews = load_csv(file_path)?;

    for (i, review) in preprocessed_reviews.iter().enumerate() {
        if i % 100 == 0 {
            println!("{:?}", review);
            println!();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    Ok(())
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
