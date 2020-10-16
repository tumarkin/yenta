use getset::Getters;
use rayon::prelude::*;
use std::cmp::{min, Ordering};

use crate::min_max_tie_heap::MinMaxTieHeap;
use crate::types::{Name, NameWeighted};

#[derive(Debug)]
pub struct MatchOptions {
    pub minimum_score: f64,
    pub num_results: usize,
    pub ties_within: Option<f64>,
}

pub fn do_match<'a>(
    from_name: &'a NameWeighted,
    to_names: &'a Vec<NameWeighted>,
    match_opts: &MatchOptions,
) -> () { // Vec<MatchResult<'a>> {
    let best_matches: MinMaxTieHeap<_> = to_names
        .iter()
        .filter_map(|to_name| {
            let swc = score_weighted_name(from_name, to_name);
            if swc.score > match_opts.minimum_score {
                Some(swc)
            } else {
                None
            }
        })
        .fold(
            min_max_tie_heap_identity_element(&match_opts),
            |mut mmth, element| {
                mmth.push(element);
                mmth
            },
        );
    // best_matches.into_vec_desc()
    println!("Best matches for {:?}", from_name.name().unprocessed());

    for element in best_matches.into_vec_desc() {
        println!("    {} {}", element.score, element.to_name.unprocessed());
    }
}

fn min_max_tie_heap_identity_element<'a>(
    match_opts: &MatchOptions,
) -> MinMaxTieHeap<MatchResult<'a>> {
    let are_tied: Box<dyn Fn(&MatchResult, &MatchResult) -> bool> = match match_opts.ties_within {
        None => Box::new(|_: &MatchResult, _: &MatchResult| false),
        Some(eps) => {
            Box::new(move |a: &MatchResult, b: &MatchResult| (a.score - b.score).abs() < eps)
        }
    };
    MinMaxTieHeap::new(match_opts.num_results, are_tied)
}

fn score_weighted_name<'a>(
    from_name: &'a NameWeighted,
    to_name: &'a NameWeighted,
) -> MatchResult<'a> {
    let score_in_common: f64 = from_name
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

    MatchResult {
        from_name: from_name.name(),
        to_name: to_name.name(),
        score: score_in_common / (from_name.total_weight * to_name.total_weight),
    }
}

/// MatchResult is an compatible with MinMaxTieHeap for storing match results.
#[derive(Debug, Getters)]
pub struct MatchResult<'a> {
    from_name: &'a Name,
    #[getset(get = "pub")]
    to_name: &'a Name,
    #[getset(get = "pub")]
    score: f64,
}

impl<'a> PartialEq for MatchResult<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.score.eq(&other.score)
    }
}
impl<'a> Eq for MatchResult<'a> {}
impl<'a> PartialOrd for MatchResult<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}
impl<'a> Ord for MatchResult<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
