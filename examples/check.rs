use harmorp::IndonesianStemmer;

fn main() {
    let s = IndonesianStemmer::new();
    let cases = [
        ("sehat", "hat"),
        ("sekolah", "kolah"),
        ("tetap", "tap"),
        ("apakah", "apa"),
        ("walaupun", "walau"),
        ("mengambil", "kambil"),
        ("pengamat", "kamat"),
        ("berdiskusi", "diskus"),
        ("makan", "makan"),
    ];
    for (word, expected) in &cases {
        let actual = s.stem(word);
        let status = if &actual == expected { "OK  " } else { "FAIL" };
        println!(
            "{} {:20} → {:15} (expected {})",
            status, word, actual, expected
        );
    }
}
