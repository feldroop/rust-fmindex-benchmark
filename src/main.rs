mod awry_bench;
mod bio_bench;
mod common_interface;
mod fmindex_bench;
mod genedex_bench;
mod sview_fmindex_bench;

use crate::common_interface::{BenchmarkFmIndex, SearchMetrics};
use clap::{Parser, ValueEnum};
use log::info;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::identity, fmt::Display, fs::File, path::PathBuf};
use strum::Display;

#[derive(Serialize, Deserialize, Debug, Parser, Clone, PartialEq, Eq)]
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

    #[arg(short, long)]
    extra_build_arg: Option<ExtraBuildArg>,

    #[arg(short, long, default_value = "data/reads.fastq")]
    queries_path: PathBuf,

    #[arg(short = 'm', long)]
    num_queries_records: Option<usize>,

    #[arg(short, long)]
    length_of_queries: Option<usize>,

    #[arg(short = 'o', long, default_value = "locate")]
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Display)]
enum ExtraBuildArg {
    LowMemory,
    MediumMemory,
}

impl Config {
    fn index_filepath(&self) -> PathBuf {
        PathBuf::from(format!(
            "indices/{}_sampling_rate_{}_lookup_depth_{}_text_records_{}.index",
            self.library,
            self.suffix_array_sampling_rate,
            self.depth_of_lookup_table,
            self.input_texts,
        ))
    }

    fn search_config(&self) -> SearchConfig {
        SearchConfig {
            search_mode: self.search_mode,
            num_queries_records: self.num_queries_records,
            length_of_queries: self.length_of_queries,
        }
    }

    fn has_same_index_config_as(&self, other: &Config) -> bool {
        self.build_thread_count == other.build_thread_count
            && self.depth_of_lookup_table == other.depth_of_lookup_table
            && self.library == other.library
            && self.suffix_array_sampling_rate == other.suffix_array_sampling_rate
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
struct SearchConfig {
    search_mode: SearchMode,
    num_queries_records: Option<usize>,
    length_of_queries: Option<usize>,
}

impl Display for SearchConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{}",
            self.search_mode,
            self.num_queries_records
                .map_or_else(|| String::from("all"), |n| n.to_string()),
            self.length_of_queries
                .map_or_else(|| String::from("full"), |l| l.to_string())
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Display)]
enum InputTexts {
    Chromosome,
    I32,
    Hg38,
    DoubleHg38,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Hash, Display)]
enum Library {
    GenedexFlat64,
    GenedexCond512,
    BioSmall,
    BioLarge,
    Awry,
    FmIndexSingle,
    FmIndexMulti,
    SviewFmIndexVec32,
    SviewFmIndexVec128,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Hash, Display)]
enum SearchMode {
    Count,
    Locate,
}

#[derive(Serialize, Deserialize)]
struct BenchmarkResult {
    config: Config,

    // only set when the build was not skipped
    construction_time_secs: Option<f64>,
    construction_peak_memory_usage_mb: Option<f64>,

    // only set when loading from disk to make sure that remaining from texts and build allocations don't count
    only_index_in_memory_size_mb: Option<f64>,

    search_metrics: HashMap<String, SearchMetrics>,

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
        config.library,
    );

    if config.verbose {
        info!("Configuration: {:#?}", config);
    }

    let result = match config.input_texts {
        InputTexts::Chromosome => run_benchmark_for_index_type::<i32, u32>(&config),
        InputTexts::I32 => run_benchmark_for_index_type::<i32, u32>(&config),
        InputTexts::Hg38 => run_benchmark_for_index_type::<u32, u32>(&config),
        InputTexts::DoubleHg38 => run_benchmark_for_index_type::<i64, u64>(&config),
    };

    update_stored_results(result, config);
}

fn run_benchmark_for_index_type<G: genedex::IndexStorage, S: sview_fmindex::Position + 'static>(
    config: &Config,
) -> BenchmarkResult {
    match config.library {
        Library::GenedexFlat64 => genedex::FmIndexFlat64::<G>::run_benchmark(config),
        Library::GenedexCond512 => genedex::FmIndexCondensed512::<G>::run_benchmark(config),
        Library::BioLarge => bio_bench::BioFmIndex::<1>::run_benchmark(config),
        Library::BioSmall => bio_bench::BioFmIndex::<2>::run_benchmark(config),
        Library::Awry => awry_bench::AwryFmIndex::run_benchmark(config),
        Library::FmIndexSingle => fmindex_bench::FMIndexCrateSingleFmIndex::run_benchmark(config),
        Library::FmIndexMulti => fmindex_bench::FMIndexCrateMultiFmIndex::run_benchmark(config),
        Library::SviewFmIndexVec32 => {
            sview_fmindex_bench::SViewFMIndex::<S, u32>::run_benchmark(config)
        }
        Library::SviewFmIndexVec128 => {
            sview_fmindex_bench::SViewFMIndex::<S, u128>::run_benchmark(config)
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

    let reader = bio::io::fasta::Reader::from_file(path_hg38).unwrap();

    let mut chromosome_writer = bio::io::fasta::Writer::to_file(path_chromosome).unwrap();
    let mut i32_writer = bio::io::fasta::Writer::to_file(path_i32).unwrap();
    let mut double_hg38_writer = bio::io::fasta::Writer::to_file(path_double_hg38).unwrap();

    for (i, record) in reader.records().enumerate() {
        let record = record.unwrap();

        if i < chromosome_num_records {
            chromosome_writer.write_record(&record).unwrap();
        }

        if i < i32_num_records {
            i32_writer.write_record(&record).unwrap();
        }

        double_hg38_writer.write_record(&record).unwrap();

        let revcomp = bio::alphabets::dna::revcomp(record.seq());

        double_hg38_writer
            .write(record.id(), record.desc(), &revcomp)
            .unwrap();
    }
}

fn setup_logger(library: Library) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, _| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(format!("logs/{}.txt", library))?)
        .apply()?;
    Ok(())
}

fn update_stored_results(result: BenchmarkResult, config: Config) {
    let results_filepath = format!("results/{}.json", config.input_texts);

    let mut results: HashMap<String, BenchmarkResult> =
        if std::fs::exists(&results_filepath).unwrap() {
            let file = File::open(&results_filepath).unwrap();
            serde_json::from_reader(file).unwrap()
        } else {
            HashMap::new()
        };

    let existing_result = results
        .entry(key_to_string(&config))
        .or_insert_with(|| BenchmarkResult::new_empty(config));

    existing_result.update(result);

    let file = File::create(&results_filepath).unwrap();

    serde_json::to_writer_pretty(file, &results).unwrap();
}

fn key_to_string(config: &Config) -> String {
    format!(
        "{}-threads-{}-arg-{}",
        config.library,
        config.build_thread_count,
        config
            .extra_build_arg
            .map_or_else(|| "none".to_string(), |arg| arg.to_string())
    )
}
