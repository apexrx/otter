// use std::env::current_exe;

use regex::Regex;
use serde_json::Value;
use strum_macros::Display;

use crate::confidence;

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaViolation {
    pub path: String,
    pub message: String,
    pub invalid_value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseErrorInfo {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationReport {
    Valid { parsed: Value },
    ParseError(ParseErrorInfo),
    SchemaErrors { violations: Vec<SchemaViolation> },
    InvalidSchema { message: String },
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum RepairRule {
    StripMarkdownFences,
    ExtractJsonPayload,
    FixTruncatedJson,
    FixTrailingCommas,
    FixSingleQuotes,
    FixUnquotedKeys,
    FixPythonBooleans,
    FixWrongNumericTypes,
    FixNullValues,

    Custom {
        name: String,
        description: String,
        cost: f32,
    },
}

impl RepairRule {
    pub fn cost(&self) -> f32 {
        match self {
            Self::StripMarkdownFences => 0.02,
            Self::ExtractJsonPayload => 0.05,
            Self::FixTrailingCommas => 0.05,
            Self::FixPythonBooleans => 0.08,
            Self::FixSingleQuotes => 0.12,
            Self::FixUnquotedKeys => 0.15,
            Self::FixTruncatedJson => 0.40,
            Self::FixWrongNumericTypes => 0.05,
            Self::FixNullValues => 0.10,

            Self::Custom { cost, .. } => cost.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepairResult {
    pub repaired: String,
    pub rule: Vec<RepairRule>,
    pub confidence_level: f32,
}

pub fn validate(output: &str, schema: &Value) -> ValidationReport {
    let parsed_value = match serde_json::from_str::<Value>(output) {
        Ok(val) => val,
        Err(err) => {
            return ValidationReport::ParseError(ParseErrorInfo {
                line: err.line(),
                column: err.column(),
                message: err.to_string(),
            });
        }
    };

    let mut violations = Vec::new();

    // validation of parsed value against schema and population of violations
    let validator = match jsonschema::validator_for(schema) {
        Ok(v) => v,
        Err(e) => {
            return ValidationReport::InvalidSchema {
                message: e.to_string(),
            };
        }
    };

    for violation in validator.iter_errors(&parsed_value) {
        violations.push(SchemaViolation {
            path: violation.instance_path().to_string(),
            message: violation.to_string(),
            invalid_value: Some(violation.instance().clone().into_owned()),
        });
    }

    if violations.is_empty() {
        ValidationReport::Valid {
            parsed: parsed_value,
        }
    } else {
        ValidationReport::SchemaErrors { violations }
    }
}

pub fn fix_trailing_commas(input: &str) -> Option<String> {
    let re = Regex::new(r",(\s*[}\]])").unwrap();
    let result = re.replace_all(input, "$1");

    if result == input {
        None
    } else {
        Some(result.into_owned())
    }
}

pub fn fix_single_quotes(input: &str) -> Option<String> {
    let re = Regex::new(r"'([^']+)'").unwrap();
    let result = re.replace_all(input, r#""$1""#);

    if result == input {
        None
    } else {
        Some(result.into_owned())
    }
}

pub fn fix_unquoted_keys(input: &str) -> Option<String> {
    let re = Regex::new(r"([{,]\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*:").unwrap();
    let result = re.replace_all(input, r#"$1"$2":"#);

    if result == input {
        None
    } else {
        Some(result.into_owned())
    }
}

pub fn fix_python_booleans(input: &str) -> Option<String> {
    let re_true = Regex::new(r"\bTrue\b").unwrap();
    let re_false = Regex::new(r"\bFalse\b").unwrap();
    let re_none = Regex::new(r"\bNone\b").unwrap();
    let result = re_true.replace_all(input, "true");
    let result = re_false.replace_all(&result, "false");
    let result = re_none.replace_all(&result, "null");

    if result == input {
        None
    } else {
        Some(result.into_owned())
    }
}

pub fn fix_truncated_json(input: &str) -> Option<String> {
    let mut in_string = false;
    let mut is_escaped = false;

    let mut stack = Vec::new();

    for c in input.chars() {
        if is_escaped {
            is_escaped = false;
            continue;
        }

        if c == '\\' {
            is_escaped = true;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            continue;
        }

        if !in_string {
            match c {
                ']' => {
                    if stack.last() == Some(&'[') {
                        stack.pop();
                    }
                }
                '}' => {
                    if stack.last() == Some(&'{') {
                        stack.pop();
                    }
                }
                '[' | '{' => {
                    stack.push(c);
                }
                _ => {}
            }
        }
    }

    let mut repaired = input.to_string();

    if in_string {
        repaired.push('"');
    }

    while let Some(top) = stack.pop() {
        if top == '[' {
            repaired.push(']');
        } else if top == '{' {
            repaired.push('}');
        }
    }

    if repaired == input {
        None
    } else {
        Some(repaired)
    }
}

pub fn strip_markdown_fences(input: &str) -> Option<String> {
    let re = Regex::new(r"```\s*(?:json)?\s*([\s\S]*?)\s*```").unwrap();
    let result = re.replace_all(input, "$1");

    if result == input {
        None
    } else {
        Some(result.into_owned())
    }
}

pub fn extract_json_payload(input: &str) -> Option<String> {
    let first = input.find(&['{', '['][..]);
    let last = input.rfind(&['}', ']'][..]);

    if let (Some(f), Some(l)) = (first, last) {
        if f <= l {
            let actually_trimmed = f > 0 || l + 1 < input.len();

            if actually_trimmed {
                Some(input[f..=l].to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub fn apply_schema_repairs(
    data: &mut serde_json::Value,
    schema: &serde_json::Value,
    applied_rules: &mut Vec<RepairRule>,
) {
    if data.is_string() {
        if let Some(typ) = schema.get("type").and_then(|v| v.as_str()) {
            if typ == "number" || typ == "integer" {
                if let Some(text) = data.as_str() {
                    match text.parse::<f64>() {
                        Ok(val) => {
                            if let Some(num) = serde_json::Number::from_f64(val) {
                                *data = serde_json::Value::Number(num);
                                applied_rules.push(RepairRule::FixWrongNumericTypes);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse '{}' as number: {}", text, e);
                        }
                    }
                }
            }
        }
    } else if data.is_null() {
        if let Some(def) = schema.get("default") {
            *data = def.clone();
            applied_rules.push(RepairRule::FixNullValues);
        }
    } else {
        if let Some(arr) = data.as_array_mut() {
            if let Some(items_schema) = schema.get("items") {
                for elm in arr.iter_mut() {
                    apply_schema_repairs(elm, items_schema, applied_rules);
                }
            }
        }

        if let Some(obj) = data.as_object_mut() {
            if let Some(properties) = schema.get("properties") {
                for (key, value) in obj.iter_mut() {
                    if let Some(prop_schema) = properties.get(key) {
                        apply_schema_repairs(value, prop_schema, applied_rules);
                    }
                }
            }
        }
    }
}

pub fn repair(input: &str, schema: &Value) -> RepairResult {
    let mut current_string = input.to_string();
    let mut applied_rules = Vec::new();

    if let Some(json) = strip_markdown_fences(&current_string) {
        current_string = json;
        applied_rules.push(RepairRule::StripMarkdownFences);
    }

    if let Some(json) = extract_json_payload(&current_string) {
        current_string = json;
        applied_rules.push(RepairRule::ExtractJsonPayload);
    }

    if let Some(json) = fix_truncated_json(&current_string) {
        current_string = json;
        applied_rules.push(RepairRule::FixTruncatedJson);
    }

    if let Some(json) = fix_trailing_commas(&current_string) {
        current_string = json;
        applied_rules.push(RepairRule::FixTrailingCommas);
    }

    if let Some(json) = fix_single_quotes(&current_string) {
        current_string = json;
        applied_rules.push(RepairRule::FixSingleQuotes);
    }

    if let Some(json) = fix_unquoted_keys(&current_string) {
        current_string = json;
        applied_rules.push(RepairRule::FixUnquotedKeys);
    }

    if let Some(json) = fix_python_booleans(&current_string) {
        current_string = json;
        applied_rules.push(RepairRule::FixPythonBooleans);
    }

    // applying schema level repairs
    let json_value: Option<Value> = match serde_json::from_str(&current_string) {
        Ok(value) => Some(value),
        Err(e) => {
            eprintln!("Failed to parse JSON string: {}", e);
            None
        }
    };

    if let Some(mut json_value) = json_value {
        apply_schema_repairs(&mut json_value, schema, &mut applied_rules);
        current_string = serde_json::to_string(&json_value).unwrap_or_default();
    }

    let confidence = confidence::compute_confidence(&applied_rules, &current_string);

    RepairResult {
        repaired: current_string,
        rule: applied_rules,
        confidence_level: confidence,
    }
}

#[must_use]
pub fn generate_correction_prompt(
    report: &ValidationReport,
    schema: &Value,
) -> Result<String, String> {
    match report {
        ValidationReport::Valid { .. } => Ok(String::new()),
        ValidationReport::ParseError(info) => {
            let sanitized = info.message.chars().take(120).collect::<String>();
            Ok(format!(
                "Your previous response was not valid JSON and could not be parsed. \
                 Parse error: {} at line {}, column {}. \
                 Please return only valid JSON with no additional text, markdown, or code fences.",
                sanitized, info.line, info.column
            ))
        }
        ValidationReport::SchemaErrors { violations } => {
            if violations.is_empty() {
                return Ok(String::from(
                    "Your previous response was valid JSON but did not conform to the required schema.\n\
                     Please review the schema and return valid JSON accordingly.",
                ));
            }

            let error_messages: Vec<String> = violations
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let type_hint = extract_type(schema, &v.path)
                        .map(|expected_type| format!(" (expected: {})", expected_type))
                        .unwrap_or_default();

                    format!("{}. At '{}': {}{}", i + 1, v.path, v.message, type_hint)
                })
                .collect();

            Ok(format!(
                "Your previous response was valid JSON but did not conform to the required schema.\n\
                 Please fix the following {} violation(s) and try again:\n{}\n\n\
                 The required schema is:\n{}\n\n\
                 Return only valid JSON that satisfies this schema, \
                 with no additional text, markdown, or code fences.",
                violations.len(),
                error_messages.join("\n"),
                serde_json::to_string_pretty(schema).unwrap_or_else(|_| schema.to_string())
            ))
        }
        ValidationReport::InvalidSchema { message } => Err(format!("Invalid schema: {}", message)),
    }
}

fn get_type_from_value(v: &Value) -> Option<&str> {
    v.get("type").and_then(|t| t.as_str())
}

#[must_use]
pub fn extract_type<'a>(schema: &'a Value, path: &str) -> Option<&'a str> {
    let segments: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let mut current: &'a Value = schema;

    if segments.is_empty() {
        return get_type_from_value(current);
    }

    for segment in &segments {
        if let Some(props) = current.get("properties") {
            if let Some(next) = props.get(segment) {
                current = next;
            } else if let Some(prefix_items) = current.get("prefixItems") {
                if let Ok(idx) = segment.parse::<usize>() {
                    if let Some(next) = prefix_items.get(idx) {
                        current = next;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else if segment.parse::<usize>().is_ok() {
            if let Some(itm) = current.get("items") {
                current = itm;
            } else if let Some(prefix_items) = current.get("prefixItems") {
                current = prefix_items;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    get_type_from_value(current)
}
