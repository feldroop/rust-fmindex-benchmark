use crate::{Args, SearchMode, print_after_build_metrics, read_queries, read_texts};
use log::info;
use std::fs::File;

// serialization needs to be tested, I'm unsure about rank transformation

// Large package of many algorithms and data structures. The API is the most complicated one,
// because individual parts of the index must be constructed by hand. No multitext support.
pub fn bio(args: Args) {
    use bio::alphabets;
    use bio::data_structures::bwt;
    use bio::data_structures::fmindex::{BackwardSearchResult, FMIndex, FMIndexable};
    use bio::data_structures::suffix_array::{self, SampledSuffixArray, SuffixArray};

    let index_filepath = format!(
        "indices/{}_sampling_rate_{}_text_records_{}.bincode",
        args.library.to_string(),
        args.suffix_array_sampling_rate,
        args.num_text_records
            .map_or_else(|| "all".to_string(), |n| n.to_string())
    );

    let mut text: Vec<_> = read_texts(&args).into_iter().flatten().collect();
    text.push(b'$');

    let start = std::time::Instant::now();

    let alphabet = alphabets::Alphabet::new(b"$ACGTN");

    // let rank_transform = alphabets::RankTransform::new(&alphabet);
    // text = rank_transform.transform(text);

    // let rank_alphabet = alphabets::Alphabet::new([0, 1, 2, 3, 4, 5]);

    let suffix_array = suffix_array::suffix_array(&text);
    let bwt = bwt::bwt(&text, &suffix_array);
    let less = bwt::less(&bwt, &alphabet);
    let occ = bwt::Occ::new(
        &bwt,
        (args.suffix_array_sampling_rate * 6) as u32,
        &alphabet,
    );

    let sampled_suffix_array =
        suffix_array.sample(&text, &bwt, &less, &occ, args.suffix_array_sampling_rate);
    drop(suffix_array);
    drop(text);

    let index = FMIndex::new(&bwt, &less, &occ);

    print_after_build_metrics(start);

    let queries = read_queries(&args);

    let start = std::time::Instant::now();
    let mut total_num_hits = 0;

    for query in queries {
        // let query = rank_transform.transform(query);

        total_num_hits += match args.search_mode {
            SearchMode::Count => match index.backward_search(query.iter()) {
                BackwardSearchResult::Complete(interval) => interval.upper - interval.lower,
                BackwardSearchResult::Partial(..) => 0,
                BackwardSearchResult::Absent => 0,
            },
            SearchMode::Locate => match index.backward_search(query.iter()) {
                BackwardSearchResult::Complete(interval) => {
                    interval.occ(&sampled_suffix_array).len()
                }
                BackwardSearchResult::Partial(..) => 0,
                BackwardSearchResult::Absent => 0,
            },
        };
    }

    info!(
        "Search queries time: {} seconds, total number of hits: {total_num_hits}",
        start.elapsed().as_secs()
    );

    if !std::fs::exists(&index_filepath).unwrap() || args.force_write_and_load {
        let start = std::time::Instant::now();
        let mut file = File::create(&index_filepath).unwrap();
        let config = bincode::config::standard().with_fixed_int_encoding();

        bincode::serde::encode_into_std_write(sampled_suffix_array, &mut file, config).unwrap();
        drop(occ);
        drop(less);
        drop(bwt);
        info!("Write to disk time: {} seconds", start.elapsed().as_secs());

        let start = std::time::Instant::now();

        let mut file = File::open(&index_filepath).unwrap();
        let sampled_suffix_array: SampledSuffixArray<Vec<u8>, Vec<usize>, bwt::Occ> =
            bincode::serde::decode_from_std_read(&mut file, config).unwrap();
        let index = FMIndex::new(
            sampled_suffix_array.bwt(),
            sampled_suffix_array.less(),
            sampled_suffix_array.occ(),
        );

        let count = match index.backward_search([1, 2, 3, 4].iter()) {
            BackwardSearchResult::Complete(interval) => interval.upper - interval.lower,
            BackwardSearchResult::Partial(..) => 0,
            BackwardSearchResult::Absent => 0,
        };

        info!(
            "Load from disk time: {} seconds (dummy: {})",
            start.elapsed().as_secs(),
            count
        );
    }
}
