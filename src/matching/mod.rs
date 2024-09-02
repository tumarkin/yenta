mod get_potential_matches;
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
    wrap_error, HasName, Idf, IsName, MatchModeEnum, MatchOptions, MinMaxTieHeap,
    PreprocessingOptions,
};
use crate::preprocess::{prep_name, prep_names};

use crate::matching::get_potential_matches::*;
use types::{DamerauLevenshteinMatch, LevenshteinMatch, NGramMatch, TokenMatch};
use types::{MatchMode, MatchResult, MatchResultSend};

pub fn execute_match<N>(mme: &MatchModeEnum) -> Result<(), Box<dyn Error>>
where
    N: IsName + Send + Sync + GetPotentialMatches<TokenMatch>,
    <N as GetPotentialMatches<TokenMatch>>::PotentialMatchLookup: Sync,
{
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
    let to_names = N::from_csv(&io_args.to_file)?;
    let from_names = N::from_csv(&io_args.from_file)?;

    // Spawn the CSV writer
    spawn_csv_writer(&io_args.output_file, rx)?;

    // Dispatch by match mode
    dispatch_match(mme, from_names, to_names, prep_opts, match_opts, tx)
}

pub fn dispatch_match<N>(
    mme: &MatchModeEnum,
    from_names: Vec<N>,
    to_names: Vec<N>,
    prep_opts: &PreprocessingOptions,
    match_opts: &MatchOptions,
    tx: mpsc::Sender<Vec<MatchResultSend>>,
) -> Result<(), Box<dyn Error>>
where
    N: IsName + Send + Sync + GetPotentialMatches<TokenMatch>,
    <N as GetPotentialMatches<TokenMatch>>::PotentialMatchLookup: Sync,
{
    // Run the match
    match mme {
        MatchModeEnum::TokenMatch { .. } => {
            match_vec_to_generic(TokenMatch, from_names, to_names, prep_opts, match_opts, tx)
        }
        MatchModeEnum::NGramMatch { n_gram_length, .. } => match_vec_to_vec(
            NGramMatch::new(*n_gram_length),
            from_names,
            to_names,
            prep_opts,
            match_opts,
            tx,
        ),
        MatchModeEnum::Levenshtein { .. } => match_vec_to_vec(
            LevenshteinMatch,
            from_names,
            to_names,
            prep_opts,
            match_opts,
            tx,
        ),
        MatchModeEnum::DamerauLevenshtein { .. } => match_vec_to_vec(
            DamerauLevenshteinMatch,
            from_names,
            to_names,
            prep_opts,
            match_opts,
            tx,
        ),
    };

    Ok(())
}

// trait GetPotentialMatches<M>
// where
//     M: MatchMode<Self>,
//     Self: Sized,
// {
//     type PotentialMatchLookup;

//     fn to_names_weighted(
//         match_mode: &M,
//         ns: Vec<NameProcessed<Self>>,
//         idf: &Idf,
//     ) -> Self::PotentialMatchLookup;

//     fn get_potential_names<'a>(
//         n: &'a M::MatchableData,
//         pml: &'a Self::PotentialMatchLookup,
//     ) -> &'a Vec<M::MatchableData>;
// }

// impl<M> GetPotentialMatches<M> for NameUngrouped
// where
//     M: MatchMode<NameUngrouped> + Sync + Sized,
//     M::MatchableData: Send + Sync,
// {
//     type PotentialMatchLookup = Vec<M::MatchableData>;

//     fn to_names_weighted(
//         match_mode: &M,
//         ns: Vec<NameProcessed<Self>>,
//         idf: &Idf,
//     ) -> Self::PotentialMatchLookup {
//         ns.into_par_iter()
//             .map(|name_processed| match_mode.make_matchable_name(name_processed, &idf))
//             .collect()
//     }

//     fn get_potential_names<'a>(
//         _: &'a M::MatchableData,
//         pml: &'a Self::PotentialMatchLookup,
//     ) -> &'a Vec<M::MatchableData> {
//         pml
//     }
// }

// impl<M> GetPotentialMatches<M> for NameGrouped
// where
//     M: MatchMode<NameGrouped> + Sync + Sized,
//     M::MatchableData: Send + Sync,
// {
//     type PotentialMatchLookup = (); // Vec<M::MatchableData>;

//     fn to_names_weighted(
//         _match_mode: &M,
//         _ns: Vec<NameProcessed<Self>>,
//         _idf: &Idf,
//     ) -> Self::PotentialMatchLookup {
//         todo!()
//         // ns.into_par_iter()
//         //     .map(|name_processed| match_mode.make_matchable_name(name_processed, &idf))
//         //     .collect()
//     }

//     fn get_potential_names<'a>(
//         _n: &'a M::MatchableData,
//         _pml: &'a Self::PotentialMatchLookup,
//     ) -> &'a Vec<M::MatchableData> {
//         todo!()
//         // pml
//     }
// }

