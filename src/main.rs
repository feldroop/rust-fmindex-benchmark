mod awry_bench;
mod bio_bench;
mod common_interface;
mod fmindex_bench;
mod genedex_bench;
mod sview_fmindex_bench;

use clap::{Parser, ValueEnum};
use log::info;
use std::{convert::identity, fs::File, path::PathBuf};

use crate::common_interface::BenchmarkFmIndex;

#[derive(Debug, Parser, Clone)]
struct Config {
    library: Library,

    #[arg(short, long)]
    input_texts: InputTexts,

    #[arg(short, long)]
    num_text_records: Option<usize>,

    #[arg(short, long, default_value_t = 4)]
    suffix_array_sampling_rate: usize,

    #[arg(short, long, default_value_t = 13)]
    depth_of_lookup_table: usize,

    #[arg(short = 't', long, default_value_t = 1)]
    build_thread_count: u16,

    #[arg(short, long, default_value = "data/reads.fastq")]
    queries_path: PathBuf,

    #[arg(short = 'm', long)]
    num_queries_records: Option<usize>,

    #[arg(short, long)]
    length_of_queries: Option<usize>,

    #[arg(short = 'o', long, default_value_t = SearchMode::Locate)]
    search_mode: SearchMode,

    #[arg(short, long, default_value_t = 3)]
    repeat_search: usize,

    #[arg(long)]
    skip_build: bool,

    #[arg(short, long)]
    force_write_and_load: bool,

    #[arg(short, long)]
    verbose: bool,
}

