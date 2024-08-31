pub mod base;
pub mod levenshtein;
pub mod ngram;
pub mod score;
pub mod weighted;

pub use crate::core::name::base::*;
pub use crate::core::name::levenshtein::*;
pub use crate::core::name::ngram::*;
pub use crate::core::name::weighted::*;

use crate::core::name::score::*;
