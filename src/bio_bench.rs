use crate::Config;
use crate::common_interface::BenchmarkFmIndex;

use std::fs::File;
use std::path::Path;

use bio::alphabets;
use bio::data_structures::bwt::{self, Occ};
use bio::data_structures::fmindex::{BackwardSearchResult, FMIndex, FMIndexable};
use bio::data_structures::suffix_array::{self, SampledSuffixArray, SuffixArray};

// Large package of many algorithms and data structures. The API is the most complicated one,
// because individual parts of the index must be constructed by hand. No multitext support.
pub struct BioFmIndex<const R: usize> {
    inner: SampledSuffixArray<Vec<u8>, Vec<usize>, Occ>,
}

impl<const R: usize> BenchmarkFmIndex for BioFmIndex<R> {
    type Stub<'a> = &'a SampledSuffixArray<Vec<u8>, Vec<usize>, Occ>;

    fn construct_for_benchmark(config: &Config, texts: Option<Vec<Vec<u8>>>) -> Self {
        let mut text: Vec<_> = texts.unwrap().into_iter().flatten().collect();
        text.push(b'$');

        let alphabet = alphabets::Alphabet::new(b"$ACGTN");

        // I'm unsure about rank transformation

        // let rank_transform = alphabets::RankTransform::new(&alphabet);
        // text = rank_transform.transform(text);

        // let rank_alphabet = alphabets::Alphabet::new([0, 1, 2, 3, 4, 5]);

        let occ_sampling_rate = (R * config.suffix_array_sampling_rate * 6) as u32;
        let suffix_array = suffix_array::suffix_array(&text);
        let bwt = bwt::bwt(&text, &suffix_array);
        let less = bwt::less(&bwt, &alphabet);
        let occ = Occ::new(&bwt, occ_sampling_rate, &alphabet);

        Self {
            inner: suffix_array.sample(&text, bwt, less, occ, config.suffix_array_sampling_rate),
        }
    }

    fn supports_file_io_for_benchmark() -> bool {
        true
    }

    fn write_to_file_for_benchmark(self, path: &Path) {
        let mut file = File::create(path).unwrap();
        let config = bincode::config::standard().with_fixed_int_encoding();

        bincode::serde::encode_into_std_write(self.inner, &mut file, config).unwrap();
    }

    fn load_from_file_for_benchmark(path: &Path) -> Self {
        let mut file = File::open(path).unwrap();
        let config = bincode::config::standard().with_fixed_int_encoding();

        Self {
            inner: bincode::serde::decode_from_std_read(&mut file, config).unwrap(),
        }
    }

    fn as_stub_for_benchmark<'a>(&'a self) -> Self::Stub<'a> {
        &self.inner
    }

    fn count_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        // loading index stub is essentially no-op
        let index_stub = FMIndex::new(index.bwt(), index.less(), index.occ());

        // let query = rank_transform.transform(query);
        match index_stub.backward_search(query.iter()) {
            BackwardSearchResult::Complete(interval) => interval.upper - interval.lower,
            BackwardSearchResult::Partial(..) => 0,
            BackwardSearchResult::Absent => 0,
        }
    }

    fn count_via_locate_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        // loading index stub is essentially  no-op
        let index_stub = FMIndex::new(index.bwt(), index.less(), index.occ());

        // let query = rank_transform.transform(query);
        match index_stub.backward_search(query.iter()) {
            BackwardSearchResult::Complete(interval) => interval.occ(*index).len(),
            BackwardSearchResult::Partial(..) => 0,
            BackwardSearchResult::Absent => 0,
        }
    }
}
