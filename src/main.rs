mod cli;
mod core;
mod matching;
mod name;
mod preprocess;

use structopt::StructOpt;

use crate::cli::MatchModeEnum;
use crate::matching::execute_match;
use crate::name::{NameGrouped, NameUngrouped};

fn main() {
    let opt = MatchModeEnum::from_args();

    let res = match opt.get_cli().group_match {
        true => execute_match::<NameGrouped>(&opt),
        false => execute_match::<NameUngrouped>(&opt),
    };

    if let Err(e) = res {
        println!("{}", e)
    }
}

// use ansi_term::Colour::Red;
// use std::error::Error;
// use std::fmt::Display;
// impl<T: Error> Display for YentaWrappedError<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
//         writeln!(f, "{}: {}", Red.paint("error"), self.msg)?;
//         std::fmt::Display::fmt(&self.original_error, f)
//     }
// }
