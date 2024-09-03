use anyhow;
use counter::Counter;
use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::core::idf::TokenDocument;
use crate::core::vec_from_csv;

pub trait UnprocessedName {
    fn unprocessed_name(&self) -> &str;
    fn idx(&self) -> &str;
    fn from_csv(file_path: &str) -> anyhow::Result<Vec<Self>>
    where
        Self: Sized;
}

pub trait NameContainer<N> {
    fn get_name(&self) -> &N;
}

/// An unprocessed Name capable of serialization from/to a tabular data file.
#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct NameUngrouped {
    #[getset(get = "pub")]
    #[serde(rename = "name")]
    unprocessed: String,
    #[getset(get = "pub")]
    #[serde(rename = "id", default)]
    idx: String,
}

impl UnprocessedName for NameUngrouped {
    fn unprocessed_name(&self) -> &str {
        &self.unprocessed
    }

    fn idx(&self) -> &str {
        &self.idx
    }

    fn from_csv(file_path: &str) -> anyhow::Result<Vec<NameUngrouped>> {
        vec_from_csv(file_path)
    }
}

/// An unprocessed Name capable of serialization from/to a tabular data file. Includes group
/// identifier string.
#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct NameGrouped {
    #[getset(get = "pub")]
    #[serde(rename = "name")]
    unprocessed: String,
    #[getset(get = "pub")]
    #[serde(rename = "id", default)]
    idx: String,
    #[getset(get = "pub")]
    group: String,
}

impl UnprocessedName for NameGrouped {
    fn unprocessed_name(&self) -> &str {
        &self.unprocessed
    }

    fn idx(&self) -> &str {
        &self.idx
    }

    fn from_csv(file_path: &str) -> anyhow::Result<Vec<NameGrouped>> {
        vec_from_csv(file_path)
    }
}

/// A processed Name with a counter for each token, use the new constructor
/// with a passed in text processing function.
#[derive(Debug, Getters)]
pub struct NameProcessed<N> {
    #[getset(get = "pub")]
    pub name: N,
    #[getset(get = "pub")]
    pub token_counter: Counter<String>,
}

impl<N> NameProcessed<N> {
    pub fn new<I>(name: N, tokens: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let token_counter: Counter<String> = tokens.into_iter().collect();
        NameProcessed {
            name,
            token_counter,
        }
    }
}

impl<N> TokenDocument for NameProcessed<N> {
    fn token_document(&self) -> Vec<&String> {
        // self.token_counter().keys().into_iter().collect()
        self.token_counter().keys().collect()
    }
}
