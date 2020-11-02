use getset::Getters;
use serde::Serialize;
use std::cmp::Ordering;

use crate::core::{Name, NameNGrams, NameProcessed, NameWeighted, IDF};

// use crate::core::idf::IDF;
// use crate::core::name::{Name, NameNGrams, NameProcessed, NameWeighted};

/******************************************************************************/
/* MatchMode Trait                                                            */
/******************************************************************************/
pub trait MatchMode {
    type MatchableData;

    fn make_matchable_name(&self, np: NameProcessed, idf: &IDF) -> Self::MatchableData;
    fn score_match<'a>(
        &self,
        _: &'a Self::MatchableData,
        _: &'a Self::MatchableData,
    ) -> MatchResult<'a>;
}

/******************************************************************************/
/* Match modes                                                                */
/******************************************************************************/

#[derive(Debug)]
pub struct ExactMatch;

#[derive(Debug)]
pub struct NGramMatch(usize);

impl NGramMatch {
    pub fn new(n: usize) -> Self {
        NGramMatch(n)
    }
}

impl MatchMode for ExactMatch {
    type MatchableData = NameWeighted;

    fn make_matchable_name(&self, np: NameProcessed, idf: &IDF) -> Self::MatchableData {
        NameWeighted::new(np, &idf)
    }

    fn score_match<'a>(
        &self,
        from_name_weighted: &'a Self::MatchableData,
        to_name_weighted: &'a Self::MatchableData,
    ) -> MatchResult<'a> {
        MatchResult {
            from_name: &from_name_weighted.name(),
            to_name: &to_name_weighted.name(),
            score: from_name_weighted.compute_match_score(&to_name_weighted),
        }
    }
}

impl MatchMode for NGramMatch {
    type MatchableData = NameNGrams;

    fn make_matchable_name(&self, np: NameProcessed, idf: &IDF) -> Self::MatchableData {
        NameNGrams::new(np, &idf, self.0)
    }

    fn score_match<'a>(
        &self,
        from_name_ngram: &'a Self::MatchableData,
        to_name_ngram: &'a Self::MatchableData,
    ) -> MatchResult<'a> {
        MatchResult {
            from_name: &from_name_ngram.name(),
            to_name: &to_name_ngram.name(),
            score: from_name_ngram.compute_match_score(&to_name_ngram),
        }
    }
}

/******************************************************************************/
/* MatchResult data structure                                                 */
/******************************************************************************/

/// MatchResult is an compatible with MinMaxTieHeap for storing match results.
#[derive(Debug, Getters)]
pub struct MatchResult<'a> {
    #[getset(get = "pub")]
    pub from_name: &'a Name,
    #[getset(get = "pub")]
    pub to_name: &'a Name,
    #[getset(get = "pub")]
    pub score: f64,
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

#[derive(Debug, Serialize)]
pub struct MatchResultSend {
    pub from_name: String,
    pub from_id: String,
    pub to_name: String,
    pub to_id: String,
    pub score: f64,
}
