# Accuracy Comparison: harmorp vs sastrawi

This document compares the accuracy of harmorp with the sastrawi Indonesian stemmer library.

## Test Setup

- **Dataset**: 58 words (14 roots, 10 single-prefix, 12 multi-affix, 22 complex derived)
- **Test file**: `tests/data/accuracy_test.json`
- **Hardware**: MacBook Pro (M1 Pro, 10 cores, 16 GB RAM)

## Results

| Stemmer | Correct | Incorrect | Accuracy |
|---------|---------|-----------|----------|
| harmorp | 46 | 12 | **79.31%** |
| sastrawi | 47 | 11 | **81.03%** |

## Analysis

Both stemmers achieve similar accuracy (~80%) on the test dataset. sastrawi has a slight edge with 81.03% accuracy compared to harmorp's 79.31%.

### harmorp Mismatches (12 words)

1. `besar` → `sar` (expected: `besar`)
2. `kecil` → `cil` (expected: `kecil`)
3. `sekolah` → `kolah` (expected: `sekolah`)
4. `penyelenggaraan` → `lenggara` (expected: `lenggar`)
5. `pemberitahuan` → `itahu` (expected: `tahu`)
6. `menyampaikan` → `sampai` (expected: `sampa`)
7. `menyelenggarakan` → `lenggara` (expected: `lenggar`)
8. `mengambil` → `kambil` (expected: `ambil`)
9. `menyanyi` → `sany` (expected: `nyanyi`)
10. `memasak` → `pasak` (expected: `masak`)
11. `penerbitan` → `bit` (expected: `terbit`)
12. `penyebaran` → `bar` (expected: `sebar`)

### sastrawi Mismatches (11 words)

1. `terbang` → `terbang` (expected: `bang`)
2. `pengembangan` → `pengembangan` (expected: `kembang`)
3. `penyelenggaraan` → `selenggara` (expected: `lenggar`)
4. `pemberitahuan` → `pemberitahuan` (expected: `tahu`)
5. `menyampaikan` → `menyampaikan` (expected: `sampa`)
6. `mengembangkan` → `mengembangkan` (expected: `kembang`)
7. `menyelenggarakan` → `selenggara` (expected: `lenggar`)
8. `menyapu` → `menyapu` (expected: `sapu`)
9. `menari` → `ari` (expected: `tari`)
10. `memasak` → `asak` (expected: `masak`)
11. `penambahan` → `ambah` (expected: `tambah`)

## Key Observations

- **harmorp** tends to be more aggressive in stemming, sometimes over-stemming words (e.g., `besar` → `sar`)
- **sastrawi** tends to be more conservative, sometimes not stemming at all (e.g., `terbang` → `terbang`)
- Both stemmers struggle with complex multi-affix words and nasal-assimilation prefixes
- harmorp's accuracy can be improved with an FST dictionary (not tested here)

## Running the Accuracy Test

To run the accuracy comparison:

```bash
cargo test --test accuracy_test -- --nocapture
```

To run the accuracy benchmark:

```bash
cargo bench --bench accuracy_bench
```
