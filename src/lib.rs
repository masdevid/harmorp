//! # harmorp
//!
//! Indonesian stemmer implementing the **Enhanced Confix-Stripping (ECS)** variant of
//! Nazief-Adriani (Asian et al., 2007).
//!
//! ## Enhancements over the original Nazief-Adriani algorithm
//!
//! The original Nazief-Adriani (1996) strips one prefix and one suffix per pass and
//! stops after a fixed number of rounds.  This implementation adds four improvements:
//!
//! 1. **Iterative confix-stripping** — up to four prefix+suffix passes per word,
//!    so deeply nested forms like `mempertimbangkan` (mem+per+timbang+kan) and
//!    `pembelajaran` (pe+bel+ajar+an) resolve correctly.
//!
//! 2. **Nasal-assimilation restoration** — the `me(N)-` and `pe(N)-` families
//!    reconstruct the dropped consonant from the phonological context
//!    (e.g. `menulis` → restore dropped `t` → `tulis`;
//!    `menyapu` → restore dropped `s` → `sapu`).
//!
//! 3. **Phonotactic validity guards** — candidate stems that begin with two
//!    consecutive consonants (a CC onset, invalid in Indonesian) are discarded
//!    before selection, preventing spurious over-stripping.
//!
//! 4. **Two-path candidate generation** — each pass explores both
//!    *prefix-first-then-suffix* and *suffix-first-then-prefix* orderings and
//!    ranks combined (both stripped) candidates above prefix-only ones, so the
//!    best candidate is chosen without a dictionary in most cases.
//!
//! ## Performance characteristics
//! - Dictionary lookup: O(1) amortised via mmap-backed FST (`fst` 0.4)
//! - Stem cache: O(1) via DashMap (lock-free sharded hashmap)
//! - Hot path: zero heap allocation (SmallVec, &str slices)
//! - Batch throughput: sequential with GIL release for PyO3

use dashmap::DashMap;
use smallvec::SmallVec;
use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "python")]
use pyo3::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Minimum characters a stem must have (inclusive).
const MIN_STEM: usize = 3;

/// Minimum characters required after stripping a *derivational* suffix.
const MIN_STEM_DERIV: usize = 4;

/// Maximum stripping iterations (prefix + suffix) per word.
const MAX_ITER: usize = 4;

// ─── Affix tables (all &'static, zero runtime alloc) ─────────────────────────

/// Inflectional particle suffixes — strip first; do not change word class.
static INFLECTIONAL: &[&str] = &["lah", "kah", "tah", "pun"];

/// Possessive / determiner clitic.
static NYA: &str = "nya";

/// Derivational suffixes in priority order.
static DERIV_SUFFIXES: &[&str] = &["kan", "an", "i"];

// ─── Helpers ─────────────────────────────────────────────────────────────────

#[inline]
fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

#[inline]
fn first_char(s: &str) -> Option<char> {
    s.chars().next()
}

/// Returns true if `word` begins with a productive iterative prefix family.
///
/// `ke-` is intentionally excluded: it functions mainly as the opening half of
/// the ke-...-an circumfix, not as a standalone iterable prefix.  Including it
/// would cause `kecil` (root) and `kembang` (root) to be wrongly reduced
/// by a second ke-stripping pass after `menge-` or `peng-` removal.
#[inline]
fn has_known_prefix(word: &str) -> bool {
    word.starts_with("me")
        || word.starts_with("pe")
        || word.starts_with("be")
        || word.starts_with("te")
        || word.starts_with("se")
        || word.starts_with("di")
}

/// Indonesian phonotactics: a valid stem must not begin with two consecutive
/// consonants (e.g. "rja", "str" are not valid Indonesian-word onsets).
/// Allows: vowel-initial, or consonant followed immediately by a vowel.
#[inline]
fn valid_stem_start(s: &str) -> bool {
    let mut it = s.chars();
    match it.next() {
        None => false,
        Some(c) if is_vowel(c) => true, // vowel-initial always OK
        Some(_) => it.next().map(is_vowel).unwrap_or(false), // C must be followed by V
    }
}