impl Config {
    fn index_filepath(&self) -> PathBuf {
        PathBuf::from(format!(
            "indices/{}_sampling_rate_{}_lookup_depth_{}_text_records_{}.{}",
            self.library.to_string(),
            self.suffix_array_sampling_rate,
            self.depth_of_lookup_table,
            self.num_text_records
                .map_or_else(|| "all".to_string(), |n| n.to_string()),
            self.library.to_string()
        ))
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum InputTexts {
    Chromosome,
    I32,
    Hg38,
    DoubleHg38,
}

impl ToString for InputTexts {
    fn to_string(&self) -> String {
        match self {
            InputTexts::Chromosome => "chromosome",
            InputTexts::I32 => "i32",
            InputTexts::Hg38 => "hg38",
            InputTexts::DoubleHg38 => "double-hg38",
        }
        .to_string()
    }
}

impl InputTexts {
    fn get_filepath(&self) -> PathBuf {
        match self {
            InputTexts::Chromosome => PathBuf::from("data/chromosome.fna"),
            InputTexts::I32 => PathBuf::from("data/i32.fna"),
            InputTexts::Hg38 => PathBuf::from("data/hg38.fna"),
            InputTexts::DoubleHg38 => PathBuf::from("data/hg38_double.fna"),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum Library {
    GenedexI32Flat64,
    GenedexU32Flat64,
    GenedexI64Flat64,
    GenedexI32Cond512,
    GenedexU32Cond512,
    GenedexI64Cond512,
    Bio,
    Awry,
    FmIndex,
    SviewFmIndexU32Vec32,
    SviewFmIndexU32Vec128,
    SviewFmIndexU64Vec32,
    SviewFmIndexU64Vec128,
}

impl ToString for Library {
    fn to_string(&self) -> String {
        match self {
            Library::GenedexI32Flat64 => "genedex_i32_flat64",
            Library::GenedexU32Flat64 => "genedex_u32_flat64",
            Library::GenedexI64Flat64 => "genedex_i64_flat64",
            Library::GenedexI32Cond512 => "genedex_i32_cond512",
            Library::GenedexU32Cond512 => "genedex_u32_cond512",
            Library::GenedexI64Cond512 => "genedex_i64_cond512",
            Library::Bio => "bio",
            Library::Awry => "awry",
            Library::FmIndex => "fmindex",
            Library::SviewFmIndexU32Vec32 => "sview_fmindex_u32_vec32",
            Library::SviewFmIndexU32Vec128 => "sview_fmindex_u32_vec128",
            Library::SviewFmIndexU64Vec32 => "sview_fmindex_u64_vec32",
            Library::SviewFmIndexU64Vec128 => "sview_fmindex_u64_vec128",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum SearchMode {
    Count,
    Locate,
}

impl ToString for SearchMode {
    fn to_string(&self) -> String {
        match self {
            SearchMode::Count => "count",
            SearchMode::Locate => "locate",
        }
        .to_string()
    }
}

// input genome should be placed at data/hg38
fn main() {
    let config = Config::parse();

    setup_logger(config.library).unwrap();

    rayon::ThreadPoolBuilder::new()
        .num_threads(config.build_thread_count as usize)
        .build_global()
        .unwrap();

    setup_input_data();

    info!(
        "------------------------------ starting benchmark for {} ------------------------------",
        config.library.to_string(),
    );

    if config.verbose {
        info!("Configuration: {:#?}", config);
    }

    match config.library {
        Library::GenedexI32Flat64 => genedex::FmIndexFlat64::<i32>::run_benchmark(&config),
        Library::GenedexU32Flat64 => genedex::FmIndexFlat64::<u32>::run_benchmark(&config),
        Library::GenedexI64Flat64 => genedex::FmIndexFlat64::<i64>::run_benchmark(&config),
        Library::GenedexI32Cond512 => genedex::FmIndexCondensed512::<i32>::run_benchmark(&config),
        Library::GenedexU32Cond512 => genedex::FmIndexCondensed512::<u32>::run_benchmark(&config),
        Library::GenedexI64Cond512 => genedex::FmIndexCondensed512::<i64>::run_benchmark(&config),
        Library::Bio => bio_bench::BioFmIndex::run_benchmark(&config),
        Library::Awry => awry_bench::AwryFmIndex::run_benchmark(&config),
        Library::FmIndex => fmindex_bench::FMIndexCrateFmIndex::run_benchmark(&config),
        Library::SviewFmIndexU32Vec32 => {
            sview_fmindex_bench::SViewFMIndex::<u32, u32>::run_benchmark(&config)
        }
        Library::SviewFmIndexU32Vec128 => {
            sview_fmindex_bench::SViewFMIndex::<u32, u128>::run_benchmark(&config)
        }
        Library::SviewFmIndexU64Vec32 => {
            sview_fmindex_bench::SViewFMIndex::<u64, u32>::run_benchmark(&config)
        }
        Library::SviewFmIndexU64Vec128 => {
            sview_fmindex_bench::SViewFMIndex::<u64, u128>::run_benchmark(&config)
        }
    }
}

fn setup_input_data() {
    let path_chromosome = InputTexts::Chromosome.get_filepath();
    let path_i32 = InputTexts::I32.get_filepath();
    let path_hg38 = InputTexts::Hg38.get_filepath();
    let path_double_hg38 = InputTexts::DoubleHg38.get_filepath();

    let paths = [&path_chromosome, &path_i32, &path_hg38, &path_double_hg38];

    if paths.map(|p| p.exists()).into_iter().all(identity) {
        return;
    }

    info!("Seems to be the first run in this environment. Preparing the different input files...",);

    let chromosome_num_records = 10;
    let i32_num_records = 30;

    let mut reader = seq_io::fasta::Reader::from_path(path_hg38).unwrap();

    let chromosome_file = File::create(path_chromosome).unwrap();
    let i32_file = File::create(path_i32).unwrap();
    let double_hg38_file = File::create(path_double_hg38).unwrap();

    for (i, record) in reader.records().enumerate() {
        let record = record.unwrap();

        if i < chromosome_num_records {
            seq_io::fasta::write_to(&chromosome_file, &record.head, &record.seq).unwrap();
        }

        if i < i32_num_records {
            seq_io::fasta::write_to(&i32_file, &record.head, &record.seq).unwrap();
        }

        seq_io::fasta::write_to(&double_hg38_file, &record.head, &record.seq).unwrap();

        let revcomp = bio::alphabets::dna::revcomp(&record.seq);

        seq_io::fasta::write_to(&double_hg38_file, &record.head, &revcomp).unwrap();
    }
}

fn read_texts(config: &Config) -> Vec<Vec<u8>> {
    let start = std::time::Instant::now();

    let mut reader = seq_io::fasta::Reader::from_path(&config.input_texts.get_filepath()).unwrap();
    let mut seqs = Vec::new();

    for (i, record) in reader.records().enumerate() {
        if let Some(n) = config.num_text_records {
            if i == n {
                break;
            }
        }

        seqs.push(record.unwrap().seq);
    }

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

    let mut reader = seq_io::fastq::Reader::from_path(&config.queries_path).unwrap();
    let mut seqs = Vec::new();

    for (i, record) in reader.records().enumerate() {
        if let Some(n) = config.num_queries_records {
            if i == n {
                break;
            }
        }

        let mut seq = record.unwrap().seq;
        if let Some(l) = config.length_of_queries {
            seq.truncate(l);
        }
        seqs.push(seq);
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

fn transfrom_seqs(seqs: &mut Vec<Vec<u8>>, name: &str, replacement_symbol: u8, verbose: bool) {
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

fn print_after_build_metrics(start: std::time::Instant) {
    info!(
        "Build/load time: {:.2} seconds",
        start.elapsed().as_millis() as f64 / 1_000.0
    );

    info!(
        "Peak memory usage after building/loading: {:.1} MB",
        process_peak_memory_usage_mb()
    );
    info!(
        "Current memory usage after building/loading: {:.1} MB",
        process_current_memory_usage_mb()
    );
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

fn setup_logger(library: Library) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, _| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(format!("logs/{}.txt", library.to_string()))?)
        .apply()?;
    Ok(())
}
