use counter::Counter;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

use crate::core::error::wrap_error;
use crate::core::idf::HasDocument;

/******************************************************************************/
/* Basic name types not suitable for matching                                 */
/******************************************************************************/

/// An unprocessed Name capable of serialization from/to a tabular data file.
#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct Name {
    #[getset(get = "pub")]
    #[serde(rename = "name")]
    unprocessed: String,
    #[getset(get = "pub")]
    #[serde(rename = "id", default)]
    idx: String,
    // group: String,
}

impl Name {
    pub fn from_csv(file_path: &str) -> Result<Vec<Name>, Box<dyn Error>> {
        let file =
            File::open(file_path).map_err(|e| wrap_error(e, format!("accessing {}", file_path)))?;
        let mut rdr = csv::Reader::from_reader(file);

        rdr.deserialize()
            // .into_iter()
            .map(|result| {
                let record: Name = result
                    .map_err(|e| wrap_error(e, format!("reading data from {}", file_path)))?;
                Ok(record)
            })
            .collect()
    }
}

/// A processed Name with a counter for each token, use the new constructor
/// with a passed in text processing function.
#[derive(Debug, Getters)]
pub struct NameProcessed {
    #[getset(get = "pub")]
    pub name: Name,
    #[getset(get = "pub")]
    pub token_counter: Counter<String>,
}

impl NameProcessed {
    pub fn new<I>(name: Name, tokens: I) -> Self
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

impl HasDocument for NameProcessed {
    fn get_tokens(&self) -> Vec<&String> {
        // self.token_counter().keys().into_iter().collect()
        self.token_counter().keys().collect()
    }
}
