mod awry_bench;
mod bio_bench;
mod common_interface;
mod fmindex_bench;
mod genedex_bench;
mod sview_fmindex_bench;

use clap::{Parser, ValueEnum};
use log::info;
use std::{collections::HashMap, convert::identity, fs::File, path::PathBuf};

use crate::common_interface::{BenchmarkFmIndex, SearchMetrics};

#[derive(serde::Serialize, serde::Deserialize, Debug, Parser, Clone, PartialEq, Eq)]
struct Config {
    library: Library,

    #[arg(short, long)]
    input_texts: InputTexts,

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

    #[arg(short, long, default_value_t = 5)]
    repeat_search: usize,

    #[arg(long)]
    skip_build: bool,

    #[arg(short, long)]
    force_write_and_load: bool,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
struct SearchConfig {
    search_mode: SearchMode,
    num_queries_records: Option<usize>,
    length_of_queries: Option<usize>,
}

impl Config {
    fn index_filepath(&self) -> PathBuf {
        PathBuf::from(format!(
            "indices/{}_sampling_rate_{}_lookup_depth_{}_text_records_{}.index",
            self.library.to_string(),
            self.suffix_array_sampling_rate,
            self.depth_of_lookup_table,
            self.input_texts.to_string(),
        ))
    }

    fn search_config(&self) -> SearchConfig {
        SearchConfig {
            search_mode: self.search_mode,
            num_queries_records: self.num_queries_records,
            length_of_queries: self.num_queries_records,
        }
    }

    fn has_same_index_config_as(&self, other: &Config) -> bool {
        self.build_thread_count == other.build_thread_count
            && self.depth_of_lookup_table == other.depth_of_lookup_table
            && self.library == other.library
            && self.suffix_array_sampling_rate == other.suffix_array_sampling_rate
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
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

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Hash,
)]
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

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Hash,
)]
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

#[derive(serde::Serialize, serde::Deserialize)]
struct BenchmarkResult {
    config: Config,

    // only set when the build was not skipped
    construction_time_secs: Option<f64>,
    construction_peak_memory_usage_mb: Option<f64>,

    // only set when loading from disk to make sure that remaining from texts and build allocations don't count
    only_index_in_memory_size_mb: Option<f64>,

    search_metrics: HashMap<SearchConfig, SearchMetrics>,

    // only set when file IO is available and was not skipped
    write_to_file_time_secs: Option<f64>,
    read_from_file_time_secs: Option<f64>,
}

impl BenchmarkResult {
    fn new_empty(config: Config) -> Self {
        Self {
            config,
            construction_time_secs: None,
            construction_peak_memory_usage_mb: None,
            only_index_in_memory_size_mb: None,
            search_metrics: HashMap::new(),
            write_to_file_time_secs: None,
            read_from_file_time_secs: None,
        }
    }

    fn update(&mut self, other: Self) {
        assert!(self.config.has_same_index_config_as(&other.config));

        self.construction_time_secs = other.construction_time_secs.or(self.construction_time_secs);
        self.construction_peak_memory_usage_mb = other
            .construction_peak_memory_usage_mb
            .or(self.construction_peak_memory_usage_mb);
        self.only_index_in_memory_size_mb = other
            .only_index_in_memory_size_mb
            .or(self.only_index_in_memory_size_mb);
        self.write_to_file_time_secs = other
            .write_to_file_time_secs
            .or(self.write_to_file_time_secs);
        self.read_from_file_time_secs = other
            .read_from_file_time_secs
            .or(self.read_from_file_time_secs);

        for (search_config, search_metrics) in other.search_metrics.into_iter() {
            self.search_metrics.insert(search_config, search_metrics);
        }
    }
}

// input genome should be placed at data/hg38
fn main() {
    let config = Config::parse();

    for dir_name in ["data", "indices", "logs", "results"] {
        if !std::fs::exists(dir_name).unwrap() {
            std::fs::create_dir(dir_name).unwrap();
        }
    }

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

    let result = match config.library {
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
    };

    update_stored_results(result, config);
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

fn setup_logger(library: Library) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, _| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(format!("logs/{}.txt", library.to_string()))?)
        .apply()?;
    Ok(())
}

fn update_stored_results(result: BenchmarkResult, config: Config) {
    let results_filepath = format!("results/{}.ron", config.input_texts.to_string());

    let mut results: HashMap<(Library, u16), BenchmarkResult> =
        if std::fs::exists(&results_filepath).unwrap() {
            let file = File::open(&results_filepath).unwrap();
            ron::de::from_reader(file).unwrap()
        } else {
            HashMap::new()
        };

    let existing_result = results
        .entry((config.library, config.build_thread_count))
        .or_insert_with(|| BenchmarkResult::new_empty(config));

    existing_result.update(result);

    let serialized = ron::ser::to_string_pretty(&results, Default::default()).unwrap();
    std::fs::write(results_filepath, serialized).unwrap();
}
