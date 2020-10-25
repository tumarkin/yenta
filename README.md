# yenta

A fast, fuzzy matchmaker for textual data. 

# Overview

*yenta* matches names across two data files. It has the following features:

* **Intelligent**: Matching is based on rareness of words, which means that one does not need to preprocess the names to remove common, non-informative words in names (i.e. and, the, company). Just feed your data in to the program and get results.
* **Robust**: *yenta* incorporates feautes that are commonly needed in name matching. It is both word-order and case insensitive (Shawn Spencer matches SPENCER, SHAWN). *yenta* removes punctuation by default.
* **Unicode aware**: By default, *yenta* automatically converts unicode accented characters to their ASCII equivalents.
* **Customizable**: Users may optionally allow for misspellings, implement phonetic algorithms, trim the constituent words of a name at a prespecified number of characters, output any number of potential matches (with and without ties), and combine any of the preceding customizations.
* **High performance**: *yenta* is a multi-core program written in [Rust](https://www.rust-lang.org/), a blazingly fast and memory-efficient language.

# Installation

- Install [Rust](https://www.rust-lang.org/tools/install)
- Clone this repository
- At the command line, change to the root of the cloned repository and then type: `cargo install --release`

# Quick Start

Save your data files in CSV format. You will match names from one file to potential matches in a second file. Assume that the first file is called `from_names.csv` and the second file is called `to_names.csv`. `yenta` requires that each of your CSV files has a column called *name*, in lower case. This column will be used by the fuzzy matcher. You may also have an optional column called *id*, which, if used, simply serves as a reference identifier that is echoed to the output.

On the command line, `cd` into the directory with your files. To create an output file called `matches.csv` use the following command:

`yenta from_names.csv to_names.csv --output-file=matches.csv`

# Recipes



# Information

See the [wiki](https://github.com/tumarkin/yenta/wiki) for information on installation, usage, and best practices. It also includes some examples for matching problems that commonly arise in research.

# Contributing

Submit a pull request and I will respond.

If *yenta* has in any way made your life easier, please send me an email or star this repository. If you would like to see a feature added, let me know through the Github forum.

# To Do

- [] Improved parallelism for correctly ordered output
- [] Benchmark BTreeMap/BTreeSet vs HashMap/HashSet
- [] Match Modes
	- [] Exact token
	- [] Ngram
	- [] Levenshtein
- [] Subgroup search
- [] CLI error reporting
- [] NameProcessed::new takes token iterator instead of Counter

