use otter::{repair, validate, ValidationReport};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

#[derive(Debug, Deserialize)]
struct CorpusEntry {
    name: String,
    description: String,
    schema: Value,
    input: String,
    expected_repair: String,
    should_repair: bool,
}

struct FailureInfo {
    name: String,
    description: String,
    input: String,
    expected: String,
    actual: String,
    is_valid_json: bool,
    passes_schema: bool,
    matches_expected: bool,
}

fn load_corpus() -> Vec<CorpusEntry> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{manifest_dir}/tests/corpus.json");
    let data = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read corpus file at {path}: {e}"));
    serde_json::from_str(&data)
        .unwrap_or_else(|e| panic!("Failed to parse corpus JSON: {e}\nData preview: {}", &data[..200.min(data.len())]))
}

/// Normalize whitespace in JSON strings for comparison.
/// serde_json::to_string produces compact output; we normalize both sides.
fn normalize_json(s: &str) -> String {
    match serde_json::from_str::<Value>(s) {
        Ok(val) => serde_json::to_string(&val).unwrap_or_else(|_| s.to_string()),
        Err(_) => s.to_string(),
    }
}

#[test]
fn corpus_repair_all_cases() {
    let corpus = load_corpus();
    assert!(corpus.len() >= 50, "Corpus must contain at least 50 entries, got {}", corpus.len());

    let mut passed = 0usize;
    let mut failed = Vec::<FailureInfo>::new();

    for entry in &corpus {
        let result = repair(&entry.input, &entry.schema);

        // Check that the repaired output is valid JSON
        let is_valid_json = serde_json::from_str::<Value>(&result.repaired).is_ok();

        // Check that the repaired output passes schema validation
        let passes_schema = matches!(
            validate(&result.repaired, &entry.schema),
            ValidationReport::Valid { .. }
        );

        // For cases that should repair, verify the result matches expected
        if entry.should_repair {
            let normalized_actual = normalize_json(&result.repaired);
            let normalized_expected = normalize_json(&entry.expected_repair);

            if is_valid_json && passes_schema && normalized_actual == normalized_expected {
                passed += 1;
            } else {
                failed.push(FailureInfo {
                    name: entry.name.clone(),
                    description: entry.description.clone(),
                    input: entry.input.clone(),
                    expected: entry.expected_repair.clone(),
                    actual: result.repaired.clone(),
                    is_valid_json: is_valid_json,
                    passes_schema,
                    matches_expected: normalized_actual == normalized_expected,
                });
            }
        }
    }

    let total = passed + failed.len();
    let success_rate = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    if !failed.is_empty() {
        eprintln!("\n=== FAILED REPAIRS ({}/{}) ===", failed.len(), total);
        for f in &failed {
            eprintln!("\n--- {} ---", f.name);
            eprintln!("  Description: {}", f.description);
            eprintln!("  Input:       {}", f.input);
            eprintln!("  Expected:    {}", f.expected);
            eprintln!("  Actual:      {}", f.actual);
            eprintln!("  Valid JSON:  {}", f.is_valid_json);
            eprintln!("  Schema OK:   {}", f.passes_schema);
            eprintln!("  Matches:     {}", f.matches_expected);
        }
    }

    assert!(
        success_rate >= 80.0,
        "Repair success rate is {success_rate:.1}% ({passed}/{total}), must be >= 80%"
    );

    eprintln!(
        "Corpus repair rate: {success_rate:.1}% ({passed}/{total} cases passed)"
    );
}

/// Verify that already-valid inputs are not corrupted by the repair engine.
/// "Corruption" means the semantic value changed when no repair was needed.
/// Note: schema-level repairs (null→default, string→number) are intentional
/// repairs, not corruption. This test only checks cases where the input is
/// already valid JSON AND passes schema validation without any repairs.
#[test]
fn corpus_no_corruption_on_valid_inputs() {
    let corpus = load_corpus();

    for entry in &corpus {
        // If the raw input is already valid JSON AND passes schema validation,
        // the repair engine should leave it semantically unchanged.
        if serde_json::from_str::<Value>(&entry.input).is_ok()
            && matches!(validate(&entry.input, &entry.schema), ValidationReport::Valid { .. })
        {
            let result = repair(&entry.input, &entry.schema);

            let actual_val: Value = serde_json::from_str(&result.repaired)
                .unwrap_or_else(|e| {
                    panic!("Repair corrupted valid input '{}' in '{}': {e}", entry.name, result.repaired)
                });

            let expected_val: Value = serde_json::from_str(&entry.input)
                .expect("Input was confirmed valid above");

            assert_eq!(
                actual_val, expected_val,
                "Repair corrupted valid input in '{}': expected {:?}, got {:?}",
                entry.name, expected_val, actual_val
            );
        }
    }
}
