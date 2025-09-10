mod build;
mod search;

use clap::{Parser, Subcommand, ValueEnum};
use genedex::{FmIndexU32, alphabet::AsciiDnaWithN};
use std::path::PathBuf;
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows::Win32::System::Threading::GetCurrentProcess;

type GenedexFmIndex<B> = FmIndexU32<AsciiDnaWithN, B>;

#[derive(Parser)]
struct Cli {
    library: Library,

    #[arg(short, long, default_value_t = 8)]
    suffix_array_sampling_rate: usize,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Build {
        #[arg(
            short,
            long,
            default_value = "data\\GCF_000001405.40_GRCh38.p14_genomic.fna"
        )]
        input_texts_path: PathBuf,
        #[arg(short, long, default_value_t = 1)]
        thread_count: u16,
        #[arg(long, short)]
        num_records: Option<usize>,
    },
    Search {
        #[arg(short, long, default_value = "data\\ERR15362081.fastq")]
        queries_path: PathBuf,
        #[arg(long, short)]
        num_records: Option<usize>,
        #[arg(long, short)]
        length_of_queries: Option<usize>,
        #[arg(long, short, default_value_t = SearchMode::Locate)]
        mode: SearchMode,
    },
}

impl Command {
    fn name(&self) -> &'static str {
        match self {
            Command::Build { .. } => "build",
            Command::Search { .. } => "search",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Library {
    Genedex64,
    Genedex512,
}

impl Library {
    fn name(&self) -> &'static str {
        match self {
            Library::Genedex64 => "genedex64",
            Library::Genedex512 => "genedex512",
        }
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
            SearchMode::Count => "count".to_string(),
            SearchMode::Locate => "locate".to_string(),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    setup_logger(cli.library.name(), cli.command.name()).unwrap();

    log::info!(
        "-------------------- starting {} benchmark of {} --------------------",
        cli.command.name(),
        cli.library.name()
    );

    match cli.command {
        Command::Build {
            input_texts_path: texts_path,
            thread_count,
            num_records,
        } => build::build_and_write_index(
            &texts_path,
            cli.library,
            thread_count,
            cli.suffix_array_sampling_rate,
            num_records,
        ),
        Command::Search {
            queries_path,
            num_records,
            length_of_queries,
            mode,
        } => search::load_index_and_search_queries(
            &queries_path,
            cli.library,
            cli.suffix_array_sampling_rate,
            num_records,
            length_of_queries,
            mode,
        ),
    }
}

fn current_process_peak_memory_usage_mb() -> f64 {
    let handle = unsafe { GetCurrentProcess() };
    let mut memory_info = PROCESS_MEMORY_COUNTERS::default();
    let ptr: *mut PROCESS_MEMORY_COUNTERS = &mut memory_info;
    unsafe {
        GetProcessMemoryInfo(handle, ptr, std::mem::size_of_val(&memory_info) as u32).unwrap()
    };
    memory_info.PeakWorkingSetSize as f64 / 1_000_000.0
}

fn setup_logger(library_name: &str, command_name: &str) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, _| {
            out.finish(format_args!(
                "[{}] {}",
                humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(format!(
            "logs\\{command_name}\\{library_name}.txt"
        ))?)
        .apply()?;
    Ok(())
}
