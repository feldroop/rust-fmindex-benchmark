use crate::{Args, SearchMode, print_after_build_metrics, read_queries, read_texts};
use log::info;
use std::path::PathBuf;
use std::str::FromStr;

// I don't fully understand the requirements for a temporary suffix array file.
// `&String` as input for search functions is not idiomatic Rust. I believe it should be `&[u8]` in this case.
// Applies many smart tricks. Writes diagnostics to stdout, which libraries usually don't do.
pub fn awry(args: Args) {
    use awry::{alphabet, fm_index};

    let input_texts_path = PathBuf::from(format!(
        "data/input_texts_{}_records.fasta",
        args.num_text_records
            .map_or("all".to_string(), |n| n.to_string())
    ));

    // prepare the input data file. this is only necessary because of the benchmark setup.
    // (I want to sometimes build the index only for a part of the data set)
    if !std::fs::exists(&input_texts_path).unwrap() {
        let texts = read_texts(&args);
        let slice = args.num_text_records.map_or(texts.as_slice(), |l| {
            &texts[..std::cmp::min(l, texts.len())]
        });

        let mut writer = bio::io::fasta::Writer::to_file(&input_texts_path).unwrap();

        for (i, text) in slice.iter().enumerate() {
            writer.write(&i.to_string(), None, text).unwrap();
        }
    }

    let index_filepath = PathBuf::from(format!(
        "indices/{}_sampling_rate_{}_lookup_depth_{}_query_len_{}_text_records_{}.awry",
        args.library.to_string(),
        args.suffix_array_sampling_rate,
        args.depth_of_lookup_table,
        args.length_of_queries
            .map_or("full".to_string(), |l| l.to_string()),
        args.num_text_records
            .map_or_else(|| "all".to_string(), |n| n.to_string())
    ));

    let build_args = fm_index::FmBuildArgs {
        input_file_src: input_texts_path,
        suffix_array_output_src: Some(
            PathBuf::from_str("indices/awry_temporary_suffix_array_output.txt").unwrap(),
        ),
        suffix_array_compression_ratio: Some(args.suffix_array_sampling_rate as u64),
        lookup_table_kmer_len: Some(args.depth_of_lookup_table as u8),
        alphabet: alphabet::SymbolAlphabet::Nucleotide,
        max_query_len: args.length_of_queries,
        remove_intermediate_suffix_array_file: true,
    };

    let start = std::time::Instant::now();
    let index = if args.skip_build {
        fm_index::FmIndex::load(&index_filepath).unwrap()
    } else {
        fm_index::FmIndex::new(&build_args).unwrap()
    };

    print_after_build_metrics(start);

    let queries = read_queries(&args);

    let start = std::time::Instant::now();
    let mut total_num_hits = 0;

    for query in queries {
        let query = String::from_utf8(query).unwrap();

        total_num_hits += match args.search_mode {
            SearchMode::Count => index.count_string(&query) as usize,
            SearchMode::Locate => index.locate_string(&query).len(),
        };
    }

    info!(
        "Search queries time: {} seconds, total number of hits: {total_num_hits}",
        start.elapsed().as_secs()
    );

    if !std::fs::exists(&index_filepath).unwrap() || args.force_write_and_load {
        let start = std::time::Instant::now();
        index.save(&index_filepath).unwrap();
        info!("Write to disk time: {} seconds", start.elapsed().as_secs());

        let start = std::time::Instant::now();
        let index = fm_index::FmIndex::load(&index_filepath).unwrap();
        info!(
            "Load from disk time: {} seconds (dummy: {})",
            start.elapsed().as_secs(),
            index.count_string(&String::from("ACGT"))
        );
    }
}
