use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about="A matchmaker for text files")]
// #[structopt(author="Robert Tumarkin <r.tumarkin@unsw.edu.au>")]
pub enum MatchModeEnum {
    #[structopt(name = "token")]
    /// Exact match on processed tokens
    ExactMatch {
        #[structopt(flatten)]
        em_cli: CLI,
    },
    /// Fuzzy match using n-grams on processed tokens
    #[structopt(name = "ngram")]
    NGramMatch {
        #[structopt(long = "ngram-size", default_value = "2")]
        /// Length of n-grams in characters
        n_gram_length: usize,

        #[structopt(flatten)]
        ng_cli: CLI,
    },
}

impl MatchModeEnum {
    pub fn get_cli(&self) -> &CLI {
        match self {
            MatchModeEnum::ExactMatch { em_cli } => &em_cli,
            MatchModeEnum::NGramMatch { ng_cli, .. } => &ng_cli,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct CLI {
    #[structopt(flatten)]
    pub io_args: IoArgs,
    #[structopt(flatten)]
    pub preprocessing_options: PreprocessingOptions,
    #[structopt(flatten)]
    pub match_options: MatchOptions,
}

// #[structopt(subcommand)]
// pub match_mode_enum: MatchModeEnum,

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct IoArgs {
    /// Match names from this file...
    pub from_file: String,
    /// ...to names in this file
    pub to_file: String,
    #[structopt(long, short)]
    /// Save matches to this filepath (REQUIRED)
    pub output_file: String,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct PreprocessingOptions {
    #[structopt(long)]
    /// Do not convert unicode characters to ASCII equivalents
    pub retain_unicode: bool,
    #[structopt(skip)]
    pub adjust_case: bool,
    #[structopt(long)]
    /// Retain non-alphabetic characters
    pub retain_non_alphabetic: bool,
    #[structopt(long)]
    /// Process words with SoundEx algorithm
    pub soundex: bool,
    #[structopt(long, short)]
    /// Trim each word to have a maximum number of characters
    pub token_length: Option<usize>,
}

#[derive(Debug, StructOpt)]
pub struct MatchOptions {
    // pub match_mode: MatchMode,
    #[structopt(long = "minimum-match-score", short)]
    #[structopt(default_value = "0.01")]
    /// The minimum score required to be considered a match
    pub minimum_score: f64,
    #[structopt(long = "number-of-results", short)]
    #[structopt(default_value = "1")]
    /// The number of results to output
    pub num_results: usize,
    #[structopt(long = "include-ties-within", short = "i")]
    /// Include ties within FLOAT of the nth requested result
    pub ties_within: Option<f64>,
}
