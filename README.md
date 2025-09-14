# Rust FM-Index Benchmark

## Existing Libraries

Libraries were gathered from [crates.io](crates.io) searches. The library [`lt-fm-index`] was excluded, because it seems to be a predecessor of [`sview-fmindex`]. The analysis was done to the best of my ability. If I made a mistake in using/analyzing any of the libraries, please let me know. **Bias alert:** I am the author of [`genedex`].

### Feature Comparison

- **Good construction memory usage:** FM-Indices always need a lot of memory during construction, mainly because a suffix array has to be constructed for the text. However, 6 or 10 times the memory usage of the input text should be sufficient (text + BWT + 32/64 bit suffix array). The numbers displayed were measured for the construction of the index for the smallest input text (see below, 10 records, 250 MB).
- **Multiple texts:** The library directly supports indexing multiple texts, such as a genome with multiple chromosomes.
- **Disk I/O:** The library supports writing the index to disk and restoring it after. The warning sign is used to indicate that the library supports it, but it is very slow (usually due to the usage of slow serializers). [`rust-bio`] was used with one of the fastest `serde` (de)serializer libraries, [`bincode`]. I tried multiple (de)serializer libraries, and none of them was fast. Most of them seem to be optimized for small space usage.
- **Multithreaded construction:** Not the most important feature, but nice to have. Even for [`genedex`], the scaling of the parallelization is far from optimal.

| **Library** | **Good construction memory usage** | **Multiple texts** |  **Disk I/O** | **Multithreaded construction** | 
| ----------- | :-------------: | :-------------: | :-------------: | :-------------: | 
| [`awry`]          | ❌ (18x) | ✅ | ⚠️ | ✅ (unsure) |
| [`fm-index`]      | ❌ (34x) | ✅ | ❌ | ❌ |
| [`genedex`]       | ✅ (6x/10x) | ✅ | ✅ | ✅ |
| [`rust-bio`]      | ❌ (25x) | ❌  | ⚠️ | ❌ |
| [`sview-fmindex`] | ❌ (18x) | ❌ | ✅ | ❌ |

## Benchmark Setup

**Input text:** Human genome [hg38](https://www.ncbi.nlm.nih.gov/datasets/genome/GCF_000001405.38/), first 10 records (~250 MB), first 30 records (~2 GB). In the future, I want to also run the benchmark for the full genome (~3.3 GB).

**Input queries:** Randomly selected sample of (quite short) Illumina [reads from SRA](https://www.ncbi.nlm.nih.gov/sra/ERX14765811), queries truncated to length 30, 50, or not truncated.

**Benchmark Platform:** Windows 11 laptop with an AMD Ryzen 7 PRO 8840HS (octa-core) processor and 32 GB of RAM.

**Task:** The library has to build the FM-Index and then search queries, either by only counting occurrences, or by also locating them. If the library supports it, the index is written to disk and then read back into memory.

## Benchmark Results

Coming soon!

[`awry`]: https://github.com/UM-Applied-Algorithms-Lab/AWRY
[`bincode`]: https://sr.ht/~stygianentity/bincode/
[`fm-index`]: https://github.com/ajalab/fm-index
[`genedex`]: https://github.com/feldroop/genedex
[`rust-bio`]: https://github.com/rust-bio/rust-bio
[`lt-fm-index`]: https://github.com/baku4/lt-fm-index/
[`sview-fmindex`]: https://github.com/baku4/sview-fmindex
