mod awry_bench;
mod bio_bench;
mod fmindex_bench;
mod genedex_bench;
mod lt_and_sview_fmindex_bench;

use clap::{Parser, ValueEnum};
use genedex::block::{Block64, Block512};
use log::info;
use std::path::PathBuf;

#[derive(Debug, Parser, Clone)]
struct Args {
    library: Library,

    #[arg(short, long, default_value = "data/hg38.fna")]
    input_texts_path: PathBuf,

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

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum Library {
    GenedexI32B64,
    GenedexI64B64,
    GenedexI32B512,
    GenedexI64B512,
    Bio,
    Awry,
    FmIndex,
    SviewFmIndex32,
    SviewFmIndex128,
    LtFmIndex32,
    LtFmIndex128,
}

impl ToString for Library {
    fn to_string(&self) -> String {
        match self {
            Library::GenedexI32B64 => "genedex_i32_b64",
            Library::GenedexI64B64 => "genedex_i64_b64",
            Library::GenedexI32B512 => "genedex_i32_b512",
            Library::GenedexI64B512 => "genedex_i64_b512",
            Library::Bio => "bio",
            Library::Awry => "awry",
            Library::FmIndex => "fmindex",
            Library::SviewFmIndex32 => "sview_fmindex32",
            Library::SviewFmIndex128 => "sview_fmindex128",
            Library::LtFmIndex32 => "lt_fmindex32",
            Library::LtFmIndex128 => "lt_fmindex128",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
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

fn main() {
    let args = Args::parse();

    setup_logger(args.library).unwrap();

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.build_thread_count as usize)
        .build_global()
        .unwrap();

    info!(
        "------------------------------ starting benchmark for {} ------------------------------",
        args.library.to_string(),
    );

    if args.verbose {
        info!("Configuration: {:#?}", args);
    }

    match args.library {
        Library::GenedexI32B64 => genedex_bench::genedex::<i32, Block64>(args),
        Library::GenedexI64B64 => genedex_bench::genedex::<i64, Block64>(args),
        Library::GenedexI32B512 => genedex_bench::genedex::<i32, Block64>(args),
        Library::GenedexI64B512 => genedex_bench::genedex::<i64, Block512>(args),
        Library::Bio => bio_bench::bio(args),
        Library::Awry => awry_bench::awry(args),
        Library::FmIndex => fmindex_bench::fmindex(args),
        Library::SviewFmIndex32 => lt_and_sview_fmindex_bench::sview_fmindex::<u32>(args),
        Library::SviewFmIndex128 => lt_and_sview_fmindex_bench::sview_fmindex::<u128>(args),
        Library::LtFmIndex32 => lt_and_sview_fmindex_bench::lt_fmindex::<u32>(args),
        Library::LtFmIndex128 => lt_and_sview_fmindex_bench::lt_fmindex::<u128>(args),
    }
}

fn read_texts(args: &Args) -> Vec<Vec<u8>> {
    let start = std::time::Instant::now();

    let reader = bio::io::fasta::Reader::from_file(&args.input_texts_path).unwrap();
    let mut seqs = Vec::new();

    for (i, record) in reader.records().enumerate() {
        if let Some(n) = args.num_text_records {
            if i == n {
                break;
            }
        }

        seqs.push(record.unwrap().seq().to_vec());
    }

    transfrom_seqs(&mut seqs, "texts", b'N', args.verbose);

    if args.verbose {
        info!(
            "Texts reading time: {:.2} seconds",
            start.elapsed().as_millis() as f64 / 1_000.0
        );
    }

    seqs
}

fn read_queries(args: &Args) -> Vec<Vec<u8>> {
    let start = std::time::Instant::now();

    let reader = bio::io::fastq::Reader::from_file(&args.queries_path).unwrap();
    let mut seqs = Vec::new();

    for (i, record) in reader.records().enumerate() {
        if let Some(n) = args.num_queries_records {
            if i == n {
                break;
            }
        }

        let record = record.unwrap();
        let mut slice = record.seq();
        if let Some(l) = args.length_of_queries {
            slice = &slice[..std::cmp::min(l, slice.len())];
        }
        seqs.push(slice.to_vec());
    }

    transfrom_seqs(&mut seqs, "queries", b'A', args.verbose);

    if args.verbose {
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
