use counter::Counter;
use getset::Getters;
use itertools::iproduct;
use strsim::{normalized_damerau_levenshtein, normalized_levenshtein};

use crate::core::idf::Idf;
use crate::core::name::base::NameProcessed;
use crate::core::name::score_combination_queue;

/*****************************************************************************/
/* Levenshtein name for approximate  matching                                */
/*****************************************************************************/
/// A Name using Levenshtein distance suitable for matching
#[derive(Debug, Getters)]
pub struct NameLevenshtein<N> {
    #[getset(get = "pub")]
    name: N,
    #[getset(get = "pub")]
    token_counter: Counter<String>,
    #[getset(get = "pub")]
    token_weights: Vec<(String, f64)>,
    #[getset(get = "pub")]
    norm: f64,
}

impl<N> NameLevenshtein<N> {
    pub fn new(np: NameProcessed<N>, idf: &Idf) -> Self {
        let token_counter: Counter<String> = np.token_counter;
        let mut token_weights = vec![];
        let mut total_weight: f64 = 0.0;

        for (token, count) in token_counter.iter() {
            let weight = idf.lookup(token);
            token_weights.push((token.to_string(), weight));

            total_weight += (*count as f64) * weight.powi(2);
        }

        NameLevenshtein {
            name: np.name,
            token_counter,
            token_weights,
            norm: total_weight.sqrt(),
        }
    }

    pub fn compute_match_score(&self, to_name: &Self) -> f64 {
        let mut combination_queue: Vec<_> = vec![];

        for ((from_token, from_weight), (to_token, to_weight)) in
            iproduct!(&self.token_weights, &to_name.token_weights)
        {
            combination_queue.push((
                normalized_levenshtein(from_token, to_token) * from_weight * to_weight,
                from_token,
                to_token,
            ))
        }

        score_combination_queue(
            &self.token_counter,
            self.norm,
            &to_name.token_counter,
            to_name.norm,
            combination_queue,
        )
    }
}

/*****************************************************************************/
/* Damerau-Levenshtein name for approximate  matching                        */
/*****************************************************************************/
#[derive(Debug, Getters)]
pub struct NameDamerauLevenshtein<N> {
    #[getset(get = "pub")]
    name: N,
    #[getset(get = "pub")]
    token_counter: Counter<String>,
    #[getset(get = "pub")]
    token_weights: Vec<(String, f64)>,
    #[getset(get = "pub")]
    norm: f64,
}

impl<N> NameDamerauLevenshtein<N> {
    pub fn new(np: NameProcessed<N>, idf: &Idf) -> Self {
        let token_counter: Counter<String> = np.token_counter;
        let mut token_weights = vec![];
        let mut total_weight: f64 = 0.0;

        for (token, count) in token_counter.iter() {
            let weight = idf.lookup(token);
            token_weights.push((token.to_string(), weight));

            total_weight += (*count as f64) * weight.powi(2);
        }

        NameDamerauLevenshtein {
            name: np.name,
            token_counter,
            token_weights,
            norm: total_weight.sqrt(),
        }
    }

    pub fn compute_match_score(&self, to_name: &Self) -> f64 {
        let mut combination_queue: Vec<_> = vec![];

        for ((from_token, from_weight), (to_token, to_weight)) in
            iproduct!(&self.token_weights, &to_name.token_weights)
        {
            combination_queue.push((
                normalized_damerau_levenshtein(from_token, to_token) * from_weight * to_weight,
                from_token,
                to_token,
            ))
        }

        score_combination_queue(
            &self.token_counter,
            self.norm,
            &to_name.token_counter,
            to_name.norm,
            combination_queue,
        )
    }
}
