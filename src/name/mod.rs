pub mod base;
pub mod levenshtein;
pub mod ngram;
pub mod score;
pub mod weighted;

pub use crate::name::base::*;
pub use crate::name::levenshtein::*;
pub use crate::name::ngram::*;
pub use crate::name::weighted::*;

use crate::name::score::*;