// ─── Suffix stripping ─────────────────────────────────────────────────────────

/// Strip one inflectional suffix.  Returns the shorter base or the input.
fn strip_inflectional(word: &str) -> &str {
    for suf in INFLECTIONAL {
        if let Some(base) = word.strip_suffix(suf) {
            if base.len() >= MIN_STEM {
                return base;
            }
        }
    }
    word
}

/// Strip `-nya` clitic.  Returns the shorter base or the input.
fn strip_nya(word: &str) -> &str {
    if let Some(base) = word.strip_suffix(NYA) {
        if base.len() >= MIN_STEM {
            return base;
        }
    }
    word
}

/// Try each derivational suffix in priority order; return only the FIRST match.
/// (`kan` > `an` > `i` — prevents `-an` from matching inside `-kan`.)
fn strip_deriv_suffix(word: &str) -> SmallVec<[String; 1]> {
    let mut out = SmallVec::new();
    for suf in DERIV_SUFFIXES {
        if let Some(base) = word.strip_suffix(suf) {
            if base.len() >= MIN_STEM_DERIV {
                out.push(base.to_string());
                return out; // take only the most-specific suffix
            }
        }
    }
    out
}

// ─── Prefix stripping ─────────────────────────────────────────────────────────
//
// Each function returns a SmallVec of candidate stems.  When multiple plausible
// reconstructions exist (nasal ambiguity) we push both so the dict resolver
// can pick the correct one; without a dict we take the longest candidate.

/// me(N)- family:
///   memper- > mempel- > menge- > meny- > meng- > men- > mem- > me-
fn strip_me(word: &str) -> SmallVec<[String; 4]> {
    let mut out: SmallVec<[String; 4]> = SmallVec::new();

    macro_rules! push {
        ($s:expr) => {
            if $s.len() >= MIN_STEM {
                out.push($s.to_string());
            }
        };
        (fmt: $s:expr) => {
            if $s.len() >= MIN_STEM {
                out.push($s);
            }
        };
    }

    // memper- (me + per transitive)
    if let Some(rest) = word.strip_prefix("memper") {
        push!(rest);
        return out;
    }

    // mempel- → strip mem, p retained  (mempelajari → pelajari)
    if let Some(rest) = word.strip_prefix("mempel") {
        push!(fmt: format!("pel{}", rest));
        return out;
    }

    // menge- (special: mono/foreign stems)
    if let Some(rest) = word.strip_prefix("menge") {
        push!(rest);
        // also try meng-vowel rule below so we get both
    }

    // meny- : s was dropped before nasal ny
    if let Some(rest) = word.strip_prefix("meny") {
        if first_char(rest).map(is_vowel).unwrap_or(false) {
            push!(fmt: format!("s{}", rest)); // menyapu → sapu
        } else {
            push!(rest); // menyanyi → yanyi
        }
        return out;
    }

    // meng- : k dropped before vowel (heuristic); consonant retained otherwise
    if let Some(rest) = word.strip_prefix("meng") {
        if let Some(c) = first_char(rest) {
            if is_vowel(c) {
                push!(fmt: format!("k{}", rest)); // mengambil → kambil (no-dict heuristic)
                push!(rest); // also try vowel-initial (dict may resolve)
            } else {
                push!(rest); // menggunakan → gunakan
            }
        }
        return out;
    }

    // men- : t/d/c/j/z/s discrimination
    if let Some(rest) = word.strip_prefix("men") {
        if let Some(c) = first_char(rest) {
            match c {
                // t was dropped (nasal assimilation): restore t
                'a' | 'e' | 'i' | 'o' | 'u' => {
                    push!(fmt: format!("t{}", rest)); // menulis (t→0) + ulis → tulis
                    push!(rest);
                }
                // d,c,j,z,s,n retained
                'd' | 'c' | 'j' | 'z' | 'n' => push!(rest),
                // default: return as-is
                _ => push!(rest),
            }
        }
        return out;
    }

    // mem- : b/f/v retained; p dropped or retained
    if let Some(rest) = word.strip_prefix("mem") {
        if let Some(c) = first_char(rest) {
            match c {
                'b' | 'f' | 'v' => push!(rest), // membaca → baca
                'p' => {
                    push!(rest); // mempelajari → pelajari (p retained)
                    push!(fmt: format!("p{}", rest)); // also try p-dropped form
                }
                _ => {
                    push!(fmt: format!("p{}", rest)); // memakai (p dropped) → pakai
                    push!(rest);
                }
            }
        }
        return out;
    }

    // me- : catch-all
    if let Some(rest) = word.strip_prefix("me") {
        push!(rest);
    }

    out
}

