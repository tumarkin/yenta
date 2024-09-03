use std::fs::File;

use anyhow::Context;
use serde::de::DeserializeOwned;

pub fn vec_from_csv<T>(file_path: &str) -> anyhow::Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let file = File::open(file_path).with_context(|| format!("accessing {}", file_path))?;
    let mut rdr = csv::Reader::from_reader(file);

    rdr.deserialize()
        // .into_iter()
        .map(|result| {
            let record: T = result.with_context(|| format!("reading data from {}", file_path))?;
            Ok(record)
        })
        .collect()
}
