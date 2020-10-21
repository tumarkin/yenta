use getset::Getters;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::error::Error;
use std::marker::Send;

use crate::core::idf::IDF;
use crate::core::min_max_tie_heap::MinMaxTieHeap;
use crate::core::name::{Name, NameNGrams, NameProcessed, NameWeighted};
use crate::preprocess::{prep_name, prep_names, PreprocessingOptions};

/******************************************************************************/
/* Configuration options                                                      */
/******************************************************************************/
#[derive(Debug)]
pub struct MatchOptions {
    // pub match_mode: MatchMode,
    pub minimum_score: f64,
    pub num_results: usize,
    pub ties_within: Option<f64>,
}

/******************************************************************************/
/* Match modes                                                                */
/******************************************************************************/
#[derive(Debug)]
pub struct MatchModeExactMatch;

#[derive(Debug)]
pub struct MatchModeNGram(usize);

pub trait MatchMode {
    type MatchableData;

    fn make_matchable_name(&self, np: NameProcessed, idf: &IDF) -> Self::MatchableData;
    fn score_match<'a>(
        &self,
        _: &'a Self::MatchableData,
        _: &'a Self::MatchableData,
    ) -> MatchResult<'a>;
}

impl MatchMode for MatchModeExactMatch {
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

impl MatchMode for MatchModeNGram {
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

/*****************************************************************************/
/* Matching                                                                  */
/*****************************************************************************/
pub fn execute_match(
    from_names_path: &String,
    to_names_path: &String,
    _output_path: &String,
    prep_opts: &PreprocessingOptions,
    match_opts: &MatchOptions,
) -> Result<(), Box<dyn Error>> {
    // Load in both name files. This is done immediately and eagerly to ensure that
    // all names in both files are properly formed.
    let to_names =
        Name::from_csv(&from_names_path).map_err(|e| format!("Unable to parse TO-CSV: {}", e))?;
    let from_names =
        Name::from_csv(&to_names_path).map_err(|e| format!("Unable to parse FROM-CSV: {}", e))?;

    // Create the IDF.
    let to_names_processed = prep_names(to_names, &prep_opts);
    let idf: IDF = IDF::new(&to_names_processed);

    // Run the match
    match_vec_to_vec(
        MatchModeExactMatch,
        from_names,
        to_names_processed,
        &idf,
        &prep_opts,
        &match_opts,
    );

    Ok(())
}

pub fn match_vec_to_vec<T>(
    match_mode: T,
    from_names: Vec<Name>,
    to_names_processed: Vec<NameProcessed>,
    idf: &IDF,
    prep_opts: &PreprocessingOptions,
    match_opts: &MatchOptions,
) -> ()
where
    T: MatchMode + Sync,
    T::MatchableData: Send + Sync,
{
    // Get the match iterator
    let to_names_weighted: Vec<_> = to_names_processed
        .into_par_iter()
        .map(|name_processed| match_mode.make_matchable_name(name_processed, &idf))
        .collect();

    let _: Vec<_> = from_names
        .into_par_iter()
        .progress()
        .map(|from_name| {
            let from_name_processed = prep_name(from_name, &prep_opts);
            let from_name_weighted = match_mode.make_matchable_name(from_name_processed, &idf);

            let best_matches: Vec<_> = best_matches_for_single_name(
                &match_mode,
                &from_name_weighted,
                &to_names_weighted,
                &match_opts,
            );

            for element in best_matches {
                let output_str = format!(
                    "{},{},{},{},{}",
                    element.from_name().unprocessed(),
                    element.from_name().idx(),
                    element.to_name().unprocessed(),
                    element.to_name().idx(),
                    element.score(),
                );

                println!("{}", output_str);
            }
        })
        .collect();
    ()
}

fn best_matches_for_single_name<'a, T>(
    match_mode: &'a T,
    from_name: &'a T::MatchableData,
    to_names: &'a Vec<T::MatchableData>,
    match_opts: &MatchOptions,
) -> Vec<MatchResult<'a>>
where
    T: MatchMode,
{
    let best_matches: MinMaxTieHeap<_> = to_names
        .iter()
        .filter_map(|to_name| {
            let match_result = match_mode.score_match(&from_name, &to_name);
            if match_result.score > match_opts.minimum_score {
                Some(match_result)
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
    best_matches.into_vec_desc()
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

/******************************************************************************/
/* MatchResult data structure                                                 */
/******************************************************************************/

/// MatchResult is an compatible with MinMaxTieHeap for storing match results.
#[derive(Debug, Getters)]
pub struct MatchResult<'a> {
    #[getset(get = "pub")]
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
