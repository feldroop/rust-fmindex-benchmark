set shell := ["powershell.exe", "-c"]

num_text_records := "-n 10"
suffix_array_sampling_rate := "-s 4"
depth_of_lookup_table := "-d 10"
num_queries_records := "-m 0"
length_of_queries := "-l 30"
search_mode := "-o locate"
skip_build := ""
force_write_and_load := ""

run_all:
    cargo run --release -- awry {{num_text_records}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}}
    cargo run --release -- bio {{num_text_records}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}}
    cargo run --release -- fm-index {{num_text_records}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}}
    cargo run --release -- genedex-i32b64 {{num_text_records}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}}
    cargo run --release -- genedex-i32b64 -t 8 {{num_text_records}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}}
    cargo run --release -- genedex-i64b64 -t 8 {{num_text_records}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}}
    cargo run --release -- sview-fm-index128 {{num_text_records}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}}