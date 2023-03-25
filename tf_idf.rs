fn calculate_term_frequency(review_vector: &Vec<u32>) -> Vec<f64> {
    let total_terms = review_vector.iter().map(|&count| count as f64).sum::<f64>();
    review_vector
        .iter()
        .map(|&count| count as f64 / total_terms)
        .collect()
}

fn calculate_inverse_document_frequency(review_vectors: &Vec<Vec<u32>>) -> Vec<f64> {
    let num_documents = review_vectors.len() as f64;
    let num_documents_with_term = review_vectors
        .iter()
        .map(|review_vector| {
            review_vector
                .iter()
                .map(|&count| if count > 0 { 1 } else { 0 })
                .collect::<Vec<_>>()
        })
        .fold(vec![0; review_vectors[0].len()], |mut acc, x| {
            for (i, &val) in x.iter().enumerate() {
                acc[i] += val;
            }
            acc
        });

    num_documents_with_term
        .iter()
        .map(|&count| (num_documents / (count as f64)).ln())
        .collect()
}

fn calculate_tfidf(review_vectors: &Vec<Vec<u32>>) -> Vec<Vec<f64>> {
    let idf = calculate_inverse_document_frequency(review_vectors);

    review_vectors
        .iter()
        .map(|review_vector| {
            let tf = calculate_term_frequency(review_vector);
            tf.iter()
                .enumerate()
                .map(|(i, &tf_val)| tf_val * idf[i])
                .collect()
        })
        .collect()
}
