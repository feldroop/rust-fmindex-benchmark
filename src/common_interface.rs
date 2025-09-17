use log::info;

use crate::{Config, SearchMode};

pub trait BenchmarkFmIndex {
    // TODO, might be a bit difficult with setup for awry, and borrowing for bio and sview
    // fn construct_for_benchmark(config: &Config) -> Self;
    // fn write_to_file_for_benchmark(self, config: &Config) -> Result<()>;
    // fn load_to_from_for_benchmark(config: &Config) -> Self;

    fn count_for_benchmark(&self, query: &[u8]) -> usize;
    fn count_via_locate_for_benchmark(&self, query: &[u8]) -> usize;

    fn run_search_benchmark(&self, config: &Config, queries: &[Vec<u8>]) {
        let mut total_num_hits = 0;
        let mut running_times_secs = Vec::new();

        for _ in 0..config.repeat_search {
            let start = std::time::Instant::now();
            total_num_hits = 0;

            for query in queries {
                total_num_hits += match config.search_mode {
                    SearchMode::Count => self.count_for_benchmark(&query),
                    SearchMode::Locate => self.count_via_locate_for_benchmark(&query),
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
}
