// extern crate clap;
// extern crate counter;
// extern crate deunicode;
// extern crate getset;
// extern crate indicatif;
// extern crate ngrams;
// extern crate rayon;
// extern crate soundex;

mod core;
mod matching;
mod preprocess;

use clap;
use clap::{crate_version, value_t, App, Arg, ArgGroup};
use std::error::Error;

use crate::core::args::IoArgs;
use crate::matching::{execute_match, MatchModeEnum, MatchOptions};
use crate::preprocess::PreprocessingOptions;

fn main() {
    let res =
        get_command_line_arguments().and_then(|(io_args, prep_opts, match_mode, match_opts)| {
            execute_match(&io_args, &prep_opts, &match_mode, &match_opts)
        });

    if let Err(e) = res {
        println!("{}", e)
    }
}

fn get_command_line_arguments(
) -> Result<(IoArgs, PreprocessingOptions, MatchModeEnum, MatchOptions), Box<dyn Error>> {
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
            Arg::with_name("token-match")
                .long("token-match")
                .help("Match on processed tokens"),
        )
        .arg(
            Arg::with_name("ngram-match")
                .long("ngram-match")
                .value_name("INT")
                .takes_value(true)
                .help("Match on processed n-grams"),
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
        .group(
            ArgGroup::with_name("match-mode")
                .arg("token-match")
                .arg("ngram-match"),
        )
        .after_help("More details and Wiki at github.com/tumarkin/yenta")
        .get_matches();

    // IO argument parsing
    let from_file = cli_opts.value_of("FROM-FILE").unwrap().to_string();
    let to_file = cli_opts.value_of("TO-FILE").unwrap().to_string();
    let output_file = cli_opts.value_of("output-file").unwrap().to_string();

    let io_args = IoArgs {
        from_file,
        to_file,
        output_file,
    };

    // Preprocessing options parsing
    let trim_length_r: Result<Option<usize>, clap::Error> =
        cli_opts.value_of("token-length").map_or(Ok(None), |_| {
            let tl = value_t!(cli_opts.value_of("token-length"), usize)
                .map_err(|e| format_clap_error(e, "token-length"))?;
            Ok(Some(tl))
        });
    let trim_length = trim_length_r?;

    let prep_opts = PreprocessingOptions {
        adjust_unicode: !cli_opts.is_present("retain-unicode"),
        adjust_case: true,
        alphabetic_only: !cli_opts.is_present("retain-non-alphabetic"),
        soundex: cli_opts.is_present("soundex"),
        trim_length
        // : cli_opts.value_of("token-length").map(|s| {
        //     s.trim()
        //         .parse()
        //         .expect("Unable to parse token-length integer")
        // }),
    };

    // Matchmode option parsing
    let mut match_mode = MatchModeEnum::ExactMatch;
    if let Some(_) = cli_opts.value_of("ngram-match") {
        let ngram_size = value_t!(cli_opts.value_of("ngram-match"), usize)
            .map_err(|e| format_clap_error(e, "ngram-match"))?;
        match_mode = MatchModeEnum::NGramMatch(ngram_size);
    }

    // Matching option parsing
    let minimum_score = value_t!(cli_opts.value_of("minimum-match-score"), f64)
        .map_err(|e| format_clap_error(e, "minimum-score"))?;
    let num_results = value_t!(cli_opts.value_of("number-of-results"), usize)
        .map_err(|e| format_clap_error(e, "number-of-resuts"))?;

    let match_opts = MatchOptions {
        minimum_score,
        num_results,
        ties_within: cli_opts
            .value_of("include-ties-within")
            .and_then(|s| s.trim().parse().ok()),
    };

    Ok((io_args, prep_opts, match_mode, match_opts))
}

/*****************************************************************************/
// Utility functions
/*****************************************************************************/
fn format_clap_error(e: clap::Error, field: &str) -> clap::Error {
    clap::Error {
        message: format!("{} for \"{}\"", e.message, field),
        ..e
    }
}
