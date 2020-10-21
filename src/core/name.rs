use counter::Counter;
use getset::Getters;
use itertools::iproduct;
use ngrams::Ngram;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;

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
        let file = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file);

        rdr.deserialize()
            .into_iter()
            .map(|result| {
                let record: Name = result?;
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
    pub fn new(name: Name, token_counter: Counter<String>) -> Self {
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
            let ngrams_in_common: usize = from_ngram
                .0
                .iter()
                .map(|(ng, count_in_from)| {
                    let count_in_to = to_ngram.0.get(ng).unwrap_or(&0);
                    min(count_in_from, count_in_to)
                })
                .sum();
            combination_queue.push((
                ngrams_in_common as f64 * from_weight * to_weight,
                from_token,
                to_token,
            ))
        }

        let mut sorted_combination_queue = combination_queue;
        sorted_combination_queue.sort_unstable_by(|(val_a, _, _), (val_b, _, _)| {
            val_a.partial_cmp(&val_b).unwrap().reverse()
        });

        let mut score_in_common = 0.0;
        let mut from_tokens_used: Counter<String> = Counter::new();
        let mut to_tokens_used: Counter<String> = Counter::new();

        while !sorted_combination_queue.is_empty() {
            let (this_score, from_token, to_token) = sorted_combination_queue.pop().unwrap();
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
        score_in_common / (self.total_weight * to_name.total_weight)
    }
}

/*****************************************************************************/
/* NGram related                                                             */
/*****************************************************************************/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NGram(Counter<String>);

fn n_gram(s: String, window_size: usize) -> NGram {
    if window_size <= 1 {
        panic!("preprocess::PrepString.n_gram requires a window_size of 2 or greater")
    } else {
        NGram(
            s.chars()
                .ngrams(window_size)
                .pad()
                .map(|cs| mconcat_chars(cs))
                .collect(),
        )
    }
}

fn mconcat_chars(cs: Vec<char>) -> String {
    cs.iter().collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn n_grams() {
        let s = "abcd".to_string();
        assert_eq!(
            NGram(
                vec!(
                    "\u{2060}a".to_string(),
                    "ab".to_string(),
                    "bc".to_string(),
                    "cd".to_string(),
                    "d\u{2060}".to_string()
                )
                .into_iter()
                .collect()
            ),
            n_gram(s, 2)
        );
        let s = "abcd".to_string();
        assert_eq!(
            NGram(
                vec!(
                    "\u{2060}\u{2060}a".to_string(),
                    "\u{2060}ab".to_string(),
                    "abc".to_string(),
                    "bcd".to_string(),
                    "cd\u{2060}".to_string(),
                    "d\u{2060}\u{2060}".to_string(),
                )
                .into_iter()
                .collect()
            ),
            n_gram(s, 3)
        );
        let s = "abcd".to_string();
        assert_eq!(
            NGram(
                vec!(
                    "\u{2060}\u{2060}\u{2060}a".to_string(),
                    "\u{2060}\u{2060}ab".to_string(),
                    "\u{2060}abc".to_string(),
                    "abcd".to_string(),
                    "bcd\u{2060}".to_string(),
                    "cd\u{2060}\u{2060}".to_string(),
                    "d\u{2060}\u{2060}\u{2060}".to_string(),
                )
                .into_iter()
                .collect()
            ),
            n_gram(s, 4)
        );
    }
} // /*****************************************************************************/
  // /* Tests                                                                     */
  // /*****************************************************************************/
  // #[cfg(test)]
  // mod test {
  //     use super::*;

//     #[test]
//     fn name_same_group() {
//         let n1 = Name {
//             unprocessed: "John Smith".to_string(),
//             idx: "id1".to_string(),
//             group: "Group A".to_string(),
//         };

//         let n2 = Name {
//             unprocessed: "Smith Smithington".to_string(),
//             idx: "id2".to_string(),
//             group: "Group A".to_string(),
//         };
//         let n3 = Name {
//             unprocessed: "Smith Smithington".to_string(),
//             idx: "id3".to_string(),
//             group: "No group".to_string(),
//         };
//         assert_eq!(n1.same_group(&n2), true);
//         assert_eq!(n1.same_group(&n3), false);
//         assert_eq!(n2.same_group(&n3), false);

//         // #[test]
//         // fn test_item_removal() {
//         //     let mut c = NonZeroTokenCounter::new();
//         //     c = c.add_item(&"the".to_string());
//         //     c = c.add_item(&"quick".to_string());
//         //     c = c.add_item(&"fox".to_string());
//         //     c = c.add_item(&"fox".to_string());

//         //     assert_eq!(c.0[&"the".to_string()], 1);
//         //     assert_eq!(c.0[&"quick".to_string()], 1);
//         //     assert_eq!(c.0[&"fox".to_string()], 2);

//         //     println!("{:?}", c);
//         // }
//     }
// }
