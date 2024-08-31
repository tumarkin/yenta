mod core;
mod matching;
mod preprocess;

use structopt::StructOpt;

use crate::core::MatchModeEnum;
use crate::core::{NameGrouped, NameUngrouped};
use crate::matching::execute_match;

fn main() {
    let opt = MatchModeEnum::from_args();

    let res = match opt.get_cli().group_match {
        true => execute_match::<NameGrouped>(&opt),
        false => execute_match::<NameUngrouped>(&opt),
    };

    // println!("{:?}", opt);
    if let Err(e) = res {
        println!("{}", e)
    }
}
