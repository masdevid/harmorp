# HARMorp

A fast Indonesian stemmer implementation using the Nazief-Adriani algorithm.

## Features

- **Nazief-Adriani Algorithm**: Implements the standard Indonesian stemming algorithm
- **Dictionary Support**: Load custom dictionaries for accurate stemming
- **Case Insensitive**: Handles both uppercase and lowercase input
- **Zero Dependencies**: Minimal external dependencies for fast compilation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
harmorp = "0.1.0"
```

## Usage

### Basic Usage

```rust
use harmorp::Stemmer;

let stemmer = Stemmer::new();
let stemmed = stemmer.stem("membaca");
println!("{}", stemmed); // Output: baca
```

### With Dictionary

```rust
use harmorp::Stemmer;

let words = vec![
    "makan".to_string(),
    "minum".to_string(),
    "baca".to_string(),
];

let stemmer = Stemmer::with_dictionary(words);
let stemmed = stemmer.stem("membaca");
println!("{}", stemmed); // Output: baca
```

### Adding Words to Dictionary

```rust
use harmorp::Stemmer;

let mut stemmer = Stemmer::new();
stemmer.add_word("makan");
stemmer.add_words(&vec!["minum".to_string(), "baca".to_string()]);

let stemmed = stemmer.stem("makanlah");
println!("{}", stemmed); // Output: makan
```

### Convenience Function

```rust
use harmorp::stem;

let stemmed = stem("membaca");
println!("{}", stemmed);
```

## Algorithm

The stemmer implements the Nazief-Adriani algorithm with the following steps:

1. **Remove inflectional suffixes**: kah, lah, tah, pun, nya, ku, mu
2. **Remove derivational suffixes**: i, an, kan
3. **Remove derivational prefixes**: meng, meny, men, mem, me, peng, peny, pen, pem, pe, di, ter, ber, ke, se
4. **Dictionary lookup**: Check if result exists in dictionary

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
