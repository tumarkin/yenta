extern crate clap;
extern crate counter;
extern crate deunicode;
extern crate soundex;

mod preprocess;
mod types;
use clap::{App, Arg};
// use counter::Counter;
use types::{Name, NameProcessed, IDF};

use preprocess::{prep_words, PreprocessingOptions};

fn main() {
    let matches = execute_cli();
    // println!("{:?}", matches);

    let prep_opts = PreprocessingOptions {
        adjust_unicode: !matches.is_present("retain-unicode"),
        adjust_case: true,
        alphabetic_only: !matches.is_present("retain-non-alphabetic"),
        soundex: matches.is_present("soundex"),
        trim_length: matches
            .value_of("token-length")
            .map(|s| s.trim().parse().unwrap()),
    };
    // println!("{:?}", prep_opts);

    let names: Vec<NameProcessed> = names()
        .into_iter()
        .enumerate()
        .map(|(pos, text)| {
            let tc = prep_words(&text, &prep_opts).into_iter().collect();

            let n = Name {
                unprocessed: text,
                idx: format!("{}", pos),
            };

            NameProcessed
                {
                    name: n,
                    token_counter: tc
                }

            
        })
        .collect();

    // let df: DocumentFrequency = names.iter().collect();
    let idf: IDF = IDF::new(&names);

    println!("{:?}", names);
    // println!("{:?}", df);
    println!("{:?}", idf);
}





fn execute_cli() -> clap::ArgMatches<'static> {
    App::new("Yenta")
        .version("1.0")
        // .author("Robert Tumarkin <r.tumarkin@unsw.edu.au>")
        .about("a matchmaker for text files")
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
        // .arg(
        //     Arg::with_name("config")
        //         .short("c")
        //         .long("config")
        //         .value_name("FILE")
        //         .help("Sets a custom config file")
        //         .takes_value(true),
        // )
        // .arg(
        //     Arg::with_name("INPUT")
        //         .help("Sets the input file to use")
        //         .required(true)
        //         .index(1),
        // )
        // .arg(
        //     Arg::with_name("v")
        //         .short("v")
        //         .multiple(true)
        //         .help("Sets the level of verbosity"),
        // )
        // .subcommand(
        //     SubCommand::with_name("test")
        //         .about("controls testing features")
        //         .version("1.3")
        //         .author("Someone E. <someone_else@other.com>")
        //         .arg(
        //             Arg::with_name("debug")
        //                 .short("d")
        //                 .help("print debug information verbosely"),
        //         ),
        // )
        .get_matches()
}

// Yente - a matchmaker for text files (Version 0.4.0.1)

// Usage: yente FROM-FILE TO-FILE ([--phonix] | [--soundex]) [--retain-numerical] [--retain-unicode] [-t|--token-length INT] (
// [--levenshtein-penalty DOUBLE] |
//              [--ngram-size INT]) [-g|--subgroup-search] [-n|--number-of-results INT] [-T|--include-ties] [-m|--minimum-match-score FLOAT] (-o|--output-file FILEPATH)

// Available options:
// --phonix                 Preprocess words with Phonix algorithm.
// --levenshtein-penalty DOUBLE
// The Levenshtein edit distance penalty factor (percent correct letters raised to factor is multiplied by token score)
// --ngram-size INT         The size of ngrams to use (2 is recommended to start)
// -g,--subgroup-search     Search for matches in groups (requires 'group' column in data files)
// -n,--number-of-results INT
// The number of results to output (default: 1)
// -T,--include-ties        Include ties in output
// -m,--minimum-match-score FLOAT
// The minimum score required to be considered a match (default: 1.0e-2)
// -o,--output-file FILEPATH
// Copy results to an output file
// -h,--help                Show this help text

// More details and Wiki at github.com/tumarkin/yente

fn names() -> Vec<String> {
    vec![
        "Cassaundra Ehrlich".to_string(),
        "Melida Clegg".to_string(),
        "Ashlyn Viveros".to_string(),
        "Barrett Slayton".to_string(),
        "Kit Mccallister".to_string(),
        "Karlene Mcafee".to_string(),
        "Marcene Shelor".to_string(),
        "Lesli Brownson".to_string(),
        "Marlen Shultz".to_string(),
        "Carolann Bingman".to_string(),
        "Glen Schrack".to_string(),
        "Jenae Arbaugh".to_string(),
        "Babette Heise".to_string(),
        "Rita Fawcett".to_string(),
        "Inge Caldera".to_string(),
        "Chadwick Palencia".to_string(),
        "Micheline Redmon".to_string(),
        "Blondell Abdallah".to_string(),
        "Nelida Winnie".to_string(),
        "Jack Hoop".to_string(),
        "Adina Nunnery".to_string(),
        "Melisa Slovinsky".to_string(),
        "Zane Teegarden".to_string(),
        "Eden Socia".to_string(),
        "Jerald Winebarger".to_string(),
        "Roslyn Brodsky".to_string(),
        "Signe Esters".to_string(),
        "Amberly Munro".to_string(),
        "Terrell Crisci".to_string(),
        "Jenna Brim".to_string(),
        "Drema Maranto".to_string(),
        "Stefan Work".to_string(),
        "Shanel Shealy".to_string(),
        "Sima Brinkman".to_string(),
        "Clark Sudduth".to_string(),
        "Haywood Oden".to_string(),
        "Wilhelmina Abee".to_string(),
        "Jeanine Ramaker".to_string(),
        "Katherin Job".to_string(),
        "Keturah Flythe".to_string(),
        "Sharyn Heger".to_string(),
        "Jacquie Baade".to_string(),
        "Clemmie Pagaduan".to_string(),
        "Emiko Hundt".to_string(),
        "Hae Atterberry".to_string(),
        "Mayola Faler".to_string(),
        "Hannah Ogilvie".to_string(),
        "Pamila Newkirk".to_string(),
        "Javier Laub".to_string(),
        "Broderick Golston".to_string(),
    ]
}
