mod cli;
mod core;
mod matching;
mod name;
mod preprocess;

use colored::Colorize;
use structopt::StructOpt;

use crate::cli::MatchModeEnum;
use crate::matching::execute_match;
use crate::name::{NameGrouped, NameUngrouped};

fn main() {
    let opt = MatchModeEnum::from_args();

    if let Some(threads) = opt.get_cli().threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .unwrap();
    }

    let res = match opt.get_cli().group_match {
        true => execute_match::<NameGrouped>(&opt),
        false => execute_match::<NameUngrouped>(&opt),
    };

    if let Err(e) = res {
        let error = "error".red();
        println!("{}: {}", error, e)
    }
}
