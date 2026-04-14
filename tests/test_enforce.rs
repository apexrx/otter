use otter::{enforce, EnforcementResult};
use serde_json::json;

fn minimal_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" }
        },
        "required": ["name", "age"]
    })
}

fn variant_name(r: &EnforcementResult) -> &'static str {
    match r {
        EnforcementResult::Valid { .. } => "Valid",
        EnforcementResult::Repaired { .. } => "Repaired",
        EnforcementResult::NeedsCorrection { .. } => "NeedsCorrection",
        EnforcementResult::InvalidSchema { .. } => "InvalidSchema",
    }
}

#[test]
fn enforce_valid_json_returns_valid() {
    let input = r#"{"name": "Alice", "age": 30}"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::Valid { json } => {
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Alice");
            assert_eq!(v["age"], 30);
        }
        other => panic!("Expected Valid, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_repairable_single_quotes() {
    let input = r#"{'name': 'Bob', 'age': 25}"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("FixSingleQuotes")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Bob");
            assert_eq!(v["age"], 25);
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_trailing_commas() {
    let input = r#"{"name": "Carol", "age": 28,}"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("FixTrailingCommas")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Carol");
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_python_booleans() {
    let schema = json!({
        "type": "object",
        "properties": {
            "active": { "type": "boolean" }
        },
        "required": ["active"]
    });
    let input = r#"{"active": True}"#;

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("FixPythonBooleans")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["active"], true);
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_markdown_fences() {
    let input = r#"```json
{"name": "Dave", "age": 40}
```"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("StripMarkdownFences")));
            assert!(!json.contains("```"));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Dave");
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_markdown_without_json_lang_tag() {
    let input = r#"```
{"name": "Dave", "age": 40}
```"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        // strip_markdown_fences matches ```...``` regardless of lang tag
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("StripMarkdownFences")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Dave");
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_schema_violation_needs_correction() {
    // valid JSON but missing required "age"
    let input = r#"{"name": "Eve"}"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::NeedsCorrection { prompt } => {
            assert!(!prompt.is_empty());
            assert!(prompt.contains("schema"));
        }
        other => panic!("Expected NeedsCorrection, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_invalid_schema() {
    let input = r#"{"x": 1}"#;
    let bad_schema = json!({
        "type": "object",
        "properties": {
            "x": { "type": "not_a_real_type" }
        }
    });

    match enforce(input, &bad_schema) {
        EnforcementResult::InvalidSchema { err } => {
            assert!(!err.is_empty());
            assert!(err.contains("Invalid schema"));
        }
        other => panic!("Expected InvalidSchema, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_unquoted_keys() {
    let input = r#"{name: "Frank", age: 33}"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("FixUnquotedKeys")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Frank");
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_truncated_json() {
    let input = r#"{"name": "Grace", "age": 29"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("FixTruncatedJson")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Grace");
            assert_eq!(v["age"], 29);
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_multiple_repairs_chain() {
    // markdown + single quotes + trailing comma
    let input = r#"```
{'name': 'Hank', 'age': 50,}
```"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.len() >= 3);
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Hank");
            assert_eq!(v["age"], 50);
        }
        other => panic!("Expected Repaired with multiple rules, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_python_none_to_null() {
    let schema = json!({
        "type": "object",
        "properties": {
            "value": { "type": ["number", "null"] }
        },
        "required": ["value"]
    });
    let input = r#"{"value": None}"#;

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("FixPythonBooleans")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert!(v["value"].is_null());
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_numeric_type_repair() {
    let schema = json!({
        "type": "object",
        "properties": {
            "count": { "type": "number" }
        },
        "required": ["count"]
    });
    // string that can be parsed as number — apply_schema_repairs fixes it
    let input = r#"{"count": "42"}"#;

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("FixWrongNumericTypes")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert!(v["count"].is_number());
            assert_eq!(v["count"].as_f64().unwrap(), 42.0);
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_null_value_default_repair() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer", "default": 0 }
        },
        "required": ["name", "age"]
    });
    let input = r#"{"name": "Test", "age": null}"#;

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("FixNullValues")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["age"], 0);
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_nested_array_repairs() {
    let schema = json!({
        "type": "object",
        "properties": {
            "items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "score": { "type": "number" }
                    },
                    "required": ["score"]
                }
            }
        },
        "required": ["items"]
    });
    let input = r#"{"items": [{"score": "99.5"}, {"score": "88"}]}"#;

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("FixWrongNumericTypes")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["items"][0]["score"].as_f64().unwrap(), 99.5);
            assert_eq!(v["items"][1]["score"].as_f64().unwrap(), 88.0);
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_extract_json_from_surrounding_text() {
    let input = r#"Here is the data: {"name": "Ivy", "age": 22}. Hope it helps!"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("ExtractJsonPayload")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Ivy");
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_valid_unchanged_when_no_repairs() {
    let input = r#"{"name": "Jack", "age": 45}"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::Valid { json } => {
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Jack");
            assert_eq!(v["age"], 45);
        }
        other => panic!("Expected Valid, got {}", variant_name(&other)),
    }
}

#[test]
fn enforce_json_with_extra_text_outside_braces() {
    // input where first `{` and last `}` contain valid JSON
    let input = r#"Sure! {"name": "Kate", "age": 35} done"#;
    let schema = minimal_schema();

    match enforce(input, &schema) {
        EnforcementResult::Repaired { json, rules_applied } => {
            assert!(rules_applied.iter().any(|r| format!("{:?}", r).contains("ExtractJsonPayload")));
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v["name"], "Kate");
            assert_eq!(v["age"], 35);
        }
        other => panic!("Expected Repaired, got {}", variant_name(&other)),
    }
}
