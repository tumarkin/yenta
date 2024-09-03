use std::collections::{HashMap, HashSet};

use counter::Counter;

/// TokenDocument trait yields a vector of tokens representing a document for
/// computing an Idf.
pub trait TokenDocument {
    fn token_document(&self) -> Vec<&String>;
}

/// Inverse document frequency values
#[derive(Debug)]
pub struct Idf {
    weight_map: HashMap<String, f64>,
    weight_for_missing: f64,
}

impl Idf {
    pub fn new<T>(docs: &Vec<T>) -> Self
    where
        T: TokenDocument,
    {
        let mut df = DocumentFrequency::new();

        for d in docs {
            df.add_document(d);
        }

        let num_docs = df.num_docs;
        let ln_num_docs = (num_docs as f64).ln();

        let idf = df
            .document_frequency
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    ln_num_docs - (*v as f64).ln(), // f64::value_from(*v).unwrap().ln(),
                )
            })
            .collect();
        Idf {
            weight_map: idf,
            weight_for_missing: ln_num_docs,
        }
    }

    pub fn lookup(&self, token: &str) -> f64 {
        self.weight_map
            .get(token)
            .map_or(self.weight_for_missing, |v| *v)
    }
}

/******************************************************************************/
/*  Document   related   types                                                */
/******************************************************************************/

/// Document frequency counter which can be converted to an Idf.
#[derive(Debug)]
struct DocumentFrequency {
    num_docs: usize,
    document_frequency: Counter<String>,
}

impl DocumentFrequency {
    fn new() -> Self {
        DocumentFrequency {
            num_docs: 0,
            document_frequency: Counter::new(),
        }
    }

    fn add_document<T>(&mut self, doc: &T)
    where
        T: TokenDocument,
    {
        let unique_tokens: HashSet<&String> = doc.token_document().into_iter().collect();
        //         name_processed.token_counter.keys().into_iter().collect();
        for k in unique_tokens {
            self.document_frequency[k] += 1;
        }
        self.num_docs += 1
    }
}
