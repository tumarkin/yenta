use getset::Getters;
use serde::Serialize;
use std::cmp::Ordering;

/// MatchResult is an compatible with MinMaxTieHeap for storing match results.
#[derive(Debug, Getters)]
pub struct MatchResult<'a, N> {
    #[getset(get = "pub")]
    pub from_name: &'a N,
    #[getset(get = "pub")]
    pub to_name: &'a N,
    #[getset(get = "pub")]
    pub score: f64,
}

impl<'a, N> PartialEq for MatchResult<'a, N> {
    fn eq(&self, other: &Self) -> bool {
        self.score.eq(&other.score)
    }
}
impl<'a, N> Eq for MatchResult<'a, N> {}
impl<'a, N> PartialOrd for MatchResult<'a, N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}
impl<'a, N> Ord for MatchResult<'a, N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Serialize)]
pub struct MatchResultSend {
    pub from_name: String,
    pub from_id: String,
    pub to_name: String,
    pub to_id: String,
    pub score: f64,
}
