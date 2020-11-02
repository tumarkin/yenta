use deunicode::deunicode;
use rayon::prelude::*;

use crate::core::{Name, NameProcessed, PreprocessingOptions};

pub fn prep_names(names: Vec<Name>, prep_opts: &PreprocessingOptions) -> Vec<NameProcessed> {
    names
        .into_par_iter()
        .map(|n| prep_name(n, &prep_opts))
        .collect()
}

pub fn prep_name(name: Name, prep_opts: &PreprocessingOptions) -> NameProcessed {
    let tokens = prep_words(&name.unprocessed(), &prep_opts);

    NameProcessed::new(name, tokens)
}

pub fn prep_words(source_string: &str, opts: &PreprocessingOptions) -> Vec<String> {
    source_string
        .split_ascii_whitespace()
        .map(|word| {
            PrepString(word.to_string())
                .deunicode(!opts.retain_unicode)
                .to_ascii_lowercase(opts.adjust_case)
                .filter_alphabetic(!opts.retain_non_alphabetic)
                .soundex(opts.soundex)
                .trim_length(opts.token_length)
                .0
        })
        .collect()
}

/// A newtype that allows for nicer chaning of functions during text preprocessing
struct PrepString(String);

impl PrepString {
    fn deunicode(self, execute: bool) -> Self {
        if execute {
            PrepString(deunicode(&self.0))
        } else {
            self
        }
    }

    fn to_ascii_lowercase(self, execute: bool) -> Self {
        if execute {
            PrepString(self.0.to_ascii_lowercase())
        } else {
            self
        }
    }

    fn filter_alphabetic(self, execute: bool) -> Self {
        if execute {
            PrepString(self.0.chars().filter(|&c| c.is_alphabetic()).collect())
        } else {
            self
        }
    }

    fn soundex(self, execute: bool) -> Self {
        if execute {
            PrepString(soundex::american_soundex(&self.0))
        } else {
            self
        }
    }

    fn trim_length(self, len: Option<usize>) -> Self {
        match len {
            Some(i) => PrepString(self.0.chars().take(i).collect()),
            None => self,
        }
    }
}
