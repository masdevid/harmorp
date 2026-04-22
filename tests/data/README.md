# Stemmer Test Data

This directory contains test case collections for the Indonesian stemmer.

## Structure

Each JSON file contains:
- `description`: What this test file covers
- `category`: Test category for organization
- `test_cases`: Verified test cases with expected outputs
- `pending_from_scraper`: Words to be verified when scraper data is available

## Files

| File | Coverage | Current Cases | Pending |
|------|----------|---------------|---------|
| `inflectional_suffixes.json` | -lah, -kah, -pun, -tah | 6 | 5 |
| `derivational_suffixes.json` | -kan, -i, -an | 8 | 10 |
| `me_prefixes.json` | me-, men-, mem-, meng-, meny- | 10 | 12 |
| `pe_prefixes.json` | pe-, pem-, pen-, peng-, peny- | 8 | 10 |
| `be_te_se_prefixes.json` | be-, te-, se- | 9 | 16 |
| `complex_forms.json` | Multiple affix combinations | 8 | 17 |
| `edge_cases.json` | Root words, short words | 10 | 15 |

## Adding Test Cases

To add new verified test cases:

1. Verify the word and its expected root form using linguistic resources
2. Add to `test_cases` section with:
   - `word`: The inflected/derived form
   - `expected`: The expected stem
   - `rule`: Brief rule description (optional)
   - `notes`: Additional context (optional)
3. Remove from `pending_from_scraper` list

## Test Case Format

```json
{
  "word": "membaca",
  "expected": "baca",
  "rule": "mem + b → b",
  "notes": "Labial assimilation rule"
}
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific category
cargo test test_me_prefixes

# See pending words summary
cargo test print_pending_summary -- --ignored
```
