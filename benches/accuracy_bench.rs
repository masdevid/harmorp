use criterion::{black_box, criterion_group, criterion_main, Criterion};
use harmorp::IndonesianStemmer;
use sastrawi::{Dictionary, Stemmer as SastrawiStemmer};
use serde::Deserialize;

#[derive(Deserialize)]
struct TestWord {
    input: String,
    expected: String,
}

fn load_test_data() -> Vec<TestWord> {
    let data = include_str!("../tests/data/accuracy_test.json");
    serde_json::from_str(data).expect("Failed to parse accuracy test data")
}

fn create_sastrawi_dict() -> Dictionary {
    Dictionary::new()
}

fn bench_accuracy_harmorp(c: &mut Criterion) {
    let stemmer = IndonesianStemmer::new();
    let test_data = load_test_data();

    let mut group = c.benchmark_group("accuracy_harmorp");

    group.bench_function("accuracy", |b| {
        b.iter(|| {
            let mut correct = 0;
            let mut total = 0;

            for word in &test_data {
                let result = stemmer.stem(black_box(&word.input));
                if result == word.expected {
                    correct += 1;
                }
                total += 1;
            }

            (correct, total)
        });
    });

    group.finish();
}

fn bench_accuracy_sastrawi(c: &mut Criterion) {
    let dict = create_sastrawi_dict();
    let stemmer = SastrawiStemmer::new(&dict);
    let test_data = load_test_data();

    let mut group = c.benchmark_group("accuracy_sastrawi");

    group.bench_function("accuracy", |b| {
        b.iter(|| {
            let mut correct = 0;
            let mut total = 0;

            for word in &test_data {
                let mut input = word.input.clone();
                stemmer.stem_word(&mut input);
                if input == word.expected {
                    correct += 1;
                }
                total += 1;
            }

            (correct, total)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_accuracy_harmorp, bench_accuracy_sastrawi);
criterion_main!(benches);
