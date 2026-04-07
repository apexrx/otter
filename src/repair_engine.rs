// use std::env::current_exe;

use serde_json::Value;
use regex::Regex;

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

#[derive(Debug, Clone, PartialEq)]
pub enum RepairRule {
    StripMarkdownFences,
    ExtractJsonPayload,
    FixTruncatedJson,
    FixTrailingCommas,
    FixSingleQuotes,
    FixUnquotedKeys,
    FixPythonBooleans,

    Custom { name: String, description: String, cost: f32 },
}

impl RepairRule {
    pub fn cost(&self) -> f32 {
        match self {
            Self::StripMarkdownFences  => 0.02,
            Self::ExtractJsonPayload   => 0.05,
            Self::FixTrailingCommas    => 0.05,
            Self::FixPythonBooleans    => 0.08,
            Self::FixSingleQuotes      => 0.12,
            Self::FixUnquotedKeys      => 0.15,
            Self::FixTruncatedJson     => 0.40,

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

pub fn repair(input: &str) -> RepairResult {
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

    let confidence = confidence::compute_confidence(&applied_rules, &current_string);

    RepairResult {
        repaired: current_string,
        rule: applied_rules,
        confidence_level: confidence,
    }
}
