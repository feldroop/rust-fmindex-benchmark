use std::path::Path;

use log::info;

use crate::{Config, SearchMode, print_after_build_metrics, read_queries, read_texts};

pub trait BenchmarkFmIndex: Sized {
    // this interface is a bit complicated, because the sview fmindex is essentially a reference to a slice and also is not Copy
    type Stub<'a>
    where
        Self: 'a;

    fn as_stub_for_benchmark<'a>(&'a self) -> Self::Stub<'a>;

    fn construct_for_benchmark(config: &Config, texts: Option<Vec<Vec<u8>>>) -> Self;

    fn count_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize;

    fn count_via_locate_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize;

    fn supports_file_io_for_benchmark() -> bool {
        false
    }

    fn write_to_file_for_benchmark(self, _path: &Path) {
        unreachable!()
    }

    fn load_from_file_for_benchmark(_path: &Path) -> Self {
        unreachable!()
    }

    fn needs_texts() -> bool {
        true
    }

    // only temporary, until awry's issue is fixed
    fn supports_locate_for_benchmark() -> bool {
        true
    }

    fn construct_or_load_for_benchmark(config: &Config) -> Self {
        let index_filepath = config.index_filepath();

        let start = std::time::Instant::now();
        let index = if config.skip_build
            && std::fs::exists(&index_filepath).unwrap()
            && Self::supports_file_io_for_benchmark()
        {
            Self::load_from_file_for_benchmark(&index_filepath)
        } else {
            let texts = if Self::needs_texts() {
                Some(read_texts(&config))
            } else {
                None
            };

            Self::construct_for_benchmark(&config, texts)
        };

        print_after_build_metrics(start);

        index
    }

    fn run_search_benchmark(&self, config: &Config) {
        let queries = read_queries(&config);

        let mut total_num_hits = 0;
        let mut running_times_secs = Vec::new();

        let stub = self.as_stub_for_benchmark();

        for _ in 0..config.repeat_search {
            let start = std::time::Instant::now();
            total_num_hits = 0;

            for query in &queries {
                total_num_hits += match config.search_mode {
                    SearchMode::Count => Self::count_for_benchmark(&stub, &query),
                    SearchMode::Locate => Self::count_via_locate_for_benchmark(&stub, &query),
                };
            }

            running_times_secs.push(start.elapsed().as_millis() as f64 / 1_000.0);
        }

        let min_time = running_times_secs
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .unwrap();
        let avg_time = running_times_secs.iter().sum::<f64>() / config.repeat_search as f64;

        info!(
            "Search queries time: {min_time:.2} (min), {avg_time:.2} (avg) seconds, total number of hits: {total_num_hits}"
        );
    }

    fn run_io_benchmark(self, config: &Config) {
        let index_filepath = config.index_filepath();

        if !std::fs::exists(&index_filepath).unwrap() || config.force_write_and_load {
            let start = std::time::Instant::now();
            self.write_to_file_for_benchmark(&index_filepath);
            info!(
                "Write to disk time: {:.2} seconds",
                start.elapsed().as_millis() as f64 / 1_000.0
            );

            let start = std::time::Instant::now();
            let index = Self::load_from_file_for_benchmark(&index_filepath);
            let index_stub = Self::as_stub_for_benchmark(&index);

            info!(
                "Load from disk time: {:.2} seconds (dummy: {})",
                start.elapsed().as_millis() as f64 / 1_000.0,
                Self::count_via_locate_for_benchmark(&index_stub, b"ACGT")
            );
        }
    }

    fn run_benchmark(config: &Config) {
        let index = Self::construct_or_load_for_benchmark(&config);

        if config.search_mode == SearchMode::Locate && !Self::supports_locate_for_benchmark() {
            info!(
                "Currently, {} does not support locate.",
                config.library.to_string()
            );
        } else {
            index.run_search_benchmark(&config);
        }

        if Self::supports_file_io_for_benchmark() {
            index.run_io_benchmark(&config);
        }
    }
}
