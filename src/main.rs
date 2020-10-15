extern crate clap;
extern crate counter;
extern crate deunicode;
extern crate indicatif;
extern crate rayon;
extern crate soundex;

mod io;
mod matching;
mod min_max_tie_heap;
mod preprocess;
mod types;

// use counter::Counter;
// use rayon::iter::{ParallelIterator, IntoParallelRefIterator};
// use std::error::Error;
use clap::{crate_version, App, Arg};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use io::read_name_csv;
use types::{Name, NameProcessed, NameWeighted, IDF};

use matching::MatchOptions;
use preprocess::{prep_words, PreprocessingOptions};

fn main() {
    let (from_names_path, to_names_path, _output_path, prep_opts, _match_opts) = execute_cli();

    let to_names = read_name_csv(&from_names_path).expect("Unable to parse TO-CSV");
    let (to_names_weighted, idf) = process_names_and_form_idf(to_names, &prep_opts);

    println!("{:?}", idf);

    let from_names = read_name_csv(&to_names_path).expect("Unable to parse FROM-CSV");
    for from_name in from_names.into_iter().take(5) {
        // Turn this into a function
        // unimplemented!();
        let tc = prep_words(&from_name.unprocessed, &prep_opts)
            .into_iter()
            .collect();

        let from_name_processed = NameProcessed {
            name: from_name,
            token_counter: tc,
        };
        println!("{:?}", from_name_processed);
    }

    for to_name in to_names_weighted.iter().take(5) {
        println!("{:?}", to_name);
    }
}

fn execute_cli() -> (String, String, String, PreprocessingOptions, MatchOptions) {
    let cli_opts = App::new("Yenta")
        .version(crate_version!())
        // .author("Robert Tumarkin <r.tumarkin@unsw.edu.au>")
        .about("A matchmaker for text files")
        .arg(
            Arg::with_name("FROM-FILE")
                .help("Match names from this file...")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("TO-FILE")
                .help("...to names in this file")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("retain-non-alphabetic")
                .long("retain-non-alphabetic")
                .help("Retain non-alphabetic characters"),
        )
        .arg(
            Arg::with_name("retain-unicode")
                .long("retain-unicode")
                .help("Do not convert unicode characters to ASCII equivalents"),
        )
        .arg(
            Arg::with_name("soundex")
                .long("soundex")
                .help("Process words with SoundEx algorithm"),
        )
        .arg(
            Arg::with_name("token-length")
                .short("t")
                .long("token-length")
                .value_name("INT")
                .takes_value(true)
                .help("Trim each word to have a maximum number of characters"),
        )
        .arg(
            Arg::with_name("number-of-results")
                .short("n")
                .long("number-of-results")
                .value_name("FLOAT")
                .takes_value(true)
                .help("The number of results to output")
                .default_value("1"),
        )
        .arg(
            Arg::with_name("include-ties-within")
                .long("include-ties-within")
                .value_name("FLOAT")
                .takes_value(true)
                .help("Include ties within FLOAT of the nth requested result"),
        )
        .arg(
            Arg::with_name("minimum-match-score")
                .short("m")
                .long("minimum-match-score")
                .value_name("FLOAT")
                .takes_value(true)
                .help("The minimum score required to be considered a match")
                .default_value("0.01"),
        )
        .arg(
            Arg::with_name("output-file")
                .short("o")
                .long("output-file")
                .value_name("FILEPATH")
                .takes_value(true)
                .help("Save results to FILEPATH")
                .required(true),
        )
        .after_help("More details and Wiki at github.com/tumarkin/yenta")
        .get_matches();

    let prep_opts = PreprocessingOptions {
        adjust_unicode: !cli_opts.is_present("retain-unicode"),
        adjust_case: true,
        alphabetic_only: !cli_opts.is_present("retain-non-alphabetic"),
        soundex: cli_opts.is_present("soundex"),
        trim_length: cli_opts
            .value_of("token-length")
            .map(|s| s.trim().parse().unwrap()),
    };

    let match_opts = MatchOptions::new(0.01, 1, None);

    (
        cli_opts.value_of("FROM-FILE").unwrap().to_string(),
        cli_opts.value_of("TO-FILE").unwrap().to_string(),
        cli_opts.value_of("output-file").unwrap().to_string(),
        prep_opts,
        match_opts,
    )
}

/******************************************************************************/
/* Algorithm components                                                       */
/******************************************************************************/
fn process_names_and_form_idf(
    names: Vec<Name>,
    prep_opts: &PreprocessingOptions,
) -> (Vec<NameWeighted>, IDF) {
    let names_processed: Vec<NameProcessed> = names
        .into_par_iter()
        .progress()
        .map(|name| {
            let tc = prep_words(&name.unprocessed, &prep_opts)
                .into_iter()
                .collect();

            NameProcessed {
                name,
                token_counter: tc,
            }
        })
        .collect();
    // println!("{:?}", names_processed);

    // let df: DocumentFrequency = names.iter().collect();
    let idf: IDF = IDF::new(&names_processed);

    let names_weighted: Vec<NameWeighted> = names_processed
        .into_par_iter()
        .map(|name_processed| NameWeighted::new(name_processed, &idf))
        .collect();
    (names_weighted, idf)
}