fn match_vec_to_generic<M, N>(
    match_mode: M,
    from_names: Vec<N>,
    to_names: Vec<N>,
    prep_opts: &PreprocessingOptions,
    match_opts: &MatchOptions,
    send_channel: mpsc::Sender<Vec<MatchResultSend>>,
) where
    M: MatchMode<N> + Sync,
    M::MatchableData: Send + Sync,
    N: Sized + Send + IsName,
    N: GetPotentialMatches<M>,
    <N as GetPotentialMatches<M>>::PotentialMatchLookup: Sync,
{
    // Create the Idf.
    let to_names_processed = prep_names(to_names, prep_opts);
    let idf: Idf = Idf::new(&to_names_processed);

    // Get the match iterator
    let to_names_weighted = N::to_names_weighted(&match_mode, to_names_processed, &idf);
    // .into_par_iter()
    // .map(|name_processed| match_mode.make_matchable_name(name_processed, &idf))
    // .collect();

    let _: Vec<_> = from_names
        .into_par_iter()
        .progress()
        .map_with(send_channel, |s, from_name| {
            let from_name_processed = prep_name(from_name, prep_opts);
            let from_name_weighted = match_mode.make_matchable_name(from_name_processed, &idf);
            if let Some(to_potential_names) =
                N::get_potential_names(from_name_weighted.get_name(), &to_names_weighted)
            {
                let best_matches: Vec<_> = best_matches_for_single_name(
                    &match_mode,
                    &from_name_weighted,
                    &to_potential_names,
                    match_opts,
                );

                let match_results_to_send: Vec<_> = best_matches
                    .iter()
                    .map(|bm| MatchResultSend {
                        from_name: bm.from_name().unprocessed_name().to_string(),
                        from_id: bm.from_name().idx().to_string(),
                        to_name: bm.to_name().unprocessed_name().to_string(),
                        to_id: bm.to_name().idx().to_string(),
                        score: *bm.score(),
                    })
                    .collect();

                s.send(match_results_to_send).unwrap();
            }
        })
        .collect();
}

pub fn match_vec_to_vec<T, N>(
    match_mode: T,
    from_names: Vec<N>,
    to_names: Vec<N>,
    prep_opts: &PreprocessingOptions,
    match_opts: &MatchOptions,
    send_channel: mpsc::Sender<Vec<MatchResultSend>>,
) where
    T: MatchMode<N> + Sync,
    T::MatchableData: Send + Sync,
    N: Sized + Send + IsName,
{
    // Create the Idf.
    let to_names_processed = prep_names(to_names, prep_opts);
    let idf: Idf = Idf::new(&to_names_processed);

    // Get the match iterator
    let to_names_weighted: Vec<_> = to_names_processed
        .into_par_iter()
        .map(|name_processed| match_mode.make_matchable_name(name_processed, &idf))
        .collect();

    let _: Vec<_> = from_names
        .into_par_iter()
        .progress()
        .map_with(send_channel, |s, from_name| {
            let from_name_processed = prep_name(from_name, prep_opts);
            let from_name_weighted = match_mode.make_matchable_name(from_name_processed, &idf);

            let best_matches: Vec<_> = best_matches_for_single_name(
                &match_mode,
                &from_name_weighted,
                &to_names_weighted,
                match_opts,
            );

            let match_results_to_send: Vec<_> = best_matches
                .iter()
                .map(|bm| MatchResultSend {
                    from_name: bm.from_name().unprocessed_name().to_string(),
                    from_id: bm.from_name().idx().to_string(),
                    to_name: bm.to_name().unprocessed_name().to_string(),
                    to_id: bm.to_name().idx().to_string(),
                    score: *bm.score(),
                })
                .collect();

            s.send(match_results_to_send).unwrap();
        })
        .collect();
}

fn best_matches_for_single_name<'a, T, N>(
    match_mode: &'a T,
    from_name: &'a T::MatchableData,
    to_names: &'a [T::MatchableData],
    match_opts: &MatchOptions,
) -> Vec<MatchResult<'a, N>>
where
    T: MatchMode<N>,
{
    let best_matches: MinMaxTieHeap<_> = to_names
        .iter()
        .filter_map(|to_name| {
            let match_result = match_mode.score_match(from_name, to_name);
            if match_result.score > match_opts.minimum_score {
                Some(match_result)
            } else {
                None
            }
        })
        .fold(
            min_max_tie_heap_identity_element(match_opts),
            |mut mmth, element| {
                mmth.push(element);
                mmth
            },
        );
    best_matches.into_vec_desc()
}

type AreTiedFn<T> = dyn Fn(&T, &T) -> bool;

fn min_max_tie_heap_identity_element<'a, N>(
    match_opts: &MatchOptions,
) -> MinMaxTieHeap<MatchResult<'a, N>> {
    let are_tied: Box<AreTiedFn<MatchResult<N>>> = match match_opts.ties_within {
        None => Box::new(|_: &MatchResult<N>, _: &MatchResult<N>| false),
        Some(eps) => {
            Box::new(move |a: &MatchResult<N>, b: &MatchResult<N>| (a.score - b.score).abs() < eps)
        }
    };
    MinMaxTieHeap::new(match_opts.num_results, are_tied)
}

fn spawn_csv_writer(
    path: &str,
    rx: mpsc::Receiver<Vec<MatchResultSend>>,
) -> Result<(), Box<dyn Error>> {
    let output_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| wrap_error(e, format!("when accessing output file {}", path)))?;

    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    thread::spawn(move || {
        while let Ok(match_results) = rx.recv() {
            let _: Vec<_> = match_results
                .iter()
                .map(|mrs| wtr.serialize(mrs).unwrap())
                .collect();
        }
    });

    Ok(())
}
