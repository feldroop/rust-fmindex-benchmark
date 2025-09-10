use genedex::text_with_rank_support::{Block, Block64, Block512};
use log::info;
use std::path::Path;

use crate::{GenedexFmIndex, Library, SearchMode, current_process_peak_memory_usage_mb};

pub fn load_index_and_search_queries(
    queries_path: &Path,
    library: Library,
    suffix_array_sampling_rate: usize,
    num_records: Option<usize>,
    length_of_queries: Option<usize>,
    mode: SearchMode,
) {
    let queries = read_queries(queries_path, num_records, length_of_queries);
    let average_query_length =
        queries.iter().map(|q| q.len()).sum::<usize>() as f64 / queries.len() as f64;

    info!(
        "Configuration: {} queries, configured query length: {}, average actual query length: {:.1}, mode: {}",
        num_records.map_or(format!("{} (all)", queries.len()), |n| n.to_string()),
        length_of_queries.map_or("full".to_string(), |l| l.to_string()),
        average_query_length,
        mode.to_string()
    );

    let preload_peak_memory_usage = current_process_peak_memory_usage_mb();
    info!(
        "Peak memory usage before loading index: {:.1} MB",
        preload_peak_memory_usage
    );

    let total_num_hits = match library {
        Library::Genedex64 => genedex_load_index_and_search::<Block64>(
            queries,
            length_of_queries,
            library.name(),
            mode,
            suffix_array_sampling_rate,
        ),
        Library::Genedex512 => genedex_load_index_and_search::<Block512>(
            queries,
            length_of_queries,
            library.name(),
            mode,
            suffix_array_sampling_rate,
        ),
    };

    info!("Total number of hits: {}", total_num_hits);

    let after_search_peak_memory_usage = current_process_peak_memory_usage_mb();
    info!(
        "Peak memory usage after search: {:.1} MB",
        after_search_peak_memory_usage
    );
}

pub fn read_queries(
    path: &Path,
    num_records: Option<usize>,
    length_of_queries: Option<usize>,
) -> Vec<Vec<u8>> {
    let reader = bio::io::fastq::Reader::from_file(path).unwrap();
    let mut texts = Vec::new();

    for (i, record) in reader.records().enumerate() {
        if let Some(n) = num_records {
            if i == n {
                break;
            }
        }

        let record = record.unwrap();
        let mut slice = record.seq();
        if let Some(l) = length_of_queries {
            slice = &slice[..std::cmp::min(l, slice.len())];
        }

        texts.push(slice.to_vec());
    }

    let mut translation_table: Vec<_> = (0u8..=255).collect();
    for degenerate_symbol in b"rRyYkKMmSsWwBbDdHhVv".iter().copied() {
        translation_table[degenerate_symbol as usize] = b'N';
    }

    for text in texts.iter_mut() {
        for symbol in text.iter_mut() {
            *symbol = translation_table[*symbol as usize];
        }
    }

    texts
}

pub fn genedex_load_index_and_search<
    B: Block + savefile::Packed + savefile::Deserialize + 'static,
>(
    queries: Vec<Vec<u8>>,
    length_of_queries: Option<usize>,
    name: &str,
    mode: SearchMode,
    suffix_array_sampling_rate: usize,
) -> usize {
    let start = std::time::Instant::now();
    let index: GenedexFmIndex<B> = savefile::load_file(
        &format!("indices/{name}_sampling_rate{suffix_array_sampling_rate}.savefile"),
        0,
    )
    .unwrap();

    info!("Load time: {} seconds", start.elapsed().as_secs());

    let start = std::time::Instant::now();
    let mut total_num_hits = 0;

    for query in queries {
        let cropped_query = length_of_queries.map_or(query.as_slice(), |l| {
            &query[..std::cmp::min(l, query.len())]
        });

        total_num_hits += match mode {
            SearchMode::Count => index.count(cropped_query),
            SearchMode::Locate => index.locate(cropped_query).count(),
        };
    }

    info!("Search queries time: {} seconds", start.elapsed().as_secs());

    total_num_hits
}
