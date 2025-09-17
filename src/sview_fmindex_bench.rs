use crate::common_interface::BenchmarkFmIndex;
use crate::{Config, print_after_build_metrics, read_queries, read_texts};
use log::info;
use sview_fmindex::blocks::Block3;
use sview_fmindex::build_config::{LookupTableConfig, SuffixArrayConfig};
use sview_fmindex::text_encoders::EncodingTable;
use sview_fmindex::{Position, blocks::Vector};

pub type SViewFMIndex<'a, P, V> = sview_fmindex::FmIndex<'a, P, Block3<V>, EncodingTable>;

impl<'a, P: Position, V: Vector> BenchmarkFmIndex for SViewFMIndex<'a, P, V> {
    fn count_for_benchmark(&self, query: &[u8]) -> usize {
        self.count(&query).as_usize()
    }

    fn count_via_locate_for_benchmark(&self, query: &[u8]) -> usize {
        self.locate(&query).len()
    }
}

// Based on [`lt-fm-index`], but improved memory usage during construction and after.
// The `mmap` support is a nice idea, but probably only relevant for few applications.
pub fn sview_fmindex<V: sview_fmindex::blocks::Vector>(config: Config) {
    let index_filepath = config.index_filepath();

    let start = std::time::Instant::now();

    let symbols: &[&[u8]] = &[b"A", b"C", b"G", b"T", b"N"];
    let encoding_table = EncodingTable::from_symbols(symbols);
    let symbol_count = encoding_table.symbol_count();

    let text: Vec<_> = read_texts(&config).into_iter().flatten().collect();
    let blob = if config.skip_build {
        std::fs::read(&index_filepath).unwrap()
    } else {
        let builder = sview_fmindex::FmIndexBuilder::<u32, Block3<V>, EncodingTable>::new(
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
        blob
    };

    let index = sview_fmindex::FmIndex::<u32, Block3<V>, EncodingTable>::load(&blob[..]).unwrap();
    print_after_build_metrics(start);

    let queries = read_queries(&config);

    index.run_search_benchmark(&config, &queries);

    if !std::fs::exists(&index_filepath).unwrap() || config.force_write_and_load {
        let start = std::time::Instant::now();
        std::fs::write(&index_filepath, blob).unwrap();
        info!(
            "Write to disk time: {:.2} seconds",
            start.elapsed().as_millis() as f64 / 1_000.0
        );

        let start = std::time::Instant::now();
        let blob = std::fs::read(&index_filepath).unwrap();
        let index =
            sview_fmindex::FmIndex::<u32, Block3<V>, EncodingTable>::load(&blob[..]).unwrap();
        info!(
            "Load from disk time: {:.2} seconds (dummy: {})",
            start.elapsed().as_millis() as f64 / 1_000.0,
            index.count(b"ACGT")
        );
    }
}