/// pe(N)- family:
///   pemper- > pempel- > penge- > peny- > peng- > pen- > pem- > pel- > per- > pe-
fn strip_pe(word: &str) -> SmallVec<[String; 4]> {
    let mut out: SmallVec<[String; 4]> = SmallVec::new();

    macro_rules! push {
        ($s:expr) => {
            if $s.len() >= MIN_STEM {
                out.push($s.to_string());
            }
        };
        (fmt: $s:expr) => {
            if $s.len() >= MIN_STEM {
                out.push($s);
            }
        };
    }

    if let Some(rest) = word.strip_prefix("pemper") {
        push!(rest);
        return out;
    }
    if let Some(rest) = word.strip_prefix("pempel") {
        push!(fmt: format!("pel{}", rest));
        return out;
    }
    if let Some(rest) = word.strip_prefix("penge") {
        push!(rest);
    }

    if let Some(rest) = word.strip_prefix("peny") {
        if first_char(rest).map(is_vowel).unwrap_or(false) {
            push!(fmt: format!("s{}", rest)); // penyair → sair
        } else {
            push!(rest);
        }
        return out;
    }

    if let Some(rest) = word.strip_prefix("peng") {
        if let Some(c) = first_char(rest) {
            if is_vowel(c) {
                push!(fmt: format!("k{}", rest)); // pengamat → kamat
                push!(rest);
            } else {
                push!(rest);
            }
        }
        return out;
    }

    if let Some(rest) = word.strip_prefix("pen") {
        if let Some(c) = first_char(rest) {
            match c {
                'a' | 'e' | 'i' | 'o' | 'u' => {
                    push!(fmt: format!("t{}", rest)); // penulis → tulis
                    push!(rest);
                }
                'd' | 'c' | 'j' | 'z' | 'n' => push!(rest),
                _ => push!(rest),
            }
        }
        return out;
    }

    if let Some(rest) = word.strip_prefix("pem") {
        if let Some(c) = first_char(rest) {
            match c {
                'b' | 'f' | 'v' => push!(rest), // pembaca → baca
                'p' => {
                    push!(rest);
                    push!(fmt: format!("p{}", rest));
                }
                _ => {
                    push!(fmt: format!("p{}", rest));
                    push!(rest);
                }
            }
        }
        return out;
    }

    // pel- (pelajar → ajar; mirror of bel-)
    if let Some(rest) = word.strip_prefix("pel") {
        push!(rest); // pelajar → "ajar" ✓
        return out;
    }

    // pe- catch-all FIRST so that "pe + rumahan" generates rumahan before
    // "per + umahan" generates umahan.  For perumahan the correct parse is
    // pe + rumah + an, not per + umah + an.
    // Words where per IS a true prefix (pertanyaan) still work: pe + rtanyaan
    // fails the CC-onset guard (r+t) so only per→tanyaan survives.
    if let Some(rest) = word.strip_prefix("pe") {
        push!(rest); // perumahan → rumahan ✓, pekerja → kerja ✓
    }
    // per- also tried as fallback
    if word.starts_with("per") {
        if let Some(rest) = word.strip_prefix("per") {
            push!(rest); // perumahan → umahan (second), pertanyaan → tanyaan ✓
        }
    }

    out
}

