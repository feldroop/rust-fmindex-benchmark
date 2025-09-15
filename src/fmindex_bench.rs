use crate::{Args, SearchMode, print_after_build_metrics, read_queries, read_texts};
use log::info;

// Supports multiple different FM-Index variants.
pub fn fmindex(args: Args) {
    use fm_index::Search;

    let text: Vec<_> = read_texts(&args)
        .into_iter()
        .map(|mut t| {
            t.push(0);
            t
        })
        .flatten()
        .collect();
    let text = fm_index::Text::new(text);

    assert!(args.suffix_array_sampling_rate.is_power_of_two());
    let sampling_level = args.suffix_array_sampling_rate.ilog2() as usize;

    let start = std::time::Instant::now();
    let index = fm_index::FMIndexMultiPiecesWithLocate::new(&text, sampling_level).unwrap();
    // let index = fm_index::FMIndexWithLocate::new(&text, sampling_level).unwrap(); <-- didn't improve memory usage
    print_after_build_metrics(start);

    let queries = read_queries(&args);

    let start = std::time::Instant::now();
    let mut total_num_hits = 0;

    for query in queries {
        total_num_hits += match args.search_mode {
            SearchMode::Count => index.search(&query).count(),
            SearchMode::Locate => index.search(&query).iter_matches().count(),
        };
    }

    info!(
        "Search queries time: {:.2} seconds, total number of hits: {total_num_hits}",
        start.elapsed().as_millis() as f64 / 1_000.0
    );
}
