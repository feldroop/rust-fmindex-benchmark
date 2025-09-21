use std::path::Path;

use log::info;

use crate::{BenchmarkResult, Config, SearchMode};

pub trait BenchmarkFmIndex: Sized {
    // this interface is a bit complicated, because the sview fmindex is essentially a reference to a slice and also is not Copy
    type Stub<'a>
    where
        Self: 'a;

    fn as_stub_for_benchmark<'a>(&'a self) -> Self::Stub<'a>;

    fn construct_for_benchmark(config: &Config, texts: Option<Vec<Vec<u8>>>) -> Self;

    fn count_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize;

    fn count_via_locate_for_benchmark<'a>(index: &Self::Stub<'a>, query: &[u8]) -> usize;

    fn supports_file_io_for_benchmark() -> bool {
        false
    }

    fn write_to_file_for_benchmark(self, _path: &Path) {
        unreachable!()
    }

    fn load_from_file_for_benchmark(_path: &Path) -> Self {
        unreachable!()
    }

    fn needs_texts() -> bool {
        true
    }

    // only temporary, until awry's issue is fixed
    fn supports_locate_for_benchmark() -> bool {
        true
    }

    fn construct_or_load_for_benchmark(config: &Config) -> (Self, ConstructionMetrics) {
        let index_filepath = config.index_filepath();

        let start = std::time::Instant::now();
        let (index, was_constructed) = if config.skip_build
            && std::fs::exists(&index_filepath).unwrap()
            && Self::supports_file_io_for_benchmark()
        {
            (Self::load_from_file_for_benchmark(&index_filepath), false)
        } else {
            let texts = if Self::needs_texts() {
                Some(read_texts(config))
            } else {
                None
            };

            (Self::construct_for_benchmark(config, texts), true)
        };

        let metrics = collect_and_log_after_build_metrics(start, was_constructed);

        (index, metrics)
    }

    fn run_search_benchmark(&self, config: &Config) -> SearchMetrics {
        let queries = read_queries(config);

        let mut total_num_hits = 0;
        let mut running_times_secs = Vec::new();

        let stub = self.as_stub_for_benchmark();

        for _ in 0..config.repeat_search {
            let start = std::time::Instant::now();
            total_num_hits = 0;

            for query in &queries {
                total_num_hits += match config.search_mode {
                    SearchMode::Count => Self::count_for_benchmark(&stub, query),
                    SearchMode::Locate => Self::count_via_locate_for_benchmark(&stub, query),
                };
            }

            running_times_secs.push(start.elapsed().as_millis() as f64 / 1_000.0);
        }

        let &min_time_secs = running_times_secs
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .unwrap();
        let avg_time_secs = running_times_secs.iter().sum::<f64>() / config.repeat_search as f64;

        info!(
            "Search queries time: {min_time_secs:.2} (min), {avg_time_secs:.2} (avg) seconds, total number of hits: {total_num_hits}"
        );

        SearchMetrics {
            min_time_secs,
            avg_time_secs,
        }
    }

    fn run_io_benchmark(self, config: &Config) -> Option<FileIoMetrics> {
        let index_filepath = config.index_filepath();

        if !std::fs::exists(&index_filepath).unwrap() || config.force_write_and_load {
            let start = std::time::Instant::now();
            self.write_to_file_for_benchmark(&index_filepath);
            let write_secs = start.elapsed().as_millis() as f64 / 1_000.0;
            info!("Write to disk time: {write_secs:.2} seconds");

            let start = std::time::Instant::now();
            let index = Self::load_from_file_for_benchmark(&index_filepath);
            let index_stub = Self::as_stub_for_benchmark(&index);
            let read_secs = start.elapsed().as_millis() as f64 / 1_000.0;

            info!(
                "Load from disk time: {read_secs:.2} seconds (dummy: {})",
                Self::count_via_locate_for_benchmark(&index_stub, b"ACGT")
            );

            Some(FileIoMetrics {
                read_secs,
                write_secs,
            })
        } else {
            None
        }
    }

    fn run_benchmark(config: &Config) -> BenchmarkResult {
        let mut result = BenchmarkResult::new_empty(config.clone());

        let (index, construction_metrics) = Self::construct_or_load_for_benchmark(config);

        if construction_metrics.was_constructed {
            result.construction_peak_memory_usage_mb =
                Some(construction_metrics.peak_memory_usage_mb);
            result.construction_time_secs = Some(construction_metrics.elapsed_time_secs);
        }

        result.only_index_in_memory_size_mb = Some(construction_metrics.curr_memory_usage_mb);

        if config.search_mode == SearchMode::Locate && !Self::supports_locate_for_benchmark() {
            info!("Currently, {} does not support locate.", config.library);
        } else {
            let search_metrics = index.run_search_benchmark(config);

            result
                .search_metrics
                .insert(config.search_config().to_string(), search_metrics);
        }

        if Self::supports_file_io_for_benchmark() {
            let file_io_metrics = index.run_io_benchmark(config);

            result.read_from_file_time_secs = file_io_metrics.map(|m| m.read_secs);
            result.write_to_file_time_secs = file_io_metrics.map(|m| m.write_secs);
        }

        result
    }
}

