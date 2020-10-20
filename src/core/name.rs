use counter::Counter;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::BTreeMap;

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
// /// A Name using NGrams suitable for matching
// #[derive(Debug, Getters)]
// pub struct NameNGrams {
//     #[getset(get = "pub")]
//     name: Name,
//     #[getset(get = "pub")]
//     // token_ngram_weight: Vec<(String, NGram, f64)>,
//     #[getset(get = "pub")]
//     total_weight: f64,
// }

// // data NGram(Vec<String>);

// impl NameNGrams {
//     pub fn new(np: NameProcessed, idf: &IDF) -> Self {
//         let mut token_count_weights: BTreeMap<String, (usize, f64)> = BTreeMap::new();
//         let mut total_weight: f64 = 0.0;

//         for (token, count) in np.token_counter.iter() {
//             let weight = idf.lookup(token);
//             token_count_weights.insert(token.to_string(), (*count, weight));

//             total_weight += (*count as f64) * weight.powi(2);
//         }

//         total_weight = total_weight.sqrt();

//         NameWeighted {
//             name: np.name,
//             // token_counter: np.token_counter,
//             token_count_weights,
//             total_weight,
//         }
//     }

//     pub fn compute_match_score(&self, to_name: &Self) -> f64 {
//         todo!();
//         // let sicore_in_common: f64 = self
//         //     .token_count_weights()
//         //     .iter()
//         //     .filter_map(|(token, (count_in_from, weight))| {
//         //         to_name
//         //             .token_count_weights()
//         //             .get(token)
//         //             .and_then(|(count_in_to, _)| {
//         //                 Some(min(*count_in_from, *count_in_to) as f64 * weight.powi(2))
//         //             })
//         //     })
//         //     .sum();

//         // score_in_common / (self.total_weight * to_name.total_weight)
//     }
// }

// /*****************************************************************************/
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