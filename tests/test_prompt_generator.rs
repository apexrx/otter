use otter::*;
use serde_json::{json, Value};

fn generate_and_self_correct(
    label: &str,
    failing_output: &str,
    schema: &Value,
) -> (String, String, ValidationReport, ValidationReport) {
    let original_report = validate(failing_output, schema);

    let prompt = generate_correction_prompt(&original_report, schema)
        .expect("prompt generation should succeed");

    let repair_result = repair(failing_output, schema);

    let repaired_report = validate(&repair_result.repaired, schema);

    println!("Case: {label}");
    println!("Original output:\n{failing_output}");
    println!("Generated correction prompt:\n{prompt}");
    println!("Repaired output:\n{}", repair_result.repaired);
    println!("Original report: {:?}", original_report);
    println!("Repaired report: {:?}", repaired_report);
    println!("Applied rules:   {:?}", repair_result.rule);
    println!("Confidence:      {:.2}\n", repair_result.confidence_level);

    (
        prompt,
        repair_result.repaired,
        original_report,
        repaired_report,
    )
}

#[test]
fn case_01_missing_required_field() {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "user_id": { "type": "integer" },
            "username": { "type": "string" },
            "email": { "type": "string" }
        },
        "required": ["user_id", "username", "email"]
    });

    let broken = r#"{"user_id": 42, "username": "otter_fan"}"#;

    let (prompt, repaired, orig_report, repaired_report) =
        generate_and_self_correct("Missing required field 'email'", broken, &schema);

    assert!(prompt.contains("email"), "Prompt should mention the missing 'email' field");
    assert!(prompt.contains("email"), "Prompt should contain 'email' in the violation message");
    assert!(prompt.contains("required"));

    drop((repaired, orig_report, repaired_report));
}

#[test]
fn case_02_wrong_type_integer_got_string() {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "count": { "type": "integer" }
        },
        "required": ["count"]
    });

    let broken = r#"{"count": "five"}"#;

    let (prompt, repaired, orig_report, repaired_report) =
        generate_and_self_correct("Wrong type: string instead of integer", broken, &schema);

    assert!(prompt.contains("/count"));
    assert!(prompt.contains("integer"));
    assert!(prompt.contains("expected"));
    assert!(prompt.contains("expected: integer"));
    drop((repaired, orig_report, repaired_report));
}

#[test]
fn case_03_wrong_type_string_got_number() {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "required": ["name"]
    });

    let broken = r#"{"name": 12345}"#;

    let (prompt, repaired, orig_report, repaired_report) =
        generate_and_self_correct("Wrong type: number instead of string", broken, &schema);

    assert!(prompt.contains("/name"));
    assert!(prompt.contains("expected: string"));

    drop((repaired, orig_report, repaired_report));
}

#[test]
fn case_04_nested_object_missing_field() {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "address": {
                "type": "object",
                "properties": {
                    "street": { "type": "string" },
                    "city": { "type": "string" },
                    "zip": { "type": "string" }
                },
                "required": ["street", "city", "zip"]
            }
        },
        "required": ["address"]
    });

    let broken = r#"{"address": {"street": "123 Main St", "city": "Springfield"}}"#;

    let (prompt, repaired, orig_report, repaired_report) =
        generate_and_self_correct("Nested object missing 'zip'", broken, &schema);

    assert!(prompt.contains("zip"), "Prompt should mention the missing 'zip' field");
    assert!(prompt.contains("required"));

    drop((repaired, orig_report, repaired_report));
}

#[test]
fn case_05_array_item_schema_violation() {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "tags": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["tags"]
    });

    let broken = r#"{"tags": ["rust", 42, "otter"]}"#;

    let (prompt, repaired, orig_report, repaired_report) =
        generate_and_self_correct("Array item wrong type (number instead of string)", broken, &schema);

    assert!(prompt.contains("/tags/1"));
    assert!(prompt.contains("expected: string"));

    drop((repaired, orig_report, repaired_report));
}

#[test]
fn case_06_multiple_violations() {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "id": { "type": "integer" },
            "name": { "type": "string" },
            "active": { "type": "boolean" },
            "score": { "type": "number" }
        },
        "required": ["id", "name", "active", "score"]
    });

    let broken = r#"{"id": "100", "name": "test", "active": "yes", "score": "9.5"}"#;

    let (prompt, repaired, orig_report, repaired_report) =
        generate_and_self_correct("Multiple violations: wrong types for id, active, score", broken, &schema);

    assert!(prompt.contains("/id"));
    assert!(prompt.contains("/active"));
    assert!(prompt.contains("/score"));
    assert!(prompt.contains("expected: integer"));
    assert!(prompt.contains("expected: boolean"));
    assert!(prompt.contains("expected: number"));

    drop((repaired, orig_report, repaired_report));
}

#[test]
fn case_07_enum_violation() {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "status": {
                "type": "string",
                "enum": ["pending", "active", "closed"]
            }
        },
        "required": ["status"]
    });

    let broken = r#"{"status": "in_progress"}"#;

    let (prompt, repaired, orig_report, repaired_report) =
        generate_and_self_correct("Enum violation: invalid status value", broken, &schema);

    assert!(prompt.contains("/status"));
    assert!(prompt.contains("enum"));

    drop((repaired, orig_report, repaired_report));
}

#[test]
fn case_08_minimum_violation() {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "age": {
                "type": "integer",
                "minimum": 0
            }
        },
        "required": ["age"]
    });

    let broken = r#"{"age": -5}"#;

    let (prompt, repaired, orig_report, repaired_report) =
        generate_and_self_correct("Minimum violation: negative age", broken, &schema);

    assert!(prompt.contains("/age"));
    assert!(prompt.contains("minimum"));

    drop((repaired, orig_report, repaired_report));
}

#[test]
fn case_09_max_length_violation() {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "code": {
                "type": "string",
                "maxLength": 10
            }
        },
        "required": ["code"]
    });

    let broken = r#"{"code": "ABCDEFGHIJKLMNOPQRSTUVWXYZ"}"#;

    let (prompt, repaired, orig_report, repaired_report) =
        generate_and_self_correct("maxLength violation: code too long", broken, &schema);

    assert!(prompt.contains("/code"));
    assert!(prompt.contains("maxLength"));

    drop((repaired, orig_report, repaired_report));
}

#[test]
fn case_10_additional_properties_disallowed() {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "required": ["name"],
        "additionalProperties": false
    });

    let broken = r#"{"name": "otter", "extra_field": "should not be here"}"#;

    let (prompt, repaired, orig_report, repaired_report) =
        generate_and_self_correct("additionalProperties violation", broken, &schema);

    assert!(prompt.contains("extra_field"), "Prompt should mention the disallowed field");
    assert!(prompt.contains("additionalProperties") || prompt.contains("Additional properties"));

    drop((repaired, orig_report, repaired_report));
}

#[test]
fn test_prompt_generator_10_failure_cases() {
    let results = vec![
        run_case_01(),
        run_case_02(),
        run_case_03(),
        run_case_04(),
        run_case_05(),
        run_case_06(),
        run_case_07(),
        run_case_08(),
        run_case_09(),
        run_case_10(),
    ];

    for (label, prompt) in &results {
        assert!(!prompt.is_empty(), "Prompt for '{label}' should not be empty");
        assert!(
            prompt.contains("schema"),
            "Prompt for '{label}' should contain the schema reference"
        );
        assert!(
            prompt.contains("violation"),
            "Prompt for '{label}' should mention 'violation'"
        );
        assert!(
            prompt.contains("Return only valid JSON"),
            "Prompt for '{label}' should instruct to return only JSON"
        );
        assert!(
            prompt.contains("no additional text"),
            "Prompt for '{label}' should say 'no additional text'"
        );
    }

    println!("\nAll 10 prompts are well-formed and contain required instructions.");
}

