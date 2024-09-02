use getset::Getters;
use std::cmp::min;
use std::collections::BTreeMap;

use crate::core::idf::Idf;
use crate::name::base::{HasName, NameProcessed};

/*****************************************************************************/
/* Weighted name for exact token matching                                    */
/*****************************************************************************/
/// A weighted Name suitable for matching
#[derive(Debug, Getters)]
pub struct NameWeighted<N> {
    #[getset(get = "pub")]
    name: N,
    #[getset(get = "pub")]
    token_count_weights: BTreeMap<String, (usize, f64)>,
    #[getset(get = "pub")]
    norm: f64,
}

impl<N> HasName<N> for NameWeighted<N> {
    fn get_name(&self) -> &N {
        &self.name
    }
}

impl<N> NameWeighted<N> {
    pub fn new(np: NameProcessed<N>, idf: &Idf) -> Self {
        let mut token_count_weights: BTreeMap<String, (usize, f64)> = BTreeMap::new();
        let mut total_weight: f64 = 0.0;

        for (token, count) in np.token_counter.iter() {
            let weight = idf.lookup(token);
            token_count_weights.insert(token.to_string(), (*count, weight));

            total_weight += (*count as f64) * weight.powi(2);
        }

        NameWeighted {
            name: np.name,
            token_count_weights,
            norm: total_weight.sqrt(),
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
                    .map(|(count_in_to, _)| {
                        min(*count_in_from, *count_in_to) as f64 * weight.powi(2)
                    })
            })
            .sum();

        score_in_common / (self.norm * to_name.norm)
    }
}
