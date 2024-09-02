use counter::Counter;
use getset::Getters;
use itertools::iproduct;
use ngrams::Ngram;
use std::cmp::min;

use crate::core::Idf;
use crate::name::score::score_combination_queue;
use crate::name::{HasName, NameProcessed};

/*****************************************************************************/
/* Ngram name for approximate  matching                                      */
/*****************************************************************************/
/// A Name using NGrams suitable for matching
#[derive(Debug, Getters)]
pub struct NameNGrams<N> {
    #[getset(get = "pub")]
    name: N,
    #[getset(get = "pub")]
    token_counter: Counter<String>,
    #[getset(get = "pub")]
    token_ngram_weights: Vec<(String, NGram, f64)>,
    #[getset(get = "pub")]
    norm: f64,
}

impl<N> HasName<N> for NameNGrams<N> {
    fn get_name(&self) -> &N {
        &self.name
    }
}

impl<N> NameNGrams<N> {
    pub fn new(np: NameProcessed<N>, idf: &Idf, window_size: usize) -> Self {
        let token_counter: Counter<String> = np.token_counter;
        let mut token_ngram_weights = vec![];
        let mut total_weight: f64 = 0.0;

        for (token, count) in token_counter.iter() {
            let weight = idf.lookup(token);
            token_ngram_weights.push((
                token.to_string(),
                n_gram(token.to_string(), window_size),
                weight,
            ));

            total_weight += (*count as f64) * weight.powi(2);
        }

        NameNGrams {
            name: np.name,
            token_counter,
            token_ngram_weights,
            norm: total_weight.sqrt(),
        }
    }

    pub fn compute_match_score(&self, to_name: &Self) -> f64 {
        let mut combination_queue: Vec<_> = vec![];

        for ((from_token, from_ngram, from_weight), (to_token, to_ngram, to_weight)) in
            iproduct!(&self.token_ngram_weights, &to_name.token_ngram_weights)
        {
            let len_ngram_self: usize = from_ngram.n_ngrams;
            let len_ngram_to: usize = to_ngram.n_ngrams;
            let ngrams_in_common: usize = from_ngram
                .ngram_counter
                .iter()
                .map(|(ng, count_in_from)| {
                    let count_in_to = to_ngram.ngram_counter.get(ng).unwrap_or(&0);
                    min(count_in_from, count_in_to)
                })
                .sum();
            combination_queue.push((
                ngrams_in_common as f64 / (len_ngram_self as f64 * len_ngram_to as f64).sqrt()
                    * from_weight
                    * to_weight,
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
/* NGram related                                                             */
/*****************************************************************************/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NGram {
    ngram_counter: Counter<String>,
    n_ngrams: usize,
}

fn n_gram(s: String, window_size: usize) -> NGram {
    if window_size <= 1 {
        panic!("preprocess::PrepString.n_gram requires a window_size of 2 or greater")
    } else {
        let ngram_counter: Counter<String> = s
            .chars()
            .ngrams(window_size)
            .pad()
            .map(mconcat_chars)
            .collect();
        let n_ngrams = ngram_counter.values().sum();
        NGram {
            ngram_counter,
            n_ngrams,
        }
    }
}

fn mconcat_chars(cs: Vec<char>) -> String {
    cs.iter().collect()
}

////////////////////////////////////////////////////////////////////////////////
// Testing
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod test {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn n_grams() {
        let s = "abcd".to_string();
        assert_eq!(
            NGram {
                ngram_counter: vec!(
                    "\u{2060}a".to_string(),
                    "ab".to_string(),
                    "bc".to_string(),
                    "cd".to_string(),
                    "d\u{2060}".to_string()
                )
                .into_iter()
                .collect(),
                n_ngrams: 5,
            },
            n_gram(s, 2)
        );
        let s = "abcd".to_string();
        assert_eq!(
            NGram {
                ngram_counter: vec!(
                    "\u{2060}\u{2060}a".to_string(),
                    "\u{2060}ab".to_string(),
                    "abc".to_string(),
                    "bcd".to_string(),
                    "cd\u{2060}".to_string(),
                    "d\u{2060}\u{2060}".to_string(),
                )
                .into_iter()
                .collect(),
                n_ngrams: 6,
            },
            n_gram(s, 3)
        );
        let s = "abcd".to_string();
        assert_eq!(
            NGram {
                ngram_counter: vec!(
                    "\u{2060}\u{2060}\u{2060}a".to_string(),
                    "\u{2060}\u{2060}ab".to_string(),
                    "\u{2060}abc".to_string(),
                    "abcd".to_string(),
                    "bcd\u{2060}".to_string(),
                    "cd\u{2060}\u{2060}".to_string(),
                    "d\u{2060}\u{2060}\u{2060}".to_string(),
                )
                .into_iter()
                .collect(),
                n_ngrams: 7,
            },
            n_gram(s, 4)
        );
    }

    #[test]
    fn n_gram_matching() {
        let john_smith = "john smith";
        let jon_smyth = "jonnn sssmyth";

        let name_0 = Name {
            unprocessed: john_smith.to_string(),
            idx: "1".to_string(),
        };
        let name_1 = Name {
            unprocessed: jon_smyth.to_string(),
            idx: "1".to_string(),
        };

        let np_0 = NameProcessed::new(
            name_0,
            john_smith
                .to_string()
                .split_ascii_whitespace()
                .map(|t| t.to_string()),
        );
        let np_1 = NameProcessed::new(
            name_1,
            jon_smyth
                .to_string()
                .split_ascii_whitespace()
                .map(|t| t.to_string()),
        );

        let mut nps = vec![np_0, np_1];
        let idf: Idf = Idf::new(&nps);

        let ng_0 = NameNGrams::new(nps.pop().unwrap(), &idf, 2);
        let ng_1 = NameNGrams::new(nps.pop().unwrap(), &idf, 2);

        let ms = ng_0.compute_match_score(&ng_1);
        let ms_flipped = ng_1.compute_match_score(&ng_0);
        let ms_self_0 = ng_0.compute_match_score(&ng_0);
        let ms_self_1 = ng_1.compute_match_score(&ng_1);
        assert_approx_eq!(ms, (0.562536 as f64));
        assert_approx_eq!(ms_flipped, (0.562536 as f64));
        assert_approx_eq!(ms_self_0, 1.0);
        assert_approx_eq!(ms_self_1, 1.0);
    }
}
