# yenta - a fast fuzzy name matcher for CSV files

# Installation

- Install Rust
- Clone this repository
- At the command line: `cargo install --release`

# To Do

- Parallel inside match function instead of at the name level
- Benchmark BTreeMap/BTreeSet vs HashMap/HashSet
- Options from Yente that need to be implemented
    * --phonix                 Preprocess words with Phonix algorithm.
    * --levenshtein-penalty DOUBLE
    * The Levenshtein edit distance penalty factor (percent correct letters raised to factor is multiplied by token score)
    * --ngram-size INT         The size of ngrams to use (2 is recommended to start)
    * -g,--subgroup-search     Search for matches in groups (requires 'group' column in data files)
    * -h,--help                Show this help text

Unsupported 
