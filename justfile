# comment out first line on unix systems
set shell := ["powershell.exe", "-c"]

suffix_array_sampling_rate := "-s 4 "
depth_of_lookup_table := "-d 10 "
num_queries_records := ""
length_of_queries := "-l 50 "
search_mode := ""
verbose := ""

args := suffix_array_sampling_rate + depth_of_lookup_table + num_queries_records + length_of_queries + search_mode + verbose

run mode="twice":
    # these are mainly for debugging
    just {{mode}} genedex-flat64 -i chromosome {{args}} -t 8
    just {{mode}} genedex-cond64 -i chromosome {{args}} -t 8
    just {{mode}} genedex-cond512 -i chromosome {{args}} -t 8
    just {{mode}} genedex-cond512 -i chromosome {{args}} -t 8 -e medium-memory

    # just {{mode}} bio-small -i i32 {{args}}
    # just {{mode}} bio-large -i i32 {{args}}
    # just {{mode}} fm-index-multi -i i32 {{args}}
    # just {{mode}} genedex-flat64 -i i32 {{args}} -t 8
    # just {{mode}} genedex-cond64 -i i32 {{args}} -t 8
    # just {{mode}} genedex-cond64 -i i32 {{args}}
    # just {{mode}} genedex-cond64 -i i32 {{args}} -t 8 -e medium-memory
    # just {{mode}} genedex-cond512 -i i32 {{args}} -t 8
    # just {{mode}} sview-fm-index-vec32 -i i32 {{args}}
    # just {{mode}} sview-fm-index-vec128 -i i32 {{args}}

    # just {{mode}} bio-small -i hg38 {{args}}
    # just {{mode}} bio-large -i hg38 {{args}}
    # just {{mode}} fm-index-multi -i hg38 {{args}}
    # just {{mode}} genedex-flat64 -i hg38 {{args}} -t 8
    # just {{mode}} genedex-cond64 -i hg38 {{args}}
    # just {{mode}} genedex-cond64 -i hg38 {{args}} -t 8
    # just {{mode}} genedex-cond64 -i hg38 {{args}} -t 8 -e low-memory
    # just {{mode}} genedex-cond64 -i hg38 {{args}} -t 8 -e medium-memory
    # just {{mode}} genedex-cond512 -i hg38 {{args}} -t 8
    # just {{mode}} sview-fm-index-vec32 -i hg38 {{args}}
    # just {{mode}} sview-fm-index-vec128 -i hg38 {{args}}

    # just {{mode}} genedex-flat64 -i double-hg38 {{args}} -t 8
    # just {{mode}} genedex-cond64 -i double-hg38 {{args}} -t 8
    # just {{mode}} genedex-cond64 -i double-hg38 {{args}} -t 8 -e medium-memory
    # just {{mode}} genedex-cond512 -i double-hg38 {{args}} -t 8
    # just {{mode}} sview-fm-index-vec32 -i double-hg38 {{args}}
    # just {{mode}} sview-fm-index-vec128 -i double-hg38 {{args}}

    # excluded because it did not provide a speed or memory benefit
    # just {{mode}} fm-index-single -i i32 {{args}}
    # just {{mode}} fm-index-single -i hg38 {{args}}
    
    # excluded for now due to segmentation fault issues
    # just {{mode}} awry -i i32 {{args}}
    # just {{mode}} awry -i hg38 {{args}}

twice +ARGS:
    cargo run --release -- {{ARGS}} -f
    cargo run --release -- {{ARGS}} --skip-build

with-build +ARGS:
    cargo run --release -- {{ARGS}} -f

no-build +ARGS:
    cargo run --release -- {{ARGS}} --skip-build

no-build-flamegraph +ARGS:
    cargo flamegraph -F 10000 --deterministic --release -- {{ARGS}} --skip-build
