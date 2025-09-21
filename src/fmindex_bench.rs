use crate::Config;
use crate::common_interface::BenchmarkFmIndex;

use fm_index::Search;

// Supports multiple different FM-Index variants.
pub type FMIndexCrateSingleFmIndex = fm_index::FMIndexWithLocate<u8>;
pub type FMIndexCrateMultiFmIndex = fm_index::FMIndexMultiPiecesWithLocate<u8>;

impl BenchmarkFmIndex for FMIndexCrateMultiFmIndex {
    type Stub<'a> = &'a Self;

    fn construct_for_benchmark(config: &Config, texts: Option<Vec<Vec<u8>>>) -> Self {
        let text: Vec<_> = texts
            .unwrap()
            .into_iter()
            .flat_map(|mut t| {
                t.push(0);
                t
            })
            .collect();
        let text = fm_index::Text::new(text);

        assert!(config.suffix_array_sampling_rate.is_power_of_two());
        let sampling_level = config.suffix_array_sampling_rate.ilog2() as usize;

        // let index = fm_index::FMIndexWithLocate::new(&text, sampling_level).unwrap(); <-- didn't improve memory usage
        fm_index::FMIndexMultiPiecesWithLocate::new(&text, sampling_level).unwrap()
    }

    fn as_stub_for_benchmark<'a>(&'a self) -> Self::Stub<'a> {
        self
    }

    fn count_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        index.search(query).count()
    }

    fn count_via_locate_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        index.search(query).iter_matches().count()
    }
}

impl BenchmarkFmIndex for FMIndexCrateSingleFmIndex {
    type Stub<'a> = &'a Self;

    fn construct_for_benchmark(config: &Config, texts: Option<Vec<Vec<u8>>>) -> Self {
        let text: Vec<_> = texts
            .unwrap()
            .into_iter()
            .flat_map(|mut t| {
                t.push(0);
                t
            })
            .collect();
        let text = fm_index::Text::new(text);

        assert!(config.suffix_array_sampling_rate.is_power_of_two());
        let sampling_level = config.suffix_array_sampling_rate.ilog2() as usize;

        fm_index::FMIndexWithLocate::new(&text, sampling_level).unwrap()
    }

    fn as_stub_for_benchmark<'a>(&'a self) -> Self::Stub<'a> {
        self
    }

    fn count_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        index.search(query).count()
    }

    fn count_via_locate_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        index.search(query).iter_matches().count()
    }
}
