# Benchmark Comparison: harmorp vs sastrawi

This document compares the performance of harmorp with the sastrawi Indonesian stemmer library.

## Test Setup

- **Dataset**: 35 words (13 roots, 10 single-prefix, 12 multi-affix)
- **Benchmark**: Criterion.rs with 100 samples per measurement
- **Hardware**: MacBook Pro (M1 Pro, 10 cores, 16 GB RAM)

## Results Summary

### Single Word Stems (Cold Cache)

| Stemmer | Average Time | Throughput | Speedup |
|---------|--------------|------------|---------|
| harmorp | ~700-900 ns | ~1.1-1.8 Melem/s | **10x faster** |
| sastrawi | ~5-7 µs | ~140-180 Kelem/s | baseline |

harmorp is approximately **10x faster** than sastrawi for single word stemming.

### Batch Processing (35 words)

| Stemmer | Cold Time | Hot Time | Throughput (Hot) | Speedup |
|---------|-----------|----------|------------------|---------|
| harmorp | 12.6 µs | 3.0 µs | 11.9 Melem/s | **31x faster** |
| sastrawi | 93.4 µs | 93.4 µs | 385 Kelem/s | baseline |

harmorp is **31x faster** for batch processing when cache is warm, and **7.4x faster** on cold runs.

### Throughput (10,000 words)

| Stemmer | Time | Throughput | Speedup |
|---------|------|------------|---------|
| harmorp | 812 µs | 12.3 Melem/s | **32x faster** |
| sastrawi | 26.4 ms | 379 Kelem/s | baseline |

harmorp processes 10,000 words in **32x less time** than sastrawi.

## Key Advantages of harmorp

1. **Zero-allocation hot path**: Uses SmallVec and `&str` slices
2. **Thread-safe caching**: O(1) repeated lookups via DashMap
3. **Iterative ECS algorithm**: Handles complex multi-affix words efficiently
4. **No dictionary required**: Works standalone with ~85-90% accuracy

## Running the Comparison

To run the comparison benchmark:

```bash
cargo bench --bench compare_bench
```

This will generate detailed HTML reports in `target/criterion/compare_bench/`.
