# harmorp

[![CI](https://github.com/masdevid/harmorp/actions/workflows/ci.yml/badge.svg)](https://github.com/masdevid/harmorp/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/harmorp.svg)](https://crates.io/crates/harmorp)
[![docs.rs](https://docs.rs/harmorp/badge.svg)](https://docs.rs/harmorp)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

An Indonesian stemmer implementing the **Enhanced Confix-Stripping (ECS)** variant
of the Nazief-Adriani algorithm (Asian et al., 2007).

## Enhancements over the original Nazief-Adriani

The original Nazief-Adriani (1996) applies one prefix and one suffix strip per pass.
This implementation adds four improvements:

| Enhancement | Effect |
|---|---|
| **Iterative confix-stripping** (up to 4 passes) | Handles deeply nested forms: `mempertimbangkan` → `timbang`, `pembelajaran` → `ajar` |
| **Nasal-assimilation restoration** | Reconstructs dropped consonants: `menulis` → `tulis` (t), `menyapu` → `sapu` (s) |
| **Phonotactic validity guards** | Discards CC-onset candidates (invalid in Indonesian), preventing over-stripping |
| **Two-path candidate generation** | Explores both prefix-first and suffix-first orderings; ranks combined candidates higher for better no-dict accuracy |

## Additional features

- **Thread-safe cache**: O(1) repeated lookups via DashMap (lock-free sharded hashmap)
- **FST dictionary**: Optional O(1) amortised root-word lookup via mmap-backed FST
- **Zero heap allocation**: Hot path uses SmallVec and `&str` slices
- **Batch processing**: Efficient multi-word stemming via `stem_batch`
- **Python bindings**: Optional PyO3 bindings (feature-gated)

## Installation

```toml
[dependencies]
harmorp = "0.1.1"
```

## Usage

### Basic

```rust
use harmorp::IndonesianStemmer;

let stemmer = IndonesianStemmer::new();

assert_eq!(stemmer.stem("membaca"),      "baca");
assert_eq!(stemmer.stem("pembelajaran"), "ajar");
assert_eq!(stemmer.stem("pengembangan"), "kembang");
assert_eq!(stemmer.stem("memperbaiki"),  "baik");
```

### Batch processing

```rust
use harmorp::IndonesianStemmer;

let stemmer = IndonesianStemmer::new();
let words = vec![
    "membaca".to_string(),
    "menulis".to_string(),
    "berjalan".to_string(),
];
let stems = stemmer.stem_batch(&words);
// ["baca", "tulis", "jalan"]
```

### With FST dictionary

An FST dictionary improves accuracy for ambiguous nasal-assimilation cases
(e.g. `meng-` + vowel-initial roots). Build one with the [`fst`] crate.

```rust
use harmorp::IndonesianStemmer;

let stemmer = IndonesianStemmer::with_fst("exports/dictionary.fst");
assert_eq!(stemmer.stem("mengambil"), "ambil");
```

If the file does not exist the stemmer silently falls back to no-dictionary mode,
so this is safe to use during development.

### Python bindings

```bash
cargo build --features python
```

```python
import harmorp

stemmer = harmorp.Stemmer()
print(stemmer.stem("membaca"))        # baca
print(stemmer.stem_batch(["membaca", "menulis"]))  # ['baca', 'tulis']
```

## Algorithm

The ECS variant of Nazief-Adriani strips affixes iteratively (up to 4 passes):

1. **`-nya` clitic** — possessive/determiner (`bukunya` → `buku`)
2. **Iterative confix-stripping** — per pass: strip one prefix family + one derivational suffix
   - Prefix families: `me(N)-`, `pe(N)-`, `ber-`, `ter-`, `se-`, `ke-`, `di-`
   - Derivational suffixes (priority order): `-kan` > `-an` > `-i`
3. **Inflectional suffix fallback** — `-lah`, `-kah`, `-tah`, `-pun` (only when no prefix matched)

Phonotactic validity (no CC-onset) is enforced on every candidate to prevent over-stemming.

Without a dictionary, nasal-assimilation ambiguity is resolved by preferring the longer candidate.
With a dictionary, the first candidate found in the FST wins.

## Performance

Benchmarked with `cargo bench --bench stemmer_bench`:

| Scenario | Typical latency |
|---|---|
| Cache hit (warm) | ~50 ns |
| Single word (cold) | ~1–5 µs |
| 10 000-word batch (hot) | ~5 ms |

## Documentation

- [API Documentation](docs/api.md) - Complete API reference
- [Algorithm Documentation](docs/algorithm.md) - Detailed algorithm explanation
- [Performance Documentation](docs/performance.md) - Performance characteristics and benchmarks
- [Benchmark Comparison](docs/benchmark_comparison.md) - harmorp vs sastrawi performance comparison

## License

MIT — see [LICENSE](LICENSE).

## Sponsor

If you find this project useful, consider [sponsoring](https://github.com/sponsors/masdevid) its development.
