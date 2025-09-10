use log::info;
use std::path::Path;

use crate::{GenedexFmIndex, Library, current_process_peak_memory_usage_mb};
use genedex::text_with_rank_support::{Block, Block64, Block512};

pub fn build_and_write_index(
    texts_path: &Path,
    library: Library,
    thread_count: u16,
    suffix_array_sampling_rate: usize,
    num_records: Option<usize>,
) {
    info!(
        "Configuration: {thread_count} threads, {suffix_array_sampling_rate} sampling rate, {} records",
        num_records.map_or("all".to_string(), |n| n.to_string())
    );

    let texts = read_texts(texts_path, num_records);

    let texts_len: usize = texts.iter().map(|t| t.len()).sum();

    info!("Total length of texts: {} MB", texts_len / 1_000_000);

    let prebuild_peak_memory_usage = current_process_peak_memory_usage_mb();
    info!(
        "Peak memory usage before build start: {:.1} MB",
        prebuild_peak_memory_usage
    );

    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count as usize)
        .build_global()
        .unwrap();

    match library {
        Library::Genedex64 => genedex_build_and_write_index::<Block64>(
            texts,
            thread_count,
            suffix_array_sampling_rate,
            library.name(),
        ),
        Library::Genedex512 => genedex_build_and_write_index::<Block512>(
            texts,
            thread_count,
            suffix_array_sampling_rate,
            library.name(),
        ),
    }

    let after_build_peak_memory_usage = current_process_peak_memory_usage_mb(); // - prebuild_peak_memory_usage;

    info!(
        "Build peak memory usage: {:.1} MB",
        after_build_peak_memory_usage
    );
}

pub fn read_texts(path: &Path, num_records: Option<usize>) -> Vec<Vec<u8>> {
    let reader = bio::io::fasta::Reader::from_file(path).unwrap();
    let mut texts = Vec::new();

    for (i, record) in reader.records().enumerate() {
        if let Some(n) = num_records {
            if i == n {
                break;
            }
        }

        texts.push(record.unwrap().seq().to_vec());
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

pub fn genedex_build_and_write_index<
    B: Block + savefile::Packed + savefile::Serialize + 'static,
>(
    texts: Vec<Vec<u8>>,
    thread_count: u16,
    suffix_array_sampling_rate: usize,
    name: &str,
) {
    let start = std::time::Instant::now();
    let index =
        GenedexFmIndex::<B>::new_u32_compressed(texts, thread_count, suffix_array_sampling_rate);
    info!("Build time: {} seconds", start.elapsed().as_secs());

    let start = std::time::Instant::now();

    savefile::save_file(
        &format!("indices/{name}_sampling_rate{suffix_array_sampling_rate}.savefile"),
        0,
        &index,
    )
    .unwrap();

    info!("Write to disk time: {} seconds", start.elapsed().as_secs());
}