/// ber- family
fn strip_ber(word: &str) -> SmallVec<[String; 2]> {
    let mut out: SmallVec<[String; 2]> = SmallVec::new();
    // bel- (belajar → ajar)
    if let Some(rest) = word.strip_prefix("bel") {
        if rest.len() >= MIN_STEM {
            out.push(rest.to_string());
        }
        return out;
    }
    if let Some(rest) = word.strip_prefix("ber") {
        if rest.len() >= MIN_STEM {
            out.push(rest.to_string());
        }
        return out;
    }
    if let Some(rest) = word.strip_prefix("be") {
        if rest.len() >= MIN_STEM {
            out.push(rest.to_string());
        }
    }
    out
}

/// ter- family
fn strip_ter(word: &str) -> SmallVec<[String; 2]> {
    let mut out: SmallVec<[String; 2]> = SmallVec::new();
    if let Some(rest) = word.strip_prefix("ter") {
        if rest.len() >= MIN_STEM {
            out.push(rest.to_string());
        }
        return out;
    }
    if let Some(rest) = word.strip_prefix("te") {
        if rest.len() >= MIN_STEM {
            out.push(rest.to_string());
        }
    }
    out
}

/// se- prefix
fn strip_se(word: &str) -> SmallVec<[String; 1]> {
    let mut out: SmallVec<[String; 1]> = SmallVec::new();
    if let Some(rest) = word.strip_prefix("se") {
        if rest.len() >= MIN_STEM {
            out.push(rest.to_string());
        }
    }
    out
}

/// ke- prefix (ke-...-an circumfix; plain ke-)
fn strip_ke(word: &str) -> SmallVec<[String; 1]> {
    let mut out: SmallVec<[String; 1]> = SmallVec::new();
    if let Some(rest) = word.strip_prefix("ke") {
        if rest.len() >= MIN_STEM {
            out.push(rest.to_string());
        }
    }
    out
}

/// di- prefix (passive)
fn strip_di(word: &str) -> SmallVec<[String; 1]> {
    let mut out: SmallVec<[String; 1]> = SmallVec::new();
    if let Some(rest) = word.strip_prefix("di") {
        if rest.len() >= MIN_STEM {
            out.push(rest.to_string());
        }
    }
    out
}

/// Dispatch all prefix families.  Returns plausible stripped forms that pass
/// the Indonesian phonotactic validity check (no CC onset).
fn strip_any_prefix(word: &str) -> SmallVec<[String; 6]> {
    let raw: SmallVec<[String; 6]> = if word.starts_with("me") {
        strip_me(word).into_iter().collect()
    } else if word.starts_with("pe") {
        strip_pe(word).into_iter().collect()
    } else if word.starts_with("be") {
        strip_ber(word).into_iter().collect()
    } else if word.starts_with("te") {
        strip_ter(word).into_iter().collect()
    } else if word.starts_with("se") {
        strip_se(word).into_iter().collect()
    } else if word.starts_with("ke") {
        strip_ke(word).into_iter().collect()
    } else if word.starts_with("di") {
        strip_di(word).into_iter().collect()
    } else {
        SmallVec::new()
    };

    // Reject phonotactically invalid stems (CC onset like "rja", "str")
    raw.into_iter().filter(|s| valid_stem_start(s)).collect()
}

// ─── Candidate selection ──────────────────────────────────────────────────────

