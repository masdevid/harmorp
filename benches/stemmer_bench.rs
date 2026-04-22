use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use harmorp::IndonesianStemmer;

// ── Representative word sets ──────────────────────────────────────────────────

/// Root / base words — should return immediately (short-circuit or one pass).
const ROOTS: &[&str] = &[
    "buku", "makan", "rumah", "anak", "kerja", "besar", "kecil", "tulis", "baca", "jual", "beli",
    "ajar", "pikir", "dapat",
];

/// Single-prefix words — one prefix strip.
const SINGLE_PREFIX: &[&str] = &[
    "membaca", "menulis", "berjalan", "terbang", "sekolah", "bermain", "bekerja", "pelajar",
    "pekerja", "pembaca",
];

/// Multi-affix words — 2–3 strip iterations.
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

/// Mixed realistic corpus (roots + derived + complex).
fn corpus() -> Vec<String> {
    let mut v = Vec::with_capacity(ROOTS.len() + SINGLE_PREFIX.len() + MULTI_AFFIX.len());
    v.extend(ROOTS.iter().map(|s| s.to_string()));
    v.extend(SINGLE_PREFIX.iter().map(|s| s.to_string()));
    v.extend(MULTI_AFFIX.iter().map(|s| s.to_string()));
    v
}

// ── Benchmarks ────────────────────────────────────────────────────────────────

fn bench_single_words(c: &mut Criterion) {
    let stemmer = IndonesianStemmer::new();
    let mut group = c.benchmark_group("stem_single");

    for word in ROOTS.iter().chain(SINGLE_PREFIX).chain(MULTI_AFFIX) {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("cold", word), word, |b, w| {
            b.iter(|| {
                stemmer.clear_cache();
                stemmer.stem(black_box(w))
            });
        });
    }
    group.finish();
}

fn bench_cached(c: &mut Criterion) {
    let stemmer = IndonesianStemmer::new();
    // Warm cache
    let corp = corpus();
    for w in &corp {
        stemmer.stem(w);
    }

    let mut group = c.benchmark_group("stem_cached");
    for word in MULTI_AFFIX {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("hot", word), word, |b, w| {
            b.iter(|| stemmer.stem(black_box(w)));
        });
    }
    group.finish();
}

fn bench_batch(c: &mut Criterion) {
    let stemmer = IndonesianStemmer::new();
    let corp = corpus();

    let mut group = c.benchmark_group("stem_batch");
    group.throughput(Throughput::Elements(corp.len() as u64));
    group.bench_function("corpus_cold", |b| {
        b.iter(|| {
            stemmer.clear_cache();
            stemmer.stem_batch(black_box(&corp))
        });
    });
    group.bench_function("corpus_hot", |b| {
        b.iter(|| stemmer.stem_batch(black_box(&corp)));
    });
    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let stemmer = IndonesianStemmer::new();
    // 10 000-word synthetic batch (repeat corpus)
    let large: Vec<String> = corpus().into_iter().cycle().take(10_000).collect();

    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Elements(large.len() as u64));
    group.bench_function("10k_words_hot", |b| {
        // warm
        stemmer.stem_batch(&large);
        b.iter(|| stemmer.stem_batch(black_box(&large)));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_single_words,
    bench_cached,
    bench_batch,
    bench_throughput
);
criterion_main!(benches);
