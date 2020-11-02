mod types;

use csv::WriterBuilder;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use std::error::Error;
use std::fs::OpenOptions;
use std::marker::Send;
use std::sync::mpsc;
use std::thread;

use crate::core::{
    wrap_error, MatchModeEnum, MatchOptions, MinMaxTieHeap, Name, NameProcessed,
    PreprocessingOptions, IDF,
};
use crate::preprocess::{prep_name, prep_names};

use types::{ExactMatch, MatchMode, MatchResult, MatchResultSend, NGramMatch};

/*****************************************************************************/
/* Matching                                                                  */
/*****************************************************************************/
pub fn execute_match(mme: &MatchModeEnum) -> Result<(), Box<dyn Error>> {
    let (tx, rx): (
        mpsc::Sender<Vec<MatchResultSend>>,
        mpsc::Receiver<Vec<MatchResultSend>>,
    ) = mpsc::channel();

    // Bring in shared command line options
    let io_args = &mme.get_cli().io_args;
    let prep_opts = &mme.get_cli().preprocessing_options;
    let match_opts = &mme.get_cli().match_options;

    // Load in both name files. This is done immediately and eagerly to ensure that
    // all names in both files are properly formed.
    let to_names = Name::from_csv(&io_args.to_file)?;
    let from_names = Name::from_csv(&io_args.from_file)?;

    // Create the IDF.
    let to_names_processed = prep_names(to_names, &prep_opts);
    let idf: IDF = IDF::new(&to_names_processed);

    // Open the file for writing, failing if it already exists
    let output_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&io_args.output_file)
        .map_err(|e| {
            wrap_error(
                e,
                format!("when accessing output file {}", io_args.output_file),
            )
        })?;

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
    match mme {
        MatchModeEnum::ExactMatch { em_cli: _ } => match_vec_to_vec(
            ExactMatch,
            from_names,
            to_names_processed,
            &idf,
            &prep_opts,
            &match_opts,
            tx,
        ),
        MatchModeEnum::NGramMatch {
            ng_cli: _,
            n_gram_length,
        } => match_vec_to_vec(
            NGramMatch::new(*n_gram_length),
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