fn run_case_01() -> (String, String) {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "user_id": { "type": "integer" },
            "username": { "type": "string" },
            "email": { "type": "string" }
        },
        "required": ["user_id", "username", "email"]
    });
    let broken = r#"{"user_id": 42, "username": "otter_fan"}"#;
    let report = validate(broken, &schema);
    let prompt = generate_correction_prompt(&report, &schema).unwrap();
    ("Missing required field".into(), prompt)
}

fn run_case_02() -> (String, String) {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "count": { "type": "integer" }
        },
        "required": ["count"]
    });
    let broken = r#"{"count": "five"}"#;
    let report = validate(broken, &schema);
    let prompt = generate_correction_prompt(&report, &schema).unwrap();
    ("Wrong type: string instead of integer".into(), prompt)
}

fn run_case_03() -> (String, String) {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "required": ["name"]
    });
    let broken = r#"{"name": 12345}"#;
    let report = validate(broken, &schema);
    let prompt = generate_correction_prompt(&report, &schema).unwrap();
    ("Wrong type: number instead of string".into(), prompt)
}

fn run_case_04() -> (String, String) {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "address": {
                "type": "object",
                "properties": {
                    "street": { "type": "string" },
                    "city": { "type": "string" },
                    "zip": { "type": "string" }
                },
                "required": ["street", "city", "zip"]
            }
        },
        "required": ["address"]
    });
    let broken = r#"{"address": {"street": "123 Main St", "city": "Springfield"}}"#;
    let report = validate(broken, &schema);
    let prompt = generate_correction_prompt(&report, &schema).unwrap();
    ("Nested object missing field".into(), prompt)
}

fn run_case_05() -> (String, String) {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "tags": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["tags"]
    });
    let broken = r#"{"tags": ["rust", 42, "otter"]}"#;
    let report = validate(broken, &schema);
    let prompt = generate_correction_prompt(&report, &schema).unwrap();
    ("Array item wrong type".into(), prompt)
}

fn run_case_06() -> (String, String) {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "id": { "type": "integer" },
            "name": { "type": "string" },
            "active": { "type": "boolean" },
            "score": { "type": "number" }
        },
        "required": ["id", "name", "active", "score"]
    });
    let broken = r#"{"id": "100", "name": "test", "active": "yes", "score": "9.5"}"#;
    let report = validate(broken, &schema);
    let prompt = generate_correction_prompt(&report, &schema).unwrap();
    ("Multiple violations".into(), prompt)
}

fn run_case_07() -> (String, String) {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "status": {
                "type": "string",
                "enum": ["pending", "active", "closed"]
            }
        },
        "required": ["status"]
    });
    let broken = r#"{"status": "in_progress"}"#;
    let report = validate(broken, &schema);
    let prompt = generate_correction_prompt(&report, &schema).unwrap();
    ("Enum violation".into(), prompt)
}

fn run_case_08() -> (String, String) {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "age": {
                "type": "integer",
                "minimum": 0
            }
        },
        "required": ["age"]
    });
    let broken = r#"{"age": -5}"#;
    let report = validate(broken, &schema);
    let prompt = generate_correction_prompt(&report, &schema).unwrap();
    ("Minimum violation".into(), prompt)
}

fn run_case_09() -> (String, String) {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "code": {
                "type": "string",
                "maxLength": 10
            }
        },
        "required": ["code"]
    });
    let broken = r#"{"code": "ABCDEFGHIJKLMNOPQRSTUVWXYZ"}"#;
    let report = validate(broken, &schema);
    let prompt = generate_correction_prompt(&report, &schema).unwrap();
    ("maxLength violation".into(), prompt)
}

fn run_case_10() -> (String, String) {
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "required": ["name"],
        "additionalProperties": false
    });
    let broken = r#"{"name": "otter", "extra_field": "should not be here"}"#;
    let report = validate(broken, &schema);
    let prompt = generate_correction_prompt(&report, &schema).unwrap();
    ("additionalProperties violation".into(), prompt)
}
