use crate::types::NameWeighted;
use crate::min_max_tie_heap::MinMaxTieHeap;

pub struct MatchOptions {
    _minimum_score: f64,
    _num_results: usize,
    _ties_within: Option<f64>,
}

impl MatchOptions {
    pub fn new(_minimum_score: f64, _num_results: usize, _ties_within: Option<f64>) -> Self {
        MatchOptions {
            _minimum_score,
            _num_results,
            _ties_within,
        }
    }
}

pub fn do_match(from_name: &NameWeighted, to_names: &Vec<NameWeighted>) -> () {
    unimplemented!();
}
