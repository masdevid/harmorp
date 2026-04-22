# Performance Documentation

## Benchmarks

Run benchmarks with:

```bash
cargo bench --bench stemmer_bench
```

## Performance Characteristics

| Scenario | Typical Latency | Notes |
|----------|-----------------|-------|
| Cache hit (warm) | ~50 ns | After first stem of a word |
| Single word (cold) | ~1-5 µs | First time seeing a word |
| 10,000-word batch (hot) | ~5 ms | With cache hits |

## Optimization Techniques

### Zero Heap Allocation

The hot path uses:
- `SmallVec` for small candidate lists (stack-allocated)
- `&str` slices instead of `String` copies
- No heap allocations during stemming

### Thread-Safe Cache

The stemmer uses `DashMap` for O(1) concurrent cache lookups:
- Lock-free sharded hashmap
- Safe to share across threads
- Automatic cache warming

### FST Dictionary

When using an FST dictionary:
- O(1) amortised lookup via mmap
- Memory-mapped file access (no loading overhead)
- Zero-copy lookups

## Memory Usage

- **Base stemmer**: ~1 MB (code + static data)
- **Cache**: Grows with unique words (typically < 10 MB for 100K unique words)
- **FST dictionary**: ~2-5 MB (depending on dictionary size)

## Scaling

The stemmer scales linearly with input size:

```rust
use harmorp::IndonesianStemmer;

let stemmer = IndonesianStemmer::new();
let words: Vec<String> = (0..100_000).map(|_| "membaca".to_string()).collect();
let stems = stemmer.stem_batch(&words);
// ~50 ms for 100,000 words (with cache)
```

## Profiling

To profile performance:

```bash
cargo bench --bench stemmer_bench -- --profile-time=10
```

Or with flamegraph:

```bash
cargo install flamegraph
cargo flamegraph --bench stemmer_bench
```

## Tips for Best Performance

1. **Reuse stemmer instances**: The cache is per-instance
2. **Use batch processing**: More efficient than individual calls
3. **Enable FST dictionary**: Better accuracy with minimal overhead
4. **Warm cache**: Process common words first to populate cache

## Python Bindings Performance

Python bindings have similar performance characteristics:
- ~100-200 ns cache hit (Python overhead)
- ~2-10 µs cold stem
- Batch processing recommended for large datasets

```python
import harmorp

stemmer = harmorp.Stemmer()
# Use stem_batch for multiple words
stems = stemmer.stem_batch(["membaca", "menulis"] * 1000)
```
