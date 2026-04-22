use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use harmorp::IndonesianStemmer;
use sastrawi::{Dictionary, Stemmer as SastrawiStemmer};

// ── Representative word sets ──────────────────────────────────────────────────

const ROOTS: &[&str] = &[
    "buku", "makan", "rumah", "anak", "kerja", "besar", "kecil", "tulis", "baca", "jual", "beli",
    "ajar", "pikir", "dapat",
];

const SINGLE_PREFIX: &[&str] = &[
    "membaca", "menulis", "berjalan", "terbang", "sekolah", "bermain", "bekerja", "pelajar",
    "pekerja", "pembaca",
];

const MULTI_AFFIX: &[&str] = &[
    "pembelajaran",
    "membukakan",
    "pengembangan",
    "mempertimbangkan",
    "penyelenggaraan",
    "mengepulangkan",
    "pembangunan",
    "pemberitahuan",
    "menyampaikan",
    "mengembangkan",
    "menyelenggarakan",
    "mempertahankan",
];

fn corpus() -> Vec<String> {
    let mut v = Vec::with_capacity(ROOTS.len() + SINGLE_PREFIX.len() + MULTI_AFFIX.len());
    v.extend(ROOTS.iter().map(|s| s.to_string()));
    v.extend(SINGLE_PREFIX.iter().map(|s| s.to_string()));
    v.extend(MULTI_AFFIX.iter().map(|s| s.to_string()));
    v
}

fn create_sastrawi_dict() -> Dictionary {
    // Create a basic dictionary with common root words
    Dictionary::custom(ROOTS)
}

// ── Comparison Benchmarks ───────────────────────────────────────────────────────

fn compare_single_words(c: &mut Criterion) {
    let harmorp = IndonesianStemmer::new();
    let dict = create_sastrawi_dict();
    let sastrawi = SastrawiStemmer::new(&dict);

    let mut group = c.benchmark_group("compare_single_cold");

    for word in SINGLE_PREFIX.iter().chain(MULTI_AFFIX) {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(BenchmarkId::new("harmorp", word), word, |b, w| {
            b.iter(|| {
                harmorp.clear_cache();
                harmorp.stem(black_box(w))
            });
        });

        group.bench_with_input(BenchmarkId::new("sastrawi", word), word, |b, w| {
            b.iter(|| {
                let mut word_mut = w.to_string();
                sastrawi.stem_word(&mut word_mut);
                word_mut
            });
        });
    }
    group.finish();
}

fn compare_batch(c: &mut Criterion) {
    let harmorp = IndonesianStemmer::new();
    let dict = create_sastrawi_dict();
    let sastrawi = SastrawiStemmer::new(&dict);
    let corp = corpus();

    let mut group = c.benchmark_group("compare_batch");
    group.throughput(Throughput::Elements(corp.len() as u64));

    group.bench_function("harmorp_cold", |b| {
        b.iter(|| {
            harmorp.clear_cache();
            harmorp.stem_batch(black_box(&corp))
        });
    });

    group.bench_function("harmorp_hot", |b| {
        // Warm cache
        harmorp.stem_batch(&corp);
        b.iter(|| harmorp.stem_batch(black_box(&corp)));
    });

    group.bench_function("sastrawi", |b| {
        b.iter(|| {
            corp.iter()
                .map(|w| {
                    let mut w_mut = w.clone();
                    sastrawi.stem_word(&mut w_mut);
                    w_mut
                })
                .collect::<Vec<_>>()
        });
    });

    group.finish();
}

fn compare_throughput(c: &mut Criterion) {
    let harmorp = IndonesianStemmer::new();
    let dict = create_sastrawi_dict();
    let sastrawi = SastrawiStemmer::new(&dict);
    let large: Vec<String> = corpus().into_iter().cycle().take(10_000).collect();

    let mut group = c.benchmark_group("compare_throughput");
    group.throughput(Throughput::Elements(large.len() as u64));

    group.bench_function("harmorp_10k", |b| {
        harmorp.stem_batch(&large);
        b.iter(|| harmorp.stem_batch(black_box(&large)));
    });

    group.bench_function("sastrawi_10k", |b| {
        b.iter(|| {
            large
                .iter()
                .map(|w| {
                    let mut w_mut = w.clone();
                    sastrawi.stem_word(&mut w_mut);
                    w_mut
                })
                .collect::<Vec<_>>()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    compare_single_words,
    compare_batch,
    compare_throughput
);
criterion_main!(benches);