#[derive(Clone, Copy)]
pub struct ConstructionMetrics {
    elapsed_time_secs: f64,
    peak_memory_usage_mb: f64,
    curr_memory_usage_mb: f64,
    was_constructed: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct SearchMetrics {
    min_time_secs: f64,
    avg_time_secs: f64,
}

#[derive(Clone, Copy)]
pub struct FileIoMetrics {
    read_secs: f64,
    write_secs: f64,
}

fn read_texts(config: &Config) -> Vec<Vec<u8>> {
    let start = std::time::Instant::now();

    let reader = bio::io::fasta::Reader::from_file(config.input_texts.get_filepath()).unwrap();
    let mut seqs: Vec<_> = reader
        .records()
        .map(|r| r.unwrap().seq().to_vec())
        .collect();

    transfrom_seqs(&mut seqs, "texts", b'N', config.verbose);

    if config.verbose {
        info!(
            "Texts reading time: {:.2} seconds",
            start.elapsed().as_millis() as f64 / 1_000.0
        );
    }

    seqs
}

fn read_queries(config: &Config) -> Vec<Vec<u8>> {
    let start = std::time::Instant::now();

    let reader = bio::io::fastq::Reader::from_file(&config.queries_path).unwrap();
    let mut seqs = Vec::new();

    for (i, record) in reader.records().enumerate() {
        if let Some(n) = config.num_queries_records
            && i == n
        {
            break;
        }

        let record = record.unwrap();
        let mut slice = record.seq();
        if let Some(l) = config.length_of_queries {
            slice = &slice[..std::cmp::min(l, slice.len())];
        }
        seqs.push(slice.to_vec());
    }

    transfrom_seqs(&mut seqs, "queries", b'A', config.verbose);

    if config.verbose {
        info!(
            "Queries reading time: {:.2} seconds",
            start.elapsed().as_millis() as f64 / 1_000.0
        );
    }

    seqs
}

fn transfrom_seqs(seqs: &mut [Vec<u8>], name: &str, replacement_symbol: u8, verbose: bool) {
    let mut translation_table: Vec<_> = (0u8..=255).collect();
    for degenerate_symbol in b"rRyYkKMmSsWwBbDdHhVvNn".iter().copied() {
        translation_table[degenerate_symbol as usize] = replacement_symbol;
    }
    translation_table[b'a' as usize] = b'A';
    translation_table[b'c' as usize] = b'C';
    translation_table[b'g' as usize] = b'G';
    translation_table[b't' as usize] = b'T';

    for seq in seqs.iter_mut() {
        for symbol in seq.iter_mut() {
            *symbol = translation_table[*symbol as usize];
        }
    }

    let texts_len: usize = seqs.iter().map(|t| t.len()).sum();

    let average_record_length =
        seqs.iter().map(|s| s.len()).sum::<usize>() as f64 / seqs.len() as f64;

    if verbose {
        info!(
            "Total length of {name}: {} MB, average record length: {:.1}",
            texts_len / 1_000_000,
            average_record_length
        );
    }

    info!(
        "Current memory usage after reading {name}: {:.1} MB",
        process_current_memory_usage_mb()
    );
}

fn collect_and_log_after_build_metrics(
    start: std::time::Instant,
    was_constructed: bool,
) -> ConstructionMetrics {
    let elapsed_time_secs = start.elapsed().as_millis() as f64 / 1_000.0;
    let peak_memory_usage_mb = process_peak_memory_usage_mb();
    let curr_memory_usage_mb = process_current_memory_usage_mb();

    info!("Build/load time: {elapsed_time_secs:.2} seconds",);

    info!("Peak memory usage after building/loading: {peak_memory_usage_mb:.1} MB",);
    info!("Current memory usage after building/loading: {curr_memory_usage_mb:.1} MB",);

    ConstructionMetrics {
        elapsed_time_secs,
        peak_memory_usage_mb,
        curr_memory_usage_mb,
        was_constructed,
    }
}

// ---------- just for fun, I implemented the memory usage functionaliy by hand ----------
#[cfg(windows)]
fn get_memory_info() -> windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS {
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::GetCurrentProcess;

    let handle = unsafe { GetCurrentProcess() };
    let mut memory_info = PROCESS_MEMORY_COUNTERS::default();
    let ptr: *mut PROCESS_MEMORY_COUNTERS = &mut memory_info;
    // safety: standard usage of this windows API, I think it should be safe
    unsafe {
        GetProcessMemoryInfo(handle, ptr, std::mem::size_of_val(&memory_info) as u32).unwrap()
    };
    memory_info
}

#[cfg(windows)]
fn process_peak_memory_usage_mb() -> f64 {
    get_memory_info().PeakWorkingSetSize as f64 / 1_000_000.0
}

#[cfg(windows)]
fn process_current_memory_usage_mb() -> f64 {
    get_memory_info().WorkingSetSize as f64 / 1_000_000.0
}

#[cfg(unix)]
fn process_peak_memory_usage_mb() -> f64 {
    let mut memory_info: libc::rusage = unsafe { std::mem::zeroed() };
    let ret =
        unsafe { libc::getrusage(libc::RUSAGE_SELF, (&mut memory_info) as *mut libc::rusage) };
    assert!(ret == 0);

    memory_info.ru_maxrss as f64 / 1_000.0
}

#[cfg(unix)]
fn process_current_memory_usage_mb() -> f64 {
    let statm = std::fs::read_to_string("/proc/self/statm").unwrap();
    let fields: Vec<&str> = statm.split_whitespace().collect();

    let num_pages = fields[0].parse::<u64>().unwrap();
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
    (num_pages * page_size) as f64 / 1_000_000.0
}
