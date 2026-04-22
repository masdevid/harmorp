//! HARMorp - Indonesian Stemmer
//!
//! A fast Indonesian stemmer implementation using the Nazief-Adriani algorithm.
//! This library provides stemming functionality for Indonesian text processing.

use std::collections::HashSet;

/// Indonesian stemmer using Nazief-Adriani algorithm
pub struct Stemmer {
    dictionary: HashSet<String>,
}

impl Stemmer {
    /// Create a new stemmer with an empty dictionary
    pub fn new() -> Self {
        Stemmer {
            dictionary: HashSet::new(),
        }
    }

    /// Create a new stemmer with a pre-loaded dictionary
    pub fn with_dictionary(words: Vec<String>) -> Self {
        let mut dictionary = HashSet::new();
        for word in words {
            dictionary.insert(word.to_lowercase());
        }
        Stemmer { dictionary }
    }

    /// Add a word to the dictionary
    pub fn add_word(&mut self, word: &str) {
        self.dictionary.insert(word.to_lowercase());
    }

    /// Add multiple words to the dictionary
    pub fn add_words(&mut self, words: &[String]) {
        for word in words {
            self.dictionary.insert(word.to_lowercase());
        }
    }

    /// Check if a word exists in the dictionary
    fn is_in_dictionary(&self, word: &str) -> bool {
        self.dictionary.contains(word)
    }

    /// Stem a single word
    pub fn stem(&self, word: &str) -> String {
        let word = word.to_lowercase();
        
        // Return original if in dictionary
        if self.is_in_dictionary(&word) {
            return word;
        }

        let mut result = word.clone();
        
        // Step 1: Remove inflectional suffixes (particle, possessive, derivation)
        result = self.remove_inflectional_suffixes(&result);
        
        // Step 2: Remove derivational suffixes
        result = self.remove_derivational_suffixes(&result);
        
        // Step 3: Remove derivational prefixes
        result = self.remove_derivational_prefixes(&result);
        
        // Step 4: Check if result is in dictionary, if not return original
        if self.is_in_dictionary(&result) {
            result
        } else {
            word
        }
    }

    /// Remove inflectional suffixes (particle, possessive)
    fn remove_inflectional_suffixes(&self, word: &str) -> String {
        let suffixes = ["kah", "lah", "tah", "pun", "nya", "ku", "mu"];
        
        for suffix in &suffixes {
            if word.ends_with(suffix) {
                let base = &word[..word.len() - suffix.len()];
                if self.is_in_dictionary(base) {
                    return base.to_string();
                }
            }
        }
        
        word.to_string()
    }

    /// Remove derivational suffixes
    fn remove_derivational_suffixes(&self, word: &str) -> String {
        let suffixes = ["i", "an", "kan"];
        
        for suffix in &suffixes {
            if word.ends_with(suffix) {
                let base = &word[..word.len() - suffix.len()];
                if self.is_in_dictionary(base) {
                    return base.to_string();
                }
            }
        }
        
        word.to_string()
    }

    /// Remove derivational prefixes
    fn remove_derivational_prefixes(&self, word: &str) -> String {
        let prefixes = [
            "meng", "meny", "men", "mem", "me", "peng", "peny", "pen", "pem", "pe",
            "di", "ter", "ber", "ke", "se"
        ];
        
        for prefix in &prefixes {
            if word.starts_with(prefix) {
                let base = &word[prefix.len()..];
                if self.is_in_dictionary(base) {
                    return base.to_string();
                }
            }
        }
        
        word.to_string()
    }
}

impl Default for Stemmer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to stem a word without a dictionary
pub fn stem(word: &str) -> String {
    let stemmer = Stemmer::new();
    stemmer.stem(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stemmer_creation() {
        let stemmer = Stemmer::new();
        assert_eq!(stemmer.stem("makan"), "makan");
    }

    #[test]
    fn test_with_dictionary() {
        let words = vec!["makan".to_string(), "minum".to_string()];
        let stemmer = Stemmer::with_dictionary(words);
        assert_eq!(stemmer.stem("makan"), "makan");
    }

    #[test]
    fn test_add_word() {
        let mut stemmer = Stemmer::new();
        stemmer.add_word("makan");
        assert_eq!(stemmer.stem("makan"), "makan");
    }

    #[test]
    fn test_inflectional_suffixes() {
        let mut stemmer = Stemmer::new();
        stemmer.add_word("makan");
        assert_eq!(stemmer.stem("makanlah"), "makan");
    }

    #[test]
    fn test_case_insensitive() {
        let mut stemmer = Stemmer::new();
        stemmer.add_word("makan");
        assert_eq!(stemmer.stem("MAKAN"), "makan");
    }

    #[test]
    fn test_convenience_function() {
        assert_eq!(stem("test"), "test");
    }
}
