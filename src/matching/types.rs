use getset::Getters;
use serde::Serialize;
use std::cmp::Ordering;

use crate::core::{
    Idf, Name, NameDamerauLevenshtein, NameLevenshtein, NameNGrams, NameProcessed, NameWeighted,
};

// use crate::core::idf::Idf;
// use crate::core::name::{Name, NameNGrams, NameProcessed, NameWeighted};

/******************************************************************************/
/* MatchMode Trait                                                            */
/******************************************************************************/
pub trait MatchMode<N> {
    type MatchableData;

    fn make_matchable_name(&self, np: NameProcessed<N>, idf: &Idf) -> Self::MatchableData;
    fn score_match<'a>(
        &self,
        _: &'a Self::MatchableData,
        _: &'a Self::MatchableData,
    ) -> MatchResult<'a, N>;
}

/******************************************************************************/
/* Token match                                                                */
/******************************************************************************/
#[derive(Debug)]
pub struct TokenMatch;

impl<N> MatchMode<N> for TokenMatch {
    type MatchableData = NameWeighted<N>;

    fn make_matchable_name(&self, np: NameProcessed<N>, idf: &Idf) -> Self::MatchableData {
        NameWeighted::new(np, idf)
    }

    fn score_match<'a>(
        &self,
        from_name_weighted: &'a Self::MatchableData,
        to_name_weighted: &'a Self::MatchableData,
    ) -> MatchResult<'a, N> {
        MatchResult {
            from_name: from_name_weighted.name(),
            to_name: to_name_weighted.name(),
            score: from_name_weighted.compute_match_score(to_name_weighted),
        }
    }
}

/******************************************************************************/
/* Ngram match                                                                */
/******************************************************************************/
#[derive(Debug)]
pub struct NGramMatch(usize);

impl NGramMatch {
    pub fn new(n: usize) -> Self {
        NGramMatch(n)
    }
}
impl<N> MatchMode<N> for NGramMatch {
    type MatchableData = NameNGrams<N>;

    fn make_matchable_name(&self, np: NameProcessed<N>, idf: &Idf) -> Self::MatchableData {
        NameNGrams::new(np, idf, self.0)
    }

    fn score_match<'a>(
        &self,
        from_name_ngram: &'a Self::MatchableData,
        to_name_ngram: &'a Self::MatchableData,
    ) -> MatchResult<'a, N> {
        MatchResult {
            from_name: from_name_ngram.name(),
            to_name: to_name_ngram.name(),
            score: from_name_ngram.compute_match_score(to_name_ngram),
        }
    }
}

/******************************************************************************/
/* Levenshtein match                                                          */
/******************************************************************************/
#[derive(Debug)]
pub struct LevenshteinMatch;

impl<N> MatchMode<N> for LevenshteinMatch {
    type MatchableData = NameLevenshtein<N>;

    fn make_matchable_name(&self, np: NameProcessed<N>, idf: &Idf) -> Self::MatchableData {
        NameLevenshtein::new(np, idf)
    }

    fn score_match<'a>(
        &self,
        from_name_ngram: &'a Self::MatchableData,
        to_name_ngram: &'a Self::MatchableData,
    ) -> MatchResult<'a, N> {
        MatchResult {
            from_name: from_name_ngram.name(),
            to_name: to_name_ngram.name(),
            score: from_name_ngram.compute_match_score(to_name_ngram),
        }
    }
}

/******************************************************************************/
/* Damerau-Levenshtein match                                                  */
/******************************************************************************/
#[derive(Debug)]
pub struct DamerauLevenshteinMatch;

impl<N> MatchMode<N> for DamerauLevenshteinMatch {
    type MatchableData = NameDamerauLevenshtein<N>;

    fn make_matchable_name(&self, np: NameProcessed<N>, idf: &Idf) -> Self::MatchableData {
        NameDamerauLevenshtein::new(np, idf)
    }

    fn score_match<'a>(
        &self,
        from_name_ngram: &'a Self::MatchableData,
        to_name_ngram: &'a Self::MatchableData,
    ) -> MatchResult<'a, N> {
        MatchResult {
            from_name: from_name_ngram.name(),
            to_name: to_name_ngram.name(),
            score: from_name_ngram.compute_match_score(to_name_ngram),
        }
    }
} /******************************************************************************/
/* MatchResult data structure                                                 */
/******************************************************************************/

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
