use crate::common_interface::BenchmarkFmIndex;
use crate::{Config, print_after_build_metrics, read_queries, read_texts};
use genedex::block::{Block, BlockLayout};
use genedex::{FmIndex, FmIndexConfig, IndexStorage, alphabet};
use log::info;

pub type GenedexFMIndex<I, B, L> = FmIndex<I, B, L>;

impl<I: IndexStorage, B: Block, L: BlockLayout> BenchmarkFmIndex for GenedexFMIndex<I, B, L> {
    fn count_for_benchmark(&self, query: &[u8]) -> usize {
        self.count(&query)
    }

    fn count_via_locate_for_benchmark(&self, query: &[u8]) -> usize {
        self.locate(&query).count()
    }
}

pub fn genedex<I: IndexStorage, B: Block, L: BlockLayout>(config: Config) {
    let index_filepath = config.index_filepath();

    let start = std::time::Instant::now();
    let index = if config.skip_build {
        FmIndex::<I, B, L>::load_from_file(&index_filepath).unwrap()
    } else {
        let texts = read_texts(&config);

        FmIndexConfig::<I, B, L>::new()
            .lookup_table_depth(config.depth_of_lookup_table)
            .suffix_array_sampling_rate(config.suffix_array_sampling_rate)
            .construct_index(texts, alphabet::ascii_dna_with_n())
    };

    print_after_build_metrics(start);

    let queries = read_queries(&config);

    index.run_search_benchmark(&config, &queries);

    if !std::fs::exists(&index_filepath).unwrap() || config.force_write_and_load {
        let start = std::time::Instant::now();
        index.save_to_file(&index_filepath).unwrap();
        info!(
            "Write to disk time: {:.2} seconds",
            start.elapsed().as_millis() as f64 / 1_000.0
        );

        let start = std::time::Instant::now();
        let index = FmIndex::<I, B>::load_from_file(&index_filepath).unwrap();
        info!(
            "Load from disk time: {:.2} seconds (dummy: {})",
            start.elapsed().as_millis() as f64 / 1_000.0,
            index.count(b"ACGT")
        );
    }
}