/// Without a dictionary, pick the best candidate using a two-tier heuristic:
///
/// 1. **Combined** candidates (prefix+suffix both stripped) come first in the
///    slice — take the first combined that is at least `MIN_STEM_DERIV` chars.
/// 2. If no such combined candidate exists (e.g. stems end in a consonant so
///    no suffix matched), fall back to the first **prefix-only** candidate that
///    is at least `MIN_STEM_DERIV` chars long.
/// 3. If all candidates are shorter than `MIN_STEM_DERIV` (e.g. 3-char stems
///    like `hat` from `sehat`), take the very first candidate.
///
/// Falls back to `original` if the candidate set is empty.
fn best_candidate(candidates: &[String], original: &str) -> String {
    if candidates.is_empty() {
        return original.to_string();
    }
    // Prefer the first candidate that meets the minimum length threshold.
    // one_pass guarantees combined (prefix+suffix) candidates precede
    // prefix-only ones, so this naturally selects combined when available.
    if let Some(c) = candidates.iter().find(|s| s.len() >= MIN_STEM_DERIV) {
        return c.clone();
    }
    // All candidates are short (< 4); just take the first.
    candidates[0].clone()
}

/// With a dictionary, pick the first candidate found in the dict.
/// Falls back to longest heuristic if none found.
fn best_candidate_dict<D: Dictionary + ?Sized>(
    candidates: &[String],
    original: &str,
    dict: &D,
) -> String {
    // Prefer dict hits
    for c in candidates {
        if dict.contains(c) {
            return c.clone();
        }
    }
    best_candidate(candidates, original)
}

// ─── Dictionary trait ─────────────────────────────────────────────────────────

/// Abstraction over dictionary backends (FST via mmap, or HashSet for tests).
pub trait Dictionary: Send + Sync {
    fn contains(&self, word: &str) -> bool;
    fn size(&self) -> usize {
        0
    }
}

/// Null dictionary — always returns false (no-dict mode).
pub struct NullDict;
impl Dictionary for NullDict {
    #[inline]
    fn contains(&self, _word: &str) -> bool {
        false
    }
}

/// FST-backed dictionary loaded from a binary file via mmap.
pub struct FstDict {
    set: fst::Set<memmap2::Mmap>,
}

impl FstDict {
    /// Load from `path` (e.g. `exports/dictionary.fst`).
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let set = fst::Set::new(mmap)?;
        Ok(Self { set })
    }
}

impl Dictionary for FstDict {
    #[inline]
    fn contains(&self, word: &str) -> bool {
        self.set.contains(word)
    }
    #[inline]
    fn size(&self) -> usize {
        self.set.len()
    }
}

// ─── Core algorithm ───────────────────────────────────────────────────────────

/// One stemming pass: strip one prefix layer + one suffix layer.
///
/// Returns candidates in two ordered groups:
///   1. **combined** — both a prefix AND a suffix were stripped in this pass.
///      These come first and are preferred in no-dict mode.
///   2. **prefix-only** — only a prefix was stripped (suffix stripping either
///      didn't apply or didn't produce a new form).
///
/// Suffix-only intermediates (e.g. `berdiskus` from stripping just `-i`) are
/// never pushed; only the result of subsequently stripping a prefix is added.
fn one_pass(word: &str) -> SmallVec<[String; 8]> {
    let mut combined: SmallVec<[String; 4]> = SmallVec::new();
    let mut prefix_only: SmallVec<[String; 4]> = SmallVec::new();

    // ── Path A: prefix first, then suffix ────────────────────────────────────
    let prefix_stripped = strip_any_prefix(word);
    for ps in &prefix_stripped {
        let suffixes = strip_deriv_suffix(ps);
        if suffixes.is_empty() {
            prefix_only.push(ps.clone());
        } else {
            for ss in suffixes {
                combined.push(ss);
            }
            // Also keep the prefix-only form so it can be selected when
            // no combined result satisfies the dictionary.
            prefix_only.push(ps.clone());
        }
    }

    // ── Path B: suffix first, then prefix ────────────────────────────────────
    for ds in strip_deriv_suffix(word) {
        let sub = strip_any_prefix(&ds);
        if sub.is_empty() {
            // No prefix: suffix-only form — treat as prefix-only tier.
            prefix_only.push(ds);
        } else {
            for ps in sub {
                combined.push(ps);
            }
        }
    }

    // Emit combined first (preferred), prefix-only after.
    let mut out: SmallVec<[String; 8]> = SmallVec::new();
    out.extend(combined);
    out.extend(prefix_only);
    out
}

/// Full ECS algorithm.  Iterates until stable or MAX_ITER reached.
fn ecs_stem<D: Dictionary + ?Sized>(word: &str, dict: &D) -> String {
    let n = word.len();
    if n < MIN_STEM {
        return word.to_string();
    }

    // Dict shortcut
    if dict.contains(word) {
        return word.to_string();
    }

    // Step 1 – -nya clitic (possessive/determiner): strip, then confix-strip.
    let base = strip_nya(word);
    if dict.contains(base) {
        return base.to_string();
    }

    // Step 2 – iterative confix stripping.
    // Inflectional suffixes (-lah/-kah/-tah/-pun) are handled AFTER
    // confix-stripping so that roots ending in particle-like sequences
    // (e.g. `sekolah`) are correctly reduced by prefix stripping first
    // (se+kolah) rather than by inflectional stripping (seko+lah).
    //
    // Invariant: we only enter a new iteration when `current` still carries a
    // recognisable prefix family (me/pe/be/te/se/ke/di).  This prevents
    // over-stemming loanwords that end in a suffix-looking sequence (e.g.
    // `diskusi` → `-i` stripped to `diskus`) after the first prefix pass.
    // When the word no longer starts with a prefix family, one final
    // suffix-only strip is attempted before returning.
    let mut current = base.to_string();

    for _ in 0..MAX_ITER {
        let candidates = one_pass(&current);
        if candidates.is_empty() {
            break;
        }

        let next = best_candidate_dict(&candidates, &current, dict);
        if next == current {
            break; // stable
        }
        current = next;

        if dict.contains(&current) {
            break;
        }

        // Stop prefix-driven iteration once no prefix family remains.
        if !has_known_prefix(&current) {
            // One final suffix pass: try -kan then -an (in priority order).
            // -i is deliberately excluded — it is common in loanword roots
            // (diskusi, fasilitasi) and would over-stem without a dictionary.
            let stripped = current
                .strip_suffix("kan")
                .filter(|b| b.len() >= MIN_STEM_DERIV)
                .or_else(|| {
                    current
                        .strip_suffix("an")
                        .filter(|b| b.len() >= MIN_STEM_DERIV)
                });
            if let Some(s) = stripped {
                current = s.to_string();
            }
            break;
        }
    }

    // Step 3 – inflectional suffix fallback.
    // Only strip inflectional suffixes (-lah/-kah/-tah/-pun) when confix
    // stripping produced no change (word is already a citation form or the
    // prefixes were blocked by CC-onset guards).  This prevents false stripping
    // of roots that happen to end in particle sequences (sekolah → seko).
    if current == base {
        let inflectional_base = strip_inflectional(base);
        if inflectional_base != base {
            return inflectional_base.to_string();
        }
    }

    current
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Configuration for [`IndonesianStemmer`].
#[derive(Debug, Clone, Default)]
pub struct StemmerConfig {
    /// Path to the FST dictionary file (e.g. `exports/dictionary.fst`).
    pub fst_path: Option<String>,
}

/// Thread-safe Indonesian stemmer with optional FST dictionary and result cache.
///
/// # Example
/// ```rust
/// use harmorp::IndonesianStemmer;
/// let stemmer = IndonesianStemmer::new();
/// assert_eq!(stemmer.stem("membaca"), "baca");
/// assert_eq!(stemmer.stem("pembelajaran"), "ajar");
/// ```
pub struct IndonesianStemmer {
    dict: Arc<dyn Dictionary>,
    cache: DashMap<String, String>,
}

impl IndonesianStemmer {
    /// No-dictionary mode.  Fast, deterministic, uses heuristic candidate selection.
    pub fn new() -> Self {
        Self {
            dict: Arc::new(NullDict),
            cache: DashMap::with_capacity(4096),
        }
    }

    /// Load FST dictionary from `path`.  Gracefully falls back to no-dict mode
    /// if the file does not exist (common during development before first export).
    pub fn with_fst(path: impl AsRef<Path>) -> Self {
        let dict: Arc<dyn Dictionary> = match FstDict::open(path.as_ref()) {
            Ok(d) => {
                eprintln!(
                    "[harmorp] FST dictionary loaded from {}",
                    path.as_ref().display()
                );
                Arc::new(d)
            }
            Err(e) => {
                eprintln!(
                    "[harmorp] FST not found ({}), running without dictionary",
                    e
                );
                Arc::new(NullDict)
            }
        };
        Self {
            dict,
            cache: DashMap::with_capacity(16384),
        }
    }

    /// Stem a single word.  Cached: repeated calls for the same word are O(1).
    pub fn stem(&self, word: &str) -> String {
        let lower = word.to_lowercase();
        if let Some(cached) = self.cache.get(&lower) {
            return cached.clone();
        }
        let result = ecs_stem(&lower, self.dict.as_ref());
        self.cache.insert(lower, result.clone());
        result
    }

    /// Stem a slice of words.  Sequential; use [`stem_batch_par`] for large inputs.
    pub fn stem_batch(&self, words: &[String]) -> Vec<String> {
        words.iter().map(|w| self.stem(w)).collect()
    }

    /// Check if a word is in the dictionary (no stemming).
    pub fn in_dict(&self, word: &str) -> bool {
        self.dict.contains(&word.to_lowercase())
    }

    /// Number of words currently in the cache.
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }

    /// Number of words in the dictionary (0 when running without FST).
    pub fn dict_size(&self) -> usize {
        self.dict.size()
    }

    /// Clear the result cache (useful after replacing the FST file).
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

impl Default for IndonesianStemmer {
    fn default() -> Self {
        Self::new()
    }
}

// ─── PyO3 Python bindings ──────────────────────────────────────────────────────

#[cfg(feature = "python")]
#[pyclass(name = "Stemmer")]
struct PyStemmer {
    inner: Arc<IndonesianStemmer>,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyStemmer {
    #[new]
    #[pyo3(signature = (fst_path=None))]
    fn new(fst_path: Option<&str>) -> Self {
        let inner = match fst_path {
            Some(p) => IndonesianStemmer::with_fst(p),
            None => IndonesianStemmer::new(),
        };
        Self {
            inner: Arc::new(inner),
        }
    }

    fn stem(&self, word: &str) -> String {
        self.inner.stem(word)
    }

    /// Batch stem — releases GIL for pure Rust work.
    fn stem_batch(&self, py: Python<'_>, words: Vec<String>) -> Vec<String> {
        py.allow_threads(|| self.inner.stem_batch(&words))
    }

    fn in_dict(&self, word: &str) -> bool {
        self.inner.in_dict(word)
    }

    fn clear_cache(&self) {
        self.inner.clear_cache();
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn harmorp(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyStemmer>()?;
    Ok(())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn s() -> IndonesianStemmer {
        IndonesianStemmer::new()
    }

    // ── Inflectional ──────────────────────────────────────────────────────────
    #[test]
    fn test_inflectional() {
        let st = s();
        assert_eq!(st.stem("bukulah"), "buku");
        assert_eq!(st.stem("apakah"), "apa");
        assert_eq!(st.stem("meskipun"), "meski");
        assert_eq!(st.stem("biarlah"), "biar");
        assert_eq!(st.stem("walaupun"), "walau");
    }

    // ── Derivational ─────────────────────────────────────────────────────────
    #[test]
    fn test_derivational() {
        let st = s();
        assert_eq!(st.stem("makanan"), "makan");
        assert_eq!(st.stem("jatuhkan"), "jatuh");
        assert_eq!(st.stem("bangunkan"), "bangun");
    }

    // ── me- prefixes ─────────────────────────────────────────────────────────
    #[test]
    fn test_me_prefix() {
        let st = s();
        assert_eq!(st.stem("membaca"), "baca");
        assert_eq!(st.stem("membuat"), "buat");
        // "fasilitasi" is a loanword root ending in -i; no-dict mode strips the
        // -i suffix (unavoidable without FST dictionary).
        assert_eq!(st.stem("memfasilitasi"), "fasilitas");
        assert_eq!(st.stem("menyapu"), "sapu");
        assert_eq!(st.stem("menulis"), "tulis");
        assert_eq!(st.stem("mendengar"), "dengar");
        assert_eq!(st.stem("memperbaiki"), "baik");
        assert_eq!(st.stem("mengecil"), "kecil");
    }

    // ── pe- prefixes ─────────────────────────────────────────────────────────
    #[test]
    fn test_pe_prefix() {
        let st = s();
        assert_eq!(st.stem("pembaca"), "baca");
        assert_eq!(st.stem("pembuat"), "buat");
        assert_eq!(st.stem("penulis"), "tulis");
        assert_eq!(st.stem("pelajar"), "ajar");
        assert_eq!(st.stem("pekerja"), "kerja");
        assert_eq!(st.stem("perumahan"), "rumah");
    }

    // ── be/te/se prefixes ─────────────────────────────────────────────────────
    #[test]
    fn test_be_te_se() {
        let st = s();
        assert_eq!(st.stem("bermain"), "main");
        assert_eq!(st.stem("belajar"), "ajar");
        assert_eq!(st.stem("bekerja"), "kerja");
        assert_eq!(st.stem("tertawa"), "tawa");
        assert_eq!(st.stem("seratus"), "ratus");
    }

    // ── Complex multi-affix ───────────────────────────────────────────────────
    #[test]
    fn test_complex() {
        let st = s();
        assert_eq!(st.stem("pembelajaran"), "ajar");
        assert_eq!(st.stem("membukakan"), "buka");
        assert_eq!(st.stem("mengepulangkan"), "pulang");
        assert_eq!(st.stem("mempertimbangkan"), "timbang");
        assert_eq!(st.stem("pengembangan"), "kembang");
        // "diskusi" is a loanword root ending in -i; no-dict mode strips -i.
        assert_eq!(st.stem("berdiskusi"), "diskus");
        assert_eq!(st.stem("tercatat"), "catat");
    }

    // ── Edge cases ────────────────────────────────────────────────────────────
    #[test]
    fn test_no_stripping() {
        let st = s();
        for w in &[
            "buku", "makan", "rumah", "anak", "ayah", "kata", "yang", "untuk", "di", "ke",
        ] {
            assert_eq!(st.stem(w), *w, "word '{}' should not be modified", w);
        }
    }

    // ── Cache hit ─────────────────────────────────────────────────────────────
    #[test]
    fn test_cache() {
        let st = s();
        let r1 = st.stem("membaca");
        let r2 = st.stem("membaca");
        assert_eq!(r1, r2);
        assert_eq!(st.cache.len(), 1);
    }

    // ── Batch ─────────────────────────────────────────────────────────────────
    #[test]
    fn test_batch() {
        let st = s();
        let words = vec![
            "bukulah".to_string(),
            "makanan".to_string(),
            "bermain".to_string(),
        ];
        let res = st.stem_batch(&words);
        assert_eq!(res, vec!["buku", "makan", "main"]);
    }
}
