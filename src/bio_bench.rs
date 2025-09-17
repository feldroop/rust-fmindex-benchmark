use crate::common_interface::BenchmarkFmIndex;
use crate::{Config, print_after_build_metrics, read_queries, read_texts};
use log::info;
use std::fs::File;

use bio::alphabets;
use bio::data_structures::bwt::{self, Occ};
use bio::data_structures::fmindex::{BackwardSearchResult, FMIndex, FMIndexable};
use bio::data_structures::suffix_array::{self, SampledSuffixArray, SuffixArray};

pub type BioFmIndex<'a> = (
    FMIndex<&'a Vec<u8>, &'a Vec<usize>, &'a Occ>,
    SampledSuffixArray<&'a Vec<u8>, &'a Vec<usize>, &'a Occ>,
);

impl<'a> BenchmarkFmIndex for BioFmIndex<'a> {
    fn count_for_benchmark(&self, query: &[u8]) -> usize {
        // let query = rank_transform.transform(query);
        match self.0.backward_search(query.iter()) {
            BackwardSearchResult::Complete(interval) => interval.upper - interval.lower,
            BackwardSearchResult::Partial(..) => 0,
            BackwardSearchResult::Absent => 0,
        }
    }

    fn count_via_locate_for_benchmark(&self, query: &[u8]) -> usize {
        // let query = rank_transform.transform(query);
        match self.0.backward_search(query.iter()) {
            BackwardSearchResult::Complete(interval) => interval.occ(&self.1).len(),
            BackwardSearchResult::Partial(..) => 0,
            BackwardSearchResult::Absent => 0,
        }
    }
}

// I'm unsure about rank transformation

// Large package of many algorithms and data structures. The API is the most complicated one,
// because individual parts of the index must be constructed by hand. No multitext support.
pub fn bio(config: Config) {
    let index_filepath = config.index_filepath();

    let mut text: Vec<_> = read_texts(&config).into_iter().flatten().collect();
    text.push(b'$');

    let start = std::time::Instant::now();

    let alphabet = alphabets::Alphabet::new(b"$ACGTN");

    // let rank_transform = alphabets::RankTransform::new(&alphabet);
    // text = rank_transform.transform(text);

    // let rank_alphabet = alphabets::Alphabet::new([0, 1, 2, 3, 4, 5]);

    let suffix_array = suffix_array::suffix_array(&text);
    let bwt = bwt::bwt(&text, &suffix_array);
    let less = bwt::less(&bwt, &alphabet);
    let occ = Occ::new(
        &bwt,
        (config.suffix_array_sampling_rate * 6) as u32,
        &alphabet,
    );

    let sampled_suffix_array =
        suffix_array.sample(&text, &bwt, &less, &occ, config.suffix_array_sampling_rate);
    drop(suffix_array);
    drop(text);

    let index = FMIndex::new(&bwt, &less, &occ);

    print_after_build_metrics(start);

    let queries = read_queries(&config);

    let tup = (index, sampled_suffix_array);
    tup.run_search_benchmark(&config, &queries);

    let (_, sampled_suffix_array) = tup;

    if !std::fs::exists(&index_filepath).unwrap() || config.force_write_and_load {
        let start = std::time::Instant::now();
        let mut file = File::create(&index_filepath).unwrap();
        let config = bincode::config::standard().with_fixed_int_encoding();

        bincode::serde::encode_into_std_write(sampled_suffix_array, &mut file, config).unwrap();
        drop(occ);
        drop(less);
        drop(bwt);
        info!(
            "Write to disk time: {:.2} seconds",
            start.elapsed().as_millis() as f64 / 1_000.0
        );

        let start = std::time::Instant::now();

        let mut file = File::open(&index_filepath).unwrap();
        let sampled_suffix_array: SampledSuffixArray<Vec<u8>, Vec<usize>, bwt::Occ> =
            bincode::serde::decode_from_std_read(&mut file, config).unwrap();
        let index = FMIndex::new(
            sampled_suffix_array.bwt(),
            sampled_suffix_array.less(),
            sampled_suffix_array.occ(),
        );

        let count = match index.backward_search(b"ACGT".iter()) {
            BackwardSearchResult::Complete(interval) => interval.upper - interval.lower,
            BackwardSearchResult::Partial(..) => 0,
            BackwardSearchResult::Absent => 0,
        };

        info!(
            "Load from disk time: {:.2} seconds (dummy: {})",
            start.elapsed().as_millis() as f64 / 1_000.0,
            count
        );
    }
}
