use counter::Counter;
use getset::Getters;
use itertools::iproduct;
use ngrams::Ngram;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;

use super::error::wrap_error;
use super::idf::{HasDocument, IDF};

/******************************************************************************/
/* Basic name types not suitable for matching                                 */
/******************************************************************************/

/// An unprocessed Name capable of serialization from/to a tabular data file.
#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct Name {
    #[getset(get = "pub")]
    #[serde(rename = "name")]
    unprocessed: String,
    #[getset(get = "pub")]
    #[serde(rename = "id", default)]
    idx: String,
    // group: String,
}

impl Name {
    pub fn from_csv(file_path: &str) -> Result<Vec<Name>, Box<dyn Error>> {
        let file =
            File::open(file_path).map_err(|e| wrap_error(e, format!("accessing {}", file_path)))?;
        let mut rdr = csv::Reader::from_reader(file);

        rdr.deserialize()
            .into_iter()
            .map(|result| {
                let record: Name = result
                    .map_err(|e| wrap_error(e, format!("reading data from {}", file_path)))?;
                Ok(record)
            })
            .collect()
    }
}

/// A processed Name with a counter for each token, use the new constructor
/// with a passed in text processing function.
#[derive(Debug, Getters)]
pub struct NameProcessed {
    #[getset(get = "pub")]
    name: Name,
    #[getset(get = "pub")]
    token_counter: Counter<String>,
}

impl NameProcessed {
    // pub fn new(name: Name, token_counter: Counter<String>) -> Self {
    //     NameProcessed {
    //         name,
    //         token_counter,
    //     }
    // }

    pub fn new<I>(name: Name, tokens: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let token_counter: Counter<String> = tokens.into_iter().collect();
        NameProcessed {
            name,
            token_counter,
        }
    }

    //     fn new<F>(name: Name, text_processor: F) -> NameProcessed
    //     where
    //         F: Fn(&str) -> Vec<String>,
    //     {
    //         let token_counter: Counter<String> =
    //             text_processor(&name.unprocessed).into_iter().collect();
    //         NameProcessed {
    //             name,
    //             token_counter,
    //         }
    //     }
}

impl HasDocument for NameProcessed {
    fn get_tokens(&self) -> Vec<&String> {
        self.token_counter().keys().into_iter().collect()
    }
}

/*****************************************************************************/
/* Weighted name for exact token matching                                    */
/*****************************************************************************/
/// A weighted Name suitable for matching
#[derive(Debug, Getters)]
pub struct NameWeighted {
    #[getset(get = "pub")]
    name: Name,
    #[getset(get = "pub")]
    token_count_weights: BTreeMap<String, (usize, f64)>,
    #[getset(get = "pub")]
    total_weight: f64,
}

impl NameWeighted {
    pub fn new(np: NameProcessed, idf: &IDF) -> Self {
        let mut token_count_weights: BTreeMap<String, (usize, f64)> = BTreeMap::new();
        let mut total_weight: f64 = 0.0;

        for (token, count) in np.token_counter.iter() {
            let weight = idf.lookup(token);
            token_count_weights.insert(token.to_string(), (*count, weight));

            total_weight += (*count as f64) * weight.powi(2);
        }

        total_weight = total_weight.sqrt();

        NameWeighted {
            name: np.name,
            // token_counter: np.token_counter,
            token_count_weights,
            total_weight,
        }
    }

    pub fn compute_match_score(&self, to_name: &Self) -> f64 {
        let score_in_common: f64 = self
            .token_count_weights()
            .iter()
            .filter_map(|(token, (count_in_from, weight))| {
                to_name
                    .token_count_weights()
                    .get(token)
                    .and_then(|(count_in_to, _)| {
                        Some(min(*count_in_from, *count_in_to) as f64 * weight.powi(2))
                    })
            })
            .sum();

        score_in_common / (self.total_weight * to_name.total_weight)
    }
}

/*****************************************************************************/
/* Ngram name for approximate  matching                                      */
/*****************************************************************************/
/// A Name using NGrams suitable for matching
#[derive(Debug, Getters)]
pub struct NameNGrams {
    #[getset(get = "pub")]
    name: Name,
    #[getset(get = "pub")]
    token_counter: Counter<String>,
    #[getset(get = "pub")]
    token_ngram_weights: Vec<(String, NGram, f64)>,
    #[getset(get = "pub")]
    total_weight: f64,
}

impl NameNGrams {
    pub fn new(np: NameProcessed, idf: &IDF, window_size: usize) -> Self {
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

        total_weight = total_weight.sqrt();

        NameNGrams {
            name: np.name,
            token_counter,
            token_ngram_weights,
            total_weight,
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

        // A list of matches sort from worst to best. The algorithm will
        // pop off the last value, getting the best possible unused token match.
        let mut sorted_combination_queue = combination_queue;
        sorted_combination_queue
            .sort_unstable_by(|(val_a, _, _), (val_b, _, _)| val_a.partial_cmp(&val_b).unwrap());

        let mut score_in_common = 0.0;
        let mut from_tokens_used: Counter<String> = Counter::new();
        let mut to_tokens_used: Counter<String> = Counter::new();

        // println!("{:?}", sorted_combination_queue);

        while !sorted_combination_queue.is_empty() {
            let (this_score, from_token, to_token) = sorted_combination_queue.pop().unwrap();
            // println!("{} {} {}", this_score, from_token, to_token);
            score_in_common += this_score;

            from_tokens_used[from_token] += 1;
            to_tokens_used[to_token] += 1;

            if from_tokens_used[from_token] == self.token_counter[from_token] {
                sorted_combination_queue = sorted_combination_queue
                    .into_iter()
                    .filter(|(_, ft, _)| ft != &from_token)
                    .collect();
            }

            if to_tokens_used[to_token] == self.token_counter[to_token] {
                sorted_combination_queue = sorted_combination_queue
                    .into_iter()
                    .filter(|(_, _, tt)| tt != &to_token)
                    .collect();
            }
        }

        // println!("{:?}", score_in_common);
        score_in_common / (self.total_weight * to_name.total_weight)
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
            .map(|cs| mconcat_chars(cs))
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
        let idf: IDF = IDF::new(&nps);

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
