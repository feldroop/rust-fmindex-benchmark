# comment out first line on unix systems
set shell := ["powershell.exe", "-c"]

suffix_array_sampling_rate := "-s 4 "
depth_of_lookup_table := "-d 10 "
num_queries_records := ""
length_of_queries := "-l 50 "
search_mode := "-o locate "
verbose := ""

args := suffix_array_sampling_rate + depth_of_lookup_table + num_queries_records + length_of_queries + search_mode + force_write_and_load + verbose

run-all mode="twice":
    just {{mode}} genedex-i32-flat64 -i chromosome {{args}}
    just {{mode}} genedex-i32-cond512 -i chromosome {{args}}

    # just {{mode}} bio -i i32 {{args}}
    # just {{mode}} fm-index -i i32 {{args}}
    # just {{mode}} genedex-i32-flat64 -t 8 -i i32 {{args}}
    # just {{mode}} genedex-i32-flat64 -i i32 {{args}}
    # just {{mode}} genedex-i32-cond512 -t 8  -i i32 {{args}}
    # just {{mode}} genedex-i32-cond512 -i i32 {{args}}
    # just {{mode}} sview-fm-index-u32-vec32 -i i32 {{args}}
    # just {{mode}} sview-fm-index-u32-vec128 -i i32 {{args}}

    # just {{mode}} genedex-u32-flat64 -t 8 -i hg38 {{args}}
    # just {{mode}} genedex-u32-cond512 -t 8 -i hg38 {{args}}
    # just {{mode}} sview-fm-index-u32-vec32 -i hg38 {{args}}
    # just {{mode}} sview-fm-index-u32-vec128 -i hg38 {{args}}

    # just {{mode}} genedex-i64-flat64 -t 8  -i double-hg38 {{args}}
    # just {{mode}} genedex-i64-cond512 -t 8  -i double-hg38 {{args}}
    # just {{mode}} sview-fm-index-u64-vec32 -i double-hg38 {{args}}
    # just {{mode}} sview-fm-index-u64-vec128 -i double-hg38 {{args}}
    
    # just {{mode}} awry -i i32 {{args}}

twice +ARGS:
    cargo run --release -- {{ARGS}} -f
    cargo run --release -- {{ARGS}} --skip-build

no-build +ARGS:
    cargo run --release -- {{ARGS}} --skip-build
