use crate::Config;
use crate::common_interface::BenchmarkFmIndex;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use awry::{alphabet, fm_index};

// Applies many smart tricks. Writes diagnostics to stdout, which libraries usually don't do.
pub type AwryFmIndex = awry::fm_index::FmIndex;

impl BenchmarkFmIndex for AwryFmIndex {
    type Stub<'a> = &'a Self;

    fn construct_for_benchmark(config: &Config, _texts: Option<Vec<Vec<u8>>>) -> Self {
        let build_args = fm_index::FmBuildArgs {
            input_file_src: config.input_texts.get_filepath(),
            suffix_array_output_src: Some(
                PathBuf::from_str("indices/awry_temporary_suffix_array_output.txt").unwrap(),
            ),
            suffix_array_compression_ratio: Some(config.suffix_array_sampling_rate as u64),
            lookup_table_kmer_len: Some(config.depth_of_lookup_table as u8),
            alphabet: alphabet::SymbolAlphabet::Nucleotide,
            // for now, awry doesn't get the max_query_len advantage, because it would make the whole benchmark setup more complicated
            // as different indiced would have to be stored for different query lengths
            max_query_len: None,
            remove_intermediate_suffix_array_file: true,
        };

        fm_index::FmIndex::new(&build_args).unwrap()
    }

    fn supports_file_io_for_benchmark() -> bool {
        true
    }

    fn write_to_file_for_benchmark(self, path: &Path) {
        self.save(path).unwrap();
    }

    fn load_from_file_for_benchmark(path: &Path) -> Self {
        Self::load(path).unwrap()
    }

    fn as_stub_for_benchmark<'a>(&'a self) -> Self::Stub<'a> {
        self
    }

    fn count_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        let query = str::from_utf8(query).unwrap();
        index.count_string(query) as usize
    }

    fn count_via_locate_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        let query = str::from_utf8(query).unwrap();
        index.locate_string(&query).len()
    }
}
