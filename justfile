# comment out first line on unix systems
set shell := ["powershell.exe", "-c"]

suffix_array_sampling_rate := "-s 4 "
depth_of_lookup_table := "-d 10 "
num_queries_records := ""
length_of_queries := "-l 50 "
search_mode := "-o locate "
skip_build := "--skip-build"
force_write_and_load := ""
verbose := ""
threads := "-t 8"

args := suffix_array_sampling_rate + depth_of_lookup_table + num_queries_records + length_of_queries + search_mode + skip_build + force_write_and_load + verbose

run_all:
    cargo run --release -- genedex-i32-flat64 -i chromosome {{threads}} {{args}}
    cargo run --release -- genedex-i32-cond512 -i chromosome {{threads}} {{args}}

    # cargo run --release -- awry -i i32 {{threads}} {{args}}
    # cargo run --release -- bio -i i32 {{threads}} {{args}}
    # cargo run --release -- fm-index -i i32 {{threads}} {{args}}
    # cargo run --release -- genedex-i32-flat64 -i i32 {{args}}
    # cargo run --release -- genedex-i32-flat64 -i i32 {{threads}} {{args}}
    # cargo run --release -- genedex-i32-cond512 -i i32 {{args}}
    # cargo run --release -- genedex-i32-cond512 -i i32 {{threads}} {{args}}
    # cargo run --release -- sview-fm-index-u32-vec32 -i i32 {{threads}} {{args}}
    # cargo run --release -- sview-fm-index-u32-vec128 -i i32 {{threads}} {{args}}

    # cargo run --release -- genedex-u32-flat64 -i hg38 {{threads}} {{args}}
    # cargo run --release -- genedex-u32-cond512 -i hg38 {{threads}} {{args}}
    # cargo run --release -- sview-fm-index-u32-vec32 -i hg38 {{threads}} {{args}}
    # cargo run --release -- sview-fm-index-u32-vec128 -i hg38 {{threads}} {{args}}

    # cargo run --release -- genedex-i64-flat64 -i double-hg38 {{threads}} {{args}}
    # cargo run --release -- genedex-i64-cond512 -i double-hg38 {{threads}} {{args}}
    # cargo run --release -- sview-fm-index-u64-vec32 -i double-hg38 {{threads}} {{args}}
    # cargo run --release -- sview-fm-index-u64-vec128 -i double-hg38 {{threads}} {{args}}
