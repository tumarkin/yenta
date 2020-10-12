use conv::*;
use counter::Counter;
use std::collections::{HashMap, HashSet};

// An unprocessed Name suitable for serialization from/to a CSV or similar file.
#[derive(Debug)]
pub struct Name {
    pub unprocessed: String,
    pub idx: String,
    // group: String,
}

impl HasName for Name {
    fn name(&self) -> &Name {
        &self
    }
}

// A processed Name with a counter for each token, use the new constructor
// with a passed in text processing function.
#[derive(Debug)]
pub struct NameProcessed {
    pub name: Name,
    pub token_counter: Counter<String>,
}

// impl NameProcessed {
//     fn new<F>(name: Name, text_processor: F) -> NameProcessed
//     where
//         F: Fn(&str) -> Vec<String>,
//     {
//         let token_counter: Counter<String> =
//             text_processor(&name.unprocessed).into_iter().collect();
//         NameProcessed {
//             name,
//             token_counter,
//         }
//     }
// }

impl HasName for NameProcessed {
    fn name(&self) -> &Name {
        &self.name
    }
}

// HasName defines a uniform interface for a group of data structures that contain a
// name.
pub trait HasName {
    fn name(&self) -> &Name;

    // fn same_group<T>(&self, other: &T) -> bool
    // where
    //     T: HasName,
    // {
    //     self.name().group == other.name().group
    // }
}

/// Document frequency counter which can be converted to an IDF.
#[derive(Debug)]
struct DocumentFrequency {
    num_docs: usize,
    document_frequency: Counter<String>,
}

impl DocumentFrequency {
    fn new() -> DocumentFrequency {
        DocumentFrequency {
            num_docs: 0,
            document_frequency: Counter::new(),
        }
    }

    fn add_name(&mut self, name_processed: &NameProcessed) -> () {
        let unique_tokens: HashSet<&String> = name_processed.token_counter.keys().into_iter().collect();
        for k in unique_tokens {
            self.document_frequency[k] += 1;
        }
        self.num_docs += 1
    }
}


/// Inverse document frequency values
#[derive(Debug)]
pub struct IDF(HashMap<String, f64>);

impl IDF {
    pub fn new(names: &Vec<NameProcessed>) -> IDF {
        let mut df = DocumentFrequency::new();

        for n in names {
            df.add_name(n);
        }
        
        let num_docs = df.num_docs;
        let ln_num_docs = f64::value_from(num_docs).unwrap().ln();

        let idf = df
            .document_frequency
            .iter()
            .map(|(k, v)| (k.to_string(), ln_num_docs - f64::value_from(*v).unwrap().ln()))
            .collect();
       IDF(idf)
    }
}

/**************************************************************************************************/
/* Tests                                                                                          */
/**************************************************************************************************/
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn name_same_group() {
        let n1 = Name {
            unprocessed: "John Smith".to_string(),
            idx: "id1".to_string(),
            group: "Group A".to_string(),
        };

        let n2 = Name {
            unprocessed: "Smith Smithington".to_string(),
            idx: "id2".to_string(),
            group: "Group A".to_string(),
        };
        let n3 = Name {
            unprocessed: "Smith Smithington".to_string(),
            idx: "id3".to_string(),
            group: "No group".to_string(),
        };
        assert_eq!(n1.same_group(&n2), true);
        assert_eq!(n1.same_group(&n3), false);
        assert_eq!(n2.same_group(&n3), false);

        // #[test]
        // fn test_item_removal() {
        //     let mut c = NonZeroTokenCounter::new();
        //     c = c.add_item(&"the".to_string());
        //     c = c.add_item(&"quick".to_string());
        //     c = c.add_item(&"fox".to_string());
        //     c = c.add_item(&"fox".to_string());

        //     assert_eq!(c.0[&"the".to_string()], 1);
        //     assert_eq!(c.0[&"quick".to_string()], 1);
        //     assert_eq!(c.0[&"fox".to_string()], 2);

        //     println!("{:?}", c);
        // }
    }
}
