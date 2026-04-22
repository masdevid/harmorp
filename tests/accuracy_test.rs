use harmorp::IndonesianStemmer;
use sastrawi::{Dictionary, Stemmer as SastrawiStemmer};
use serde::Deserialize;

#[derive(Deserialize)]
struct TestWord {
    input: String,
    expected: String,
}

fn load_test_data() -> Vec<TestWord> {
    let data = include_str!("data/accuracy_test.json");
    serde_json::from_str(data).expect("Failed to parse accuracy test data")
}

#[test]
fn test_harmorp_accuracy() {
    let stemmer = IndonesianStemmer::new();
    let test_data = load_test_data();

    let mut correct = 0;
    let mut total = 0;
    let mut mismatches = Vec::new();

    for word in &test_data {
        let result = stemmer.stem(&word.input);
        if result == word.expected {
            correct += 1;
        } else {
            mismatches.push((word.input.clone(), word.expected.clone(), result));
        }
        total += 1;
    }

    let accuracy = (correct as f64 / total as f64) * 100.0;

    println!("\n=== harmorp Accuracy Results ===");
    println!("Total words: {}", total);
    println!("Correct: {}", correct);
    println!("Incorrect: {}", total - correct);
    println!("Accuracy: {:.2}%", accuracy);

    if !mismatches.is_empty() {
        println!("\nMismatches:");
        for (input, expected, got) in &mismatches {
            println!("  Input: {}, Expected: {}, Got: {}", input, expected, got);
        }
    }

    assert!(accuracy > 70.0, "Accuracy too low: {:.2}%", accuracy);
}

#[test]
fn test_sastrawi_accuracy() {
    let dict = Dictionary::new();
    let stemmer = SastrawiStemmer::new(&dict);
    let test_data = load_test_data();

    let mut correct = 0;
    let mut total = 0;
    let mut mismatches = Vec::new();

    for word in &test_data {
        let mut input = word.input.clone();
        stemmer.stem_word(&mut input);
        if input == word.expected {
            correct += 1;
        } else {
            mismatches.push((word.input.clone(), word.expected.clone(), input));
        }
        total += 1;
    }

    let accuracy = (correct as f64 / total as f64) * 100.0;

    println!("\n=== sastrawi Accuracy Results ===");
    println!("Total words: {}", total);
    println!("Correct: {}", correct);
    println!("Incorrect: {}", total - correct);
    println!("Accuracy: {:.2}%", accuracy);

    if !mismatches.is_empty() {
        println!("\nMismatches:");
        for (input, expected, got) in &mismatches {
            println!("  Input: {}, Expected: {}, Got: {}", input, expected, got);
        }
    }

    assert!(accuracy > 70.0, "Accuracy too low: {:.2}%", accuracy);
}
