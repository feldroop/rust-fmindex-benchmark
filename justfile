# comment out first line on unix systems
set shell := ["powershell.exe", "-c"]

input_texts := "-i chromosome"
suffix_array_sampling_rate := "-s 4"
depth_of_lookup_table := "-d 10"
num_queries_records := ""
length_of_queries := "-l 50"
search_mode := "-o locate"
skip_build := ""
force_write_and_load := ""
verbose := "-v"
threads := "-t 8"

run_all:
    # cargo run --release -- awry {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    # cargo run --release -- bio {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    # cargo run --release -- fm-index {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    cargo run --release -- genedex-i32-flat64 {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    # cargo run --release -- genedex-u32-flat64 {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    # cargo run --release -- genedex-i64-flat64 {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    # cargo run --release -- genedex-i32-cond512 {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    # cargo run --release -- genedex-u32-cond512 {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    # cargo run --release -- genedex-i64-cond512 {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    cargo run --release -- sview-fm-index-u32-vec32 {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    # cargo run --release -- sview-fm-index-u32-vec128 {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    # cargo run --release -- sview-fm-index-u64-vec32 {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
    # cargo run --release -- sview-fm-index-u64-vec128 {{input_texts}} {{suffix_array_sampling_rate}} {{depth_of_lookup_table}} {{num_queries_records}} {{length_of_queries}} {{search_mode}} {{skip_build}} {{force_write_and_load}} {{verbose}} {{threads}}
