//! Integration tests using JSON test data
//!
//! These tests load test cases from JSON files in tests/data/
//! and verify the stemmer produces expected outputs.

use harmorp::IndonesianStemmer;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct TestCase {
    word: String,
    expected: String,
    #[serde(default)]
    rule: String,
    #[serde(default)]
    rules: Vec<String>,
    #[serde(default)]
    prefix: String,
    #[serde(default)]
    type_: String,
    #[serde(rename = "type", default)]
    type_field: String,
    #[serde(default)]
    notes: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct TestData {
    description: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    prefix_type: String,
    #[serde(default)]
    prefix_types: Vec<String>,
    #[serde(default)]
    source: String,
    test_cases: Vec<TestCase>,
    #[serde(default)]
    pending_from_scraper: Vec<String>,
    #[serde(default)]
    ambiguous_cases: Vec<HashMap<String, String>>,
}

fn load_test_data(filename: &str) -> TestData {
    let path = Path::new("tests/data").join(filename);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e))
}

fn run_test_cases(data: &TestData) {
    let stemmer = IndonesianStemmer::new();
    let mut failures = Vec::new();

    for case in &data.test_cases {
        let result = stemmer.stem(&case.word);
        if result != case.expected {
            failures.push((
                case.word.clone(),
                case.expected.clone(),
                result,
                case.notes.clone(),
            ));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== {} Failures ===", data.category);
        for (word, expected, got, notes) in &failures {
            eprintln!(
                "  {}: expected '{}', got '{}' ({})",
                word, expected, got, notes
            );
        }
        panic!("{} test cases failed in {}", failures.len(), data.category);
    }
}

#[test]
fn test_inflectional_suffixes() {
    let data = load_test_data("inflectional_suffixes.json");
    run_test_cases(&data);
}

#[test]
fn test_derivational_suffixes() {
    let data = load_test_data("derivational_suffixes.json");
    run_test_cases(&data);
}

#[test]
fn test_me_prefixes() {
    let data = load_test_data("me_prefixes.json");
    run_test_cases(&data);
}

#[test]
fn test_pe_prefixes() {
    let data = load_test_data("pe_prefixes.json");
    run_test_cases(&data);
}

#[test]
fn test_be_te_se_prefixes() {
    let data = load_test_data("be_te_se_prefixes.json");
    run_test_cases(&data);
}

#[test]
fn test_complex_forms() {
    let data = load_test_data("complex_forms.json");
    run_test_cases(&data);
}

#[test]
fn test_edge_cases() {
    let data = load_test_data("edge_cases.json");
    run_test_cases(&data);
}

/// Test that all pending words from scraper can at least be processed without panicking
#[test]
fn test_pending_words_dont_panic() {
    let files = [
        "inflectional_suffixes.json",
        "derivational_suffixes.json",
        "me_prefixes.json",
        "pe_prefixes.json",
        "be_te_se_prefixes.json",
        "complex_forms.json",
        "edge_cases.json",
    ];

    let stemmer = IndonesianStemmer::new();

    for file in &files {
        let data = load_test_data(file);
        for word in &data.pending_from_scraper {
            let _result = stemmer.stem(word); // Should not panic
        }
    }
}

/// Benchmark-style test for batch processing
#[test]
fn test_batch_processing() {
    let stemmer = IndonesianStemmer::new();

    // Load all test words
    let mut all_words = Vec::new();
    let files = [
        "inflectional_suffixes.json",
        "derivational_suffixes.json",
        "me_prefixes.json",
        "pe_prefixes.json",
        "be_te_se_prefixes.json",
        "complex_forms.json",
        "edge_cases.json",
    ];

    for file in &files {
        let data = load_test_data(file);
        for case in &data.test_cases {
            all_words.push(case.word.clone());
        }
        all_words.extend(data.pending_from_scraper.clone());
    }

    // Process batch
    let results = stemmer.stem_batch(&all_words);

    assert_eq!(results.len(), all_words.len());
    assert!(!results.iter().any(|s| s.is_empty()));
}

/// Print summary of pending words that need verification
#[test]
#[ignore = "Diagnostic test - run with --ignored to see pending words"]
fn print_pending_summary() {
    let files = [
        ("inflectional_suffixes.json", "Inflectional"),
        ("derivational_suffixes.json", "Derivational"),
        ("me_prefixes.json", "me- Prefixes"),
        ("pe_prefixes.json", "pe- Prefixes"),
        ("be_te_se_prefixes.json", "be/te/se Prefixes"),
        ("complex_forms.json", "Complex Forms"),
        ("edge_cases.json", "Edge Cases"),
    ];

    println!("\n=== Pending Words Summary (awaiting scraper data) ===\n");
    let mut total = 0;

    for (file, category) in &files {
        let data = load_test_data(file);
        let count = data.pending_from_scraper.len();
        total += count;
        if count > 0 {
            println!("{}: {} words pending", category, count);
            for word in &data.pending_from_scraper[..count.min(5)] {
                println!("  - {}", word);
            }
            if count > 5 {
                println!("  ... and {} more", count - 5);
            }
            println!();
        }
    }

    println!("Total pending words: {}", total);
}
