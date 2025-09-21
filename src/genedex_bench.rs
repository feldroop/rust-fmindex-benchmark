use std::path::Path;

use crate::common_interface::BenchmarkFmIndex;
use crate::{Config, ExtraBuildArg};
use genedex::text_with_rank_support::TextWithRankSupport;
use genedex::{FmIndex, FmIndexConfig, IndexStorage, PerformancePriority, alphabet};

pub type GenedexFMIndex<I, R> = FmIndex<I, R>;

impl<I: IndexStorage, R: TextWithRankSupport<I>> BenchmarkFmIndex for GenedexFMIndex<I, R> {
    type Stub<'a> = &'a Self;

    fn construct_for_benchmark(config: &Config, texts: Option<Vec<Vec<u8>>>) -> Self {
        let performance_priority = match config.extra_build_arg {
            Some(ExtraBuildArg::LowMemory) => PerformancePriority::LowMemory,
            Some(ExtraBuildArg::MediumMemory) => PerformancePriority::Balanced,
            _ => PerformancePriority::HighSpeed,
        };

        FmIndexConfig::<I, R>::new()
            .lookup_table_depth(config.depth_of_lookup_table)
            .suffix_array_sampling_rate(config.suffix_array_sampling_rate)
            .construction_performance_priority(performance_priority)
            .construct_index(texts.unwrap(), alphabet::ascii_dna_with_n())
    }

    fn supports_file_io_for_benchmark() -> bool {
        true
    }

    fn write_to_file_for_benchmark(self, path: &Path) {
        self.save_to_file(path).unwrap();
    }

    fn load_from_file_for_benchmark(path: &Path) -> Self {
        Self::load_from_file(path).unwrap()
    }

    fn as_stub_for_benchmark<'a>(&'a self) -> Self::Stub<'a> {
        self
    }

    fn count_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        index.count(query)
    }

    fn count_via_locate_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        index.locate(query).count()
    }
}
