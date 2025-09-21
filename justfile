# comment out first line on unix systems
set shell := ["powershell.exe", "-c"]

suffix_array_sampling_rate := "-s 4 "
depth_of_lookup_table := "-d 10 "
num_queries_records := ""
length_of_queries := "-l 50 "
search_mode := ""
verbose := ""

args := suffix_array_sampling_rate + depth_of_lookup_table + num_queries_records + length_of_queries + search_mode + verbose

run-all mode="twice":
    # just {{mode}} genedex-flat64 -i chromosome {{args}} -t 8
    # just {{mode}} genedex-cond512 -i chromosome {{args}} -t 8
    # just {{mode}} genedex-cond512 -i chromosome {{args}} -t 8 -e low-memory

    # just {{mode}} bio -i i32 {{args}}
    # just {{mode}} fm-index -i i32 {{args}}
    just {{mode}} genedex-flat64 -i i32 {{args}}
    # just {{mode}} genedex-flat64 -i i32 {{args}} -t 8
    # just {{mode}} genedex-cond512 -i i32 {{args}}
    # just {{mode}} genedex-cond512 -i i32 {{args}} -t 8
    # just {{mode}} genedex-cond512 -i i32 {{args}} -t 8 -e low-memory
    # just {{mode}} sview-fm-index-u32-vec32 -i i32 {{args}}
    # just {{mode}} sview-fm-index-u32-vec128 -i i32 {{args}}

    # just {{mode}} bio -i hg38 {{args}}
    # just {{mode}} fm-index -i hg38 {{args}}
    # just {{mode}} genedex-flat64 -i hg38 {{args}}
    # just {{mode}} genedex-flat64 -i hg38 {{args}} -t 8
    # just {{mode}} genedex-cond512 -i hg38 {{args}}
    # just {{mode}} genedex-cond512 -i hg38 {{args}} -t 8
    # just {{mode}} genedex-cond512 -i hg38 {{args}} -t 8 -e low-memory
    # just {{mode}} sview-fm-index-vec32 -i hg38 {{args}}
    # just {{mode}} sview-fm-index-vec128 -i hg38 {{args}}

    # just {{mode}} genedex-flat64 -i double-hg38 {{args}} -t 8
    # just {{mode}} genedex-cond512 -i double-hg38 {{args}} -t 8
    # just {{mode}} genedex-cond512 -i double-hg38 {{args}} -t 8 -e low-memory
    # just {{mode}} sview-fm-index-vec32 -i double-hg38 {{args}}
    # just {{mode}} sview-fm-index-vec128 -i double-hg38 {{args}}
    
    # excluded for now due to segmentation fault issues
    # just {{mode}} awry -i i32 {{args}}

twice +ARGS:
    cargo run --release -- {{ARGS}} -f
    cargo run --release -- {{ARGS}} --skip-build

no-build +ARGS:
    cargo run --release -- {{ARGS}} --skip-build
