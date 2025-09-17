use crate::common_interface::BenchmarkFmIndex;
use crate::{Config, print_after_build_metrics, read_queries, read_texts};

use fm_index::Search;

pub type FMIndexCrateFmIndex = fm_index::FMIndexMultiPiecesWithLocate<u8>;

impl BenchmarkFmIndex for FMIndexCrateFmIndex {
    fn count_for_benchmark(&self, query: &[u8]) -> usize {
        self.search(&query).count()
    }

    fn count_via_locate_for_benchmark(&self, query: &[u8]) -> usize {
        self.search(&query).iter_matches().count()
    }
}

// Supports multiple different FM-Index variants.
pub fn fmindex(config: Config) {
    let text: Vec<_> = read_texts(&config)
        .into_iter()
        .map(|mut t| {
            t.push(0);
            t
        })
        .flatten()
        .collect();
    let text = fm_index::Text::new(text);

    assert!(config.suffix_array_sampling_rate.is_power_of_two());
    let sampling_level = config.suffix_array_sampling_rate.ilog2() as usize;

    let start = std::time::Instant::now();
    let index = fm_index::FMIndexMultiPiecesWithLocate::new(&text, sampling_level).unwrap();
    // let index = fm_index::FMIndexWithLocate::new(&text, sampling_level).unwrap(); <-- didn't improve memory usage
    print_after_build_metrics(start);

    let queries = read_queries(&config);

    index.run_search_benchmark(&config, &queries);
}
