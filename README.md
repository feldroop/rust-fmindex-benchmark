# Rust FM-Index Benchmark

Input text: Human genome [hg38](https://www.ncbi.nlm.nih.gov/datasets/genome/GCF_000001405.38/), first 10 records (~250 MB), first 30 records (~2 GB) or full (~3.3 GB).
Input queries: Randomly selected sample of (quite short) Illumina [reads from SRA](https://www.ncbi.nlm.nih.gov/sra/ERX14765811), queries truncated to length 30, 50, or not truncated.

## Existing Libraries

### [`awry`]

I don't fully understand the requirements for a temporary suffix array file. `&String` as input for search functions is not idiomatic Rust. I believe it should be `&[u8]` in this case. Applies many smart tricks. Writes diagnostics to stdout, which libraries usually don't do.

### [`fm-index`]

No support for reading/writing from/to files. But otherwise nice library with multiple different FM-Index variants. Memory usage during construction is a huge issue.

### [`genedex`]

I am the author, it's in an early state and some features are missing. Look at README for list of issues.

### [`rust-bio`]

Large package of many algorithms and data structures. The API is the most complicated one, because individual parts of the index must be constructed by hand. No multitext support. 

### [`lt-fm-index`]

No multitext support. Large index and problematic construction memory usage. Excluded from benchmark, because [`sview-fmindex`] seems to be the successor version.

### [`sview-fmindex`]

Based on [`lt-fm-index`], but improved memory usage during construction and after. The `mmap` support is a nice idea, but probably only relevant for few applications.

## Feature Comparison



[`awry`]: https://github.com/UM-Applied-Algorithms-Lab/AWRY
[`fm-index`]: https://github.com/ajalab/fm-index
[`genedex`]: https://github.com/feldroop/genedex
[`rust-bio`]: https://github.com/rust-bio/rust-bio
[`lt-fm-index`]: https://github.com/baku4/lt-fm-index/
[`sview-fmindex`]: https://github.com/baku4/sview-fmindex