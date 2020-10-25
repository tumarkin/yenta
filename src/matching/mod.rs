use csv::WriterBuilder;
use getset::Getters;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::Serialize;
use std::cmp::Ordering;
use std::error::Error;
use std::fs::OpenOptions;
use std::marker::Send;
use std::sync::mpsc;
use std::thread;

use crate::core::args::IoArgs;
use crate::core::idf::IDF;
use crate::core::min_max_tie_heap::MinMaxTieHeap;
use crate::core::name::{Name, NameNGrams, NameProcessed, NameWeighted};
use crate::preprocess::{prep_name, prep_names, PreprocessingOptions};

// mod types;
// pub use types::{MatchOptions, MatchMode, MatchResult, MatchResultSend};

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

#[derive(Debug, Serialize)]
pub struct MatchResultSend {
    from_name: String,
    from_id: String,
    to_name: String,
    to_id: String,
    score: f64,
}

/******************************************************************************/
/* Match modes                                                                */
/******************************************************************************/
pub enum MatchModeEnum {
    ExactMatch,
    NGramMatch(usize),
}

#[derive(Debug)]
pub struct ExactMatch;

#[derive(Debug)]
pub struct NGramMatch(usize);

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

/*****************************************************************************/
/* Matching                                                                  */
/*****************************************************************************/
pub fn execute_match(
    io_args: &IoArgs,
    prep_opts: &PreprocessingOptions,
    match_mode: &MatchModeEnum,
    match_opts: &MatchOptions,
) -> Result<(), Box<dyn Error>> {
    let (tx, rx): (
        mpsc::Sender<Vec<MatchResultSend>>,
        mpsc::Receiver<Vec<MatchResultSend>>,
    ) = mpsc::channel();

    // Load in both name files. This is done immediately and eagerly to ensure that
    // all names in both files are properly formed.
    let to_names =
        Name::from_csv(&io_args.to_file).map_err(|e| format!("Unable to parse TO-CSV: {}", e))?;
    let from_names = Name::from_csv(&io_args.from_file)
        .map_err(|e| format!("Unable to parse FROM-CSV: {}", e))?;

    // Create the IDF.
    let to_names_processed = prep_names(to_names, &prep_opts);
    let idf: IDF = IDF::new(&to_names_processed);

    // Open the file for writing, failing if it already exists
    let output_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&io_args.output_file)?;

    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    thread::spawn(move || loop {
        match rx.recv() {
            Ok(match_results) => {
                let _: Vec<_> = match_results
                    .iter()
                    .map(|mrs| wtr.serialize(mrs).unwrap())
                    .collect();
            }
            Err(_) => break,
        }
    });

    // Run the match
    match match_mode {
        MatchModeEnum::ExactMatch => match_vec_to_vec(
            ExactMatch,
            from_names,
            to_names_processed,
            &idf,
            &prep_opts,
            &match_opts,
            tx,
        ),
        MatchModeEnum::NGramMatch(n) => match_vec_to_vec(
            NGramMatch(*n),
            from_names,
            to_names_processed,
            &idf,
            &prep_opts,
            &match_opts,
            tx,
        ),
    };

    Ok(())
}

pub fn match_vec_to_vec<T>(
    match_mode: T,
    from_names: Vec<Name>,
    to_names_processed: Vec<NameProcessed>,
    idf: &IDF,
    prep_opts: &PreprocessingOptions,
    match_opts: &MatchOptions,
    send_channel: mpsc::Sender<Vec<MatchResultSend>>,
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
        .map_with(send_channel, |s, from_name| {
            let from_name_processed = prep_name(from_name, &prep_opts);
            let from_name_weighted = match_mode.make_matchable_name(from_name_processed, &idf);

            let best_matches: Vec<_> = best_matches_for_single_name(
                &match_mode,
                &from_name_weighted,
                &to_names_weighted,
                &match_opts,
            );

            let match_results_to_send: Vec<_> = best_matches
                .iter()
                .map(|bm| MatchResultSend {
                    from_name: bm.from_name().unprocessed().clone(),
                    from_id: bm.from_name().idx().clone(),
                    to_name: bm.to_name().unprocessed().clone(),
                    to_id: bm.to_name().idx().clone(),
                    score: *bm.score(),
                })
                .collect();

            s.send(match_results_to_send).unwrap();
        })
        .collect();
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


