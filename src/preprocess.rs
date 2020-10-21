use deunicode::deunicode;
use rayon::prelude::*;

use crate::core::name::{Name, NameProcessed};

pub fn prep_names(names: Vec<Name>, prep_opts: &PreprocessingOptions) -> Vec<NameProcessed> {
    names
        .into_par_iter()
        .map(|n| prep_name(n, &prep_opts))
        .collect()
}

pub fn prep_name(name: Name, prep_opts: &PreprocessingOptions) -> NameProcessed {
    let token_counter = prep_words(&name.unprocessed(), &prep_opts)
        .into_iter()
        .collect();

    NameProcessed::new(name, token_counter)
}

pub fn prep_words(source_string: &str, opts: &PreprocessingOptions) -> Vec<String> {
    source_string
        .split_ascii_whitespace()
        .map(|word| {
            PrepString(word.to_string())
                .deunicode(opts.adjust_unicode)
                .to_ascii_lowercase(opts.adjust_case)
                .filter_alphabetic(opts.alphabetic_only)
                .soundex(opts.soundex)
                .trim_length(opts.trim_length)
                .0
        })
        .collect()
}

#[derive(Debug)]
pub struct PreprocessingOptions {
    pub adjust_unicode: bool,
    pub adjust_case: bool,
    pub alphabetic_only: bool,
    pub soundex: bool,
    pub trim_length: Option<usize>,
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
