use std::marker::PhantomData;
use std::path::Path;

use crate::Config;
use crate::common_interface::BenchmarkFmIndex;

use sview_fmindex::blocks::Block3;
use sview_fmindex::build_config::{LookupTableConfig, SuffixArrayConfig};
use sview_fmindex::text_encoders::EncodingTable;
use sview_fmindex::{Position, blocks::Vector};

// Based on [`lt-fm-index`], but improved memory usage during construction and after.
// The `mmap` support is a nice idea, but probably only relevant for few applications.
pub struct SViewFMIndex<P, V> {
    blob: Vec<u8>,
    _position_marker: PhantomData<P>,
    _vector_marker: PhantomData<V>,
}

impl<P: Position + 'static, V: Vector + 'static> BenchmarkFmIndex for SViewFMIndex<P, V> {
    type Stub<'a> = sview_fmindex::FmIndex<'a, P, Block3<V>, EncodingTable>;

    fn construct_for_benchmark(config: &Config, texts: Option<Vec<Vec<u8>>>) -> Self {
        let symbols: &[&[u8]] = &[b"A", b"C", b"G", b"T", b"N"];
        let encoding_table = EncodingTable::from_symbols(symbols);
        let symbol_count = encoding_table.symbol_count();

        let text: Vec<_> = texts.unwrap().into_iter().flatten().collect();

        let builder = sview_fmindex::FmIndexBuilder::<P, Block3<V>, EncodingTable>::new(
            text.len(),
            symbol_count,
            encoding_table,
        )
        .unwrap()
        .set_lookup_table_config(LookupTableConfig::KmerSize(
            config.depth_of_lookup_table as u32,
        ))
        .unwrap()
        .set_suffix_array_config(SuffixArrayConfig::Compressed(
            config.suffix_array_sampling_rate as u32,
        ))
        .unwrap();

        let blob_size = builder.blob_size();
        let mut blob = vec![0; blob_size];
        builder.build(text, &mut blob).unwrap();

        SViewFMIndex {
            blob,
            _position_marker: PhantomData,
            _vector_marker: PhantomData,
        }
    }

    fn supports_file_io_for_benchmark() -> bool {
        true
    }

    fn write_to_file_for_benchmark(self, path: &Path) {
        std::fs::write(path, self.blob).unwrap();
    }

    fn load_from_file_for_benchmark(path: &Path) -> Self {
        let blob = std::fs::read(path).unwrap();
        SViewFMIndex {
            blob,
            _position_marker: PhantomData,
            _vector_marker: PhantomData,
        }
    }

    fn as_stub_for_benchmark<'a>(&'a self) -> Self::Stub<'a> {
        Self::Stub::load(&self.blob).unwrap()
    }

    fn count_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        index.count(&query).as_usize()
    }

    fn count_via_locate_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize {
        index.locate(&query).len()
    }
}
