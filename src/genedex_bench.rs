use crate::{Args, SearchMode, print_after_build_metrics, read_queries, read_texts};
use genedex::text_with_rank_support::Block;
use genedex::{FmIndexConfig, IndexStorage};
use log::info;

pub fn genedex<I: IndexStorage, B: Block>(args: Args) {
    use genedex::{FmIndex, alphabet};

    let index_filepath = format!(
        "indices/{}_sampling_rate_{}_lookup_depth_{}_text_records_{}.savefile",
        args.library.to_string(),
        args.suffix_array_sampling_rate,
        args.depth_of_lookup_table,
        args.num_text_records
            .map_or_else(|| "all".to_string(), |n| n.to_string())
    );

    let start = std::time::Instant::now();
    let index = if args.skip_build {
        FmIndex::<I, B>::load_from_file(&index_filepath).unwrap()
    } else {
        let texts = read_texts(&args);

        FmIndexConfig::<I, B>::new()
            .lookup_table_depth(args.depth_of_lookup_table)
            .suffix_array_sampling_rate(args.suffix_array_sampling_rate)
            .construct(texts, alphabet::ascii_dna_with_n())
    };

    print_after_build_metrics(start);

    let queries = read_queries(&args);

    let start = std::time::Instant::now();
    let mut total_num_hits = 0;

    for query in queries {
        total_num_hits += match args.search_mode {
            SearchMode::Count => index.count(&query),
            SearchMode::Locate => index.locate(&query).count(),
        };
    }

    info!(
        "Search queries time: {:.2} seconds, total number of hits: {total_num_hits}",
        start.elapsed().as_millis() as f64 / 1_000.0
    );

    if !std::fs::exists(&index_filepath).unwrap() || args.force_write_and_load {
        let start = std::time::Instant::now();
        index.save_to_file(&index_filepath).unwrap();
        info!(
            "Write to disk time: {:.2} seconds",
            start.elapsed().as_millis() as f64 / 1_000.0
        );

        let start = std::time::Instant::now();
        let index = FmIndex::<I, B>::load_from_file(&index_filepath).unwrap();
        info!(
            "Load from disk time: {:.2} seconds (dummy: {})",
            start.elapsed().as_millis() as f64 / 1_000.0,
            index.count(b"ACGT")
        );
    }
}
