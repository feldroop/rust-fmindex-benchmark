use crate::common_interface::BenchmarkFmIndex;
use crate::{Config, SearchMode, print_after_build_metrics, read_queries, read_texts};
use log::info;
use std::path::PathBuf;
use std::str::FromStr;

use awry::{alphabet, fm_index};

pub type AwryFmIndex = awry::fm_index::FmIndex;

impl BenchmarkFmIndex for AwryFmIndex {
    fn count_for_benchmark(&self, query: &[u8]) -> usize {
        let query = str::from_utf8(query).unwrap();
        self.count_string(query) as usize
    }

    fn count_via_locate_for_benchmark(&self, query: &[u8]) -> usize {
        let query = str::from_utf8(query).unwrap();
        self.locate_string(&query).len()
    }
}

// I don't fully understand the requirements for a temporary suffix array file.
// `&String` as input for search functions is not idiomatic Rust. I believe it should be `&[u8]` in this case.
// Applies many smart tricks. Writes diagnostics to stdout, which libraries usually don't do.
pub fn awry(config: Config) {
    let input_texts_path = PathBuf::from(format!(
        "data/input_texts_{}_records.fasta",
        config
            .num_text_records
            .map_or("all".to_string(), |n| n.to_string())
    ));

    // prepare the input data file. this is only necessary because of the benchmark setup.
    // (I want to sometimes build the index only for a part of the data set)
    if !std::fs::exists(&input_texts_path).unwrap() {
        let texts = read_texts(&config);
        let slice = config.num_text_records.map_or(texts.as_slice(), |l| {
            &texts[..std::cmp::min(l, texts.len())]
        });

        let mut writer = bio::io::fasta::Writer::to_file(&input_texts_path).unwrap();

        for (i, text) in slice.iter().enumerate() {
            writer.write(&i.to_string(), None, text).unwrap();
        }
    }

    let index_filepath = config.index_filepath();

    let build_args = fm_index::FmBuildArgs {
        input_file_src: input_texts_path,
        suffix_array_output_src: Some(
            PathBuf::from_str("indices/awry_temporary_suffix_array_output.txt").unwrap(),
        ),
        suffix_array_compression_ratio: Some(config.suffix_array_sampling_rate as u64),
        lookup_table_kmer_len: Some(config.depth_of_lookup_table as u8),
        alphabet: alphabet::SymbolAlphabet::Nucleotide,
        // for now, awry doesn't get this advantage, because it would make the whole benchmark setup more complicated
        max_query_len: None,
        remove_intermediate_suffix_array_file: true,
    };

    let start = std::time::Instant::now();
    let index = if config.skip_build {
        fm_index::FmIndex::load(&index_filepath).unwrap()
    } else {
        fm_index::FmIndex::new(&build_args).unwrap()
    };

    print_after_build_metrics(start);

    let queries = read_queries(&config);

    if let SearchMode::Locate = config.search_mode {
        info!("Currently, the awry locate function is broken");
    } else {
        index.run_search_benchmark(&config, &queries);
    }

    if !std::fs::exists(&index_filepath).unwrap() || config.force_write_and_load {
        let start = std::time::Instant::now();
        index.save(&index_filepath).unwrap();
        info!(
            "Write to disk time: {:.2} seconds",
            start.elapsed().as_millis() as f64 / 1_000.0
        );

        let start = std::time::Instant::now();
        let index = fm_index::FmIndex::load(&index_filepath).unwrap();
        info!(
            "Load from disk time: {:.2} seconds (dummy: {})",
            start.elapsed().as_millis() as f64 / 1_000.0,
            index.count_string(&String::from("ACGT"))
        );
    }
}
