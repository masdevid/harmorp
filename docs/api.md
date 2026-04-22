# API Documentation

## IndonesianStemmer

The main entry point for Indonesian stemming.

### Constructor

#### `new()`

Creates a new stemmer without a dictionary.

```rust
use harmorp::IndonesianStemmer;

let stemmer = IndonesianStemmer::new();
```

#### `with_fst(path: &str)`

Creates a new stemmer with an FST dictionary for improved accuracy.

```rust
use harmorp::IndonesianStemmer;

let stemmer = IndonesianStemmer::with_fst("dictionary.fst");
```

If the file doesn't exist, the stemmer silently falls back to no-dictionary mode.

### Methods

#### `stem(&self, word: &str) -> String`

Stems a single word.

```rust
let stemmed = stemmer.stem("membaca");
assert_eq!(stemmed, "baca");
```

**Parameters:**
- `word`: The word to stem

**Returns:**
- The stemmed word as a `String`

#### `stem_batch(&self, words: &[String]) -> Vec<String>`

Stems multiple words efficiently.

```rust
let words = vec!["membaca".to_string(), "menulis".to_string()];
let stems = stemmer.stem_batch(&words);
// Returns: ["baca", "tulis"]
```

**Parameters:**
- `words`: A slice of words to stem

**Returns:**
- A vector of stemmed words

### Performance Characteristics

- **Cache hit**: ~50 ns (after first stem of a word)
- **Cold stem**: ~1-5 µs (first time seeing a word)
- **Batch processing**: ~5 ms for 10,000 words

The stemmer uses a thread-safe cache (DashMap) for O(1) repeated lookups.

### Thread Safety

The stemmer can be safely shared across threads:

```rust
use std::sync::Arc;
use harmorp::IndonesianStemmer;

let stemmer = Arc::new(IndonesianStemmer::new());
// Safe to clone and share across threads
```
