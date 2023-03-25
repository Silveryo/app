use serde::{Deserialize, Serialize};

extern crate serde;

#[derive(Debug, Deserialize)]
pub struct Review {
    #[serde(rename = "Review")]
    pub review_text: String,
    #[serde(rename = "Rating")]
    pub review_stars: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreprocessedReview {
    #[serde(rename = "PreprocessedReview")]
    pub review_text: String,
    #[serde(rename = "Rating")]
    pub review_stars: u8,
}
