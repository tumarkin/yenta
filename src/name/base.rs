use anyhow;
use anyhow::Context;
use counter::Counter;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::fs::File;

use crate::core::idf::HasDocument;

pub trait IsName {
    type PotentialMatchesLookup;

    fn unprocessed_name(&self) -> &str;
    fn idx(&self) -> &str;
    fn from_csv(file_path: &str) -> anyhow::Result<Vec<Self>>
    where
        Self: Sized;
}

pub trait HasName<N> {
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

impl IsName for NameUngrouped {
    type PotentialMatchesLookup = ();

    fn unprocessed_name(&self) -> &str {
        &self.unprocessed
    }

    fn idx(&self) -> &str {
        &self.idx
    }

    fn from_csv(file_path: &str) -> anyhow::Result<Vec<NameUngrouped>> {
        let file =
            // File::open(file_path).map_err(|e| wrap_error(e, format!("accessing {}", file_path)))?;
            File::open(file_path).with_context(|| format!("accessing {}", file_path))?;
        let mut rdr = csv::Reader::from_reader(file);

        rdr.deserialize()
            // .into_iter()
            .map(|result| {
                let record: NameUngrouped = result
                    // .map_err(|e| wrap_error(e, format!("reading data from {}", file_path)))?;
                    .with_context(|| format!("reading data from {}", file_path))?;
                Ok(record)
            })
            .collect()
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

impl IsName for NameGrouped {
    type PotentialMatchesLookup = ();
    fn unprocessed_name(&self) -> &str {
        &self.unprocessed
    }

    fn idx(&self) -> &str {
        &self.idx
    }
    fn from_csv(file_path: &str) -> anyhow::Result<Vec<NameGrouped>> {
        let file =
            // File::open(file_path).map_err(|e| wrap_error(e, format!("accessing {}", file_path)))?;
            File::open(file_path).with_context(|| format!("accessing {}", file_path))?;
        let mut rdr = csv::Reader::from_reader(file);

        rdr.deserialize()
            // .into_iter()
            .map(|result| {
                let record: NameGrouped = result
                    // .map_err(|e| wrap_error(e, format!("reading data from {}", file_path)))?;
                    .with_context(|| format!("reading data from {}", file_path))?;
                Ok(record)
            })
            .collect()
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

impl<N> HasDocument for NameProcessed<N> {
    fn get_tokens(&self) -> Vec<&String> {
        // self.token_counter().keys().into_iter().collect()
        self.token_counter().keys().collect()
    }
}
