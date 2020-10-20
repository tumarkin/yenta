// extern crate csv;

use std::error::Error;
use std::fs::File;

use crate::core::name::Name;

pub fn read_name_csv(file_path: &str) -> Result<Vec<Name>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);

    rdr.deserialize()
        .into_iter()
        .map(|result| {
            let record: Name = result?;
            Ok(record)
        })
        .collect()
}
