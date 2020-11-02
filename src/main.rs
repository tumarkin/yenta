mod core;
mod matching;
mod preprocess;

use structopt::StructOpt;

use crate::core::MatchModeEnum;
use crate::matching::execute_match;

fn main() {
    // let opt = CLI::from_args();
    let opt = MatchModeEnum::from_args();
    let res = execute_match(&opt);

    // println!("{:?}", opt);
    if let Err(e) = res {
        println!("{}", e)
    }
}

//         .after_help("More details and Wiki at github.com/tumarkin/yenta")
