// extern crate clap;
// extern crate counter;
// extern crate deunicode;
// extern crate getset;
// extern crate indicatif;
// extern crate ngrams;
// extern crate rayon;
// extern crate soundex;

mod core;
mod io;
mod matching;
mod preprocess;

use clap::{crate_version, App, Arg};
use std::error::Error;

use crate::matching::{execute_match, MatchOptions};
use crate::preprocess::PreprocessingOptions;

fn main() {
    let res = get_command_line_arguments().and_then(
        |(from_names_path, to_names_path, _output_path, prep_opts, match_opts)| {
            execute_match(
                &from_names_path,
                &to_names_path,
                &_output_path,
                &prep_opts,
                &match_opts,
            )
        },
    );

    if let Err(e) = res {
        println!("{}", e)
    }
}

fn get_command_line_arguments(
) -> Result<(String, String, String, PreprocessingOptions, MatchOptions), Box<dyn Error>> {
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
        trim_length: cli_opts.value_of("token-length").map(|s| {
            s.trim()
                .parse()
                .expect("Unable to parse token-length integer")
        }),
    };

    let match_opts = MatchOptions {
        minimum_score: cli_opts
            .value_of("minimum-match-score")
            .unwrap()
            .trim()
            .parse()
            .expect("Unable to parse minimum-match-score floating point number"),
        num_results: cli_opts
            .value_of("number-of-results")
            .unwrap()
            .trim()
            .parse()
            .expect("Unable to parse number-of-results integer"),
        ties_within: cli_opts
            .value_of("include-ties-within")
            .and_then(|s| s.trim().parse().ok()),
    };

    Ok((
        cli_opts.value_of("FROM-FILE").unwrap().to_string(),
        cli_opts.value_of("TO-FILE").unwrap().to_string(),
        cli_opts.value_of("output-file").unwrap().to_string(),
        prep_opts,
        match_opts,
    ))
}
