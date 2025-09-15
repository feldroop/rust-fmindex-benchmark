use crate::{Args, SearchMode, print_after_build_metrics, read_queries, read_texts};
use log::info;
use std::fs::File;

// Based on [`lt-fm-index`], but improved memory usage during construction and after.
// The `mmap` support is a nice idea, but probably only relevant for few applications.
pub fn sview_fmindex<V: sview_fmindex::blocks::Vector>(args: Args) {
    use sview_fmindex::build_config::{LookupTableConfig, SuffixArrayConfig};

    let index_filepath = format!(
        "indices/{}_sampling_rate_{}_lookup_depth_{}_text_records_{}.sview_fmindex",
        args.library.to_string(),
        args.suffix_array_sampling_rate,
        args.depth_of_lookup_table,
        args.num_text_records
            .map_or_else(|| "all".to_string(), |n| n.to_string())
    );

    use sview_fmindex::blocks::Block3;
    use sview_fmindex::text_encoders::EncodingTable;

    let start = std::time::Instant::now();

    let symbols: &[&[u8]] = &[b"A", b"C", b"G", b"T", b"N"];
    let encoding_table = EncodingTable::from_symbols(symbols);
    let symbol_count = encoding_table.symbol_count();

    let text: Vec<_> = read_texts(&args).into_iter().flatten().collect();
    let blob = if args.skip_build {
        std::fs::read(&index_filepath).unwrap()
    } else {
        let builder = sview_fmindex::FmIndexBuilder::<u32, Block3<V>, EncodingTable>::new(
            text.len(),
            symbol_count,
            encoding_table,
        )
        .unwrap()
        .set_lookup_table_config(LookupTableConfig::KmerSize(
            args.depth_of_lookup_table as u32,
        ))
        .unwrap()
        .set_suffix_array_config(SuffixArrayConfig::Compressed(
            args.suffix_array_sampling_rate as u32,
        ))
        .unwrap();

        let blob_size = builder.blob_size();
        let mut blob = vec![0; blob_size];
        builder.build(text, &mut blob).unwrap();
        blob
    };

    let index = sview_fmindex::FmIndex::<u32, Block3<V>, EncodingTable>::load(&blob[..]).unwrap();
    print_after_build_metrics(start);

    let queries = read_queries(&args);

    let start = std::time::Instant::now();
    let mut total_num_hits = 0;

    for query in queries {
        total_num_hits += match args.search_mode {
            SearchMode::Count => index.count(&query) as usize,
            SearchMode::Locate => index.locate(&query).len(),
        };
    }

    info!(
        "Search queries time: {:.2} seconds, total number of hits: {total_num_hits}",
        start.elapsed().as_millis() as f64 / 1_000.0
    );

    if !std::fs::exists(&index_filepath).unwrap() || args.force_write_and_load {
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

// No multitext support. Large index and problematic construction memory usage.
// Excluded from benchmark, because [`sview-fmindex`] seems to be the successor version.
pub fn lt_fmindex<V: lt_fm_index::blocks::Vector>(args: Args) {
    use lt_fm_index::blocks::Block3;

    let index_filepath = format!(
        "indices/{}_sampling_rate_{}_lookup_depth_{}_text_records_{}.lt_fmindex",
        args.library.to_string(),
        args.suffix_array_sampling_rate,
        args.depth_of_lookup_table,
        args.num_text_records
            .map_or_else(|| "all".to_string(), |n| n.to_string())
    );

    let text: Vec<_> = read_texts(&args).into_iter().flatten().collect();

    let characters_by_index: &[&[u8]] = &[b"A", b"C", b"G", b"T", b"N"];

    let start = std::time::Instant::now();
    let index = if args.skip_build {
        let file = File::open(&index_filepath).unwrap();
        lt_fm_index::LtFmIndex::<u32, Block3<V>>::load_from(file).unwrap()
    } else {
        lt_fm_index::LtFmIndex::<u32, Block3<V>>::build(
            text,
            characters_by_index,
            args.suffix_array_sampling_rate as u32,
            args.depth_of_lookup_table as u32,
        )
        .unwrap()
    };
    print_after_build_metrics(start);

    let queries = read_queries(&args);

    let start = std::time::Instant::now();
    let mut total_num_hits = 0;

    for query in queries {
        total_num_hits += match args.search_mode {
            SearchMode::Count => index.count(&query) as usize,
            SearchMode::Locate => index.locate(&query).len(),
        };
    }

    info!(
        "Search queries time: {:.2} seconds, total number of hits: {total_num_hits}",
        start.elapsed().as_millis() as f64 / 1_000.0
    );

    if !std::fs::exists(&index_filepath).unwrap() || args.force_write_and_load {
        let start = std::time::Instant::now();
        let file = File::create(&index_filepath).unwrap();
        index.save_to(file).unwrap();
        info!(
            "Write to disk time: {:.2} seconds",
            start.elapsed().as_millis() as f64 / 1_000.0
        );

        let start = std::time::Instant::now();
        let file = File::open(&index_filepath).unwrap();
        let index = lt_fm_index::LtFmIndex::<u32, Block3<V>>::load_from(file).unwrap();
        info!(
            "Load from disk time: {:.2} seconds (dummy: {})",
            start.elapsed().as_millis() as f64 / 1_000.0,
            index.count(b"ACGT")
        );
    }
}
