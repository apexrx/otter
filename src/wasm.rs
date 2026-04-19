use crate::repair_engine::*;
use crate::{EnforcementResult, enforce};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmRepairRule {
    pub name: String,
    pub description: Option<String>,
    pub cost: f32,
}

impl From<RepairRule> for WasmRepairRule {
    fn from(rule: RepairRule) -> Self {
        match rule {
            RepairRule::Custom {
                name,
                description,
                cost,
            } => WasmRepairRule {
                name,
                description: Some(description),
                cost,
            },
            other => WasmRepairRule {
                name: format!("{:?}", other)
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        if i > 0 && c.is_uppercase() {
                            format!("_{}", c.to_lowercase())
                        } else {
                            c.to_lowercase().to_string()
                        }
                    })
                    .collect(),
                description: None,
                cost: other.cost(),
            },
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmRepairResult {
    pub repaired: String,
    pub rules: Vec<WasmRepairRule>,
    pub confidence_level: f32,
}

impl From<&RepairResult> for WasmRepairResult {
    fn from(res: &RepairResult) -> Self {
        WasmRepairResult {
            repaired: res.repaired.clone(),
            rules: res.rule.clone().into_iter().map(WasmRepairRule::from).collect(),
            confidence_level: res.confidence_level,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmSchemaViolation {
    pub path: String,
    pub message: String,
    pub invalid_value: Option<String>,
}

impl From<&SchemaViolation> for WasmSchemaViolation {
    fn from(v: &SchemaViolation) -> Self {
        WasmSchemaViolation {
            path: v.path.clone(),
            message: v.message.clone(),
            invalid_value: v.invalid_value.as_ref().map(|val| val.to_string()),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub enum WasmValidationStatus {
    Valid,
    ParseError,
    SchemaErrors,
    InvalidSchema,
}

impl From<&ValidationReport> for WasmValidationStatus {
    fn from(report: &ValidationReport) -> Self {
        match report {
            ValidationReport::Valid { .. } => WasmValidationStatus::Valid,
            ValidationReport::ParseError(_) => WasmValidationStatus::ParseError,
            ValidationReport::SchemaErrors { .. } => WasmValidationStatus::SchemaErrors,
            ValidationReport::InvalidSchema { .. } => WasmValidationStatus::InvalidSchema,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmValidationReport {
    pub status: WasmValidationStatus,
    pub parsed: Option<String>,
    pub parse_error: Option<WasmParseErrorInfo>,
    pub schema_violations: Option<Vec<WasmSchemaViolation>>,
    pub invalid_schema_message: Option<String>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmParseErrorInfo {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl From<&ParseErrorInfo> for WasmParseErrorInfo {
    fn from(err: &ParseErrorInfo) -> Self {
        WasmParseErrorInfo {
            line: err.line,
            column: err.column,
            message: err.message.clone(),
        }
    }
}

impl From<&ValidationReport> for WasmValidationReport {
    fn from(report: &ValidationReport) -> Self {
        match report {
            ValidationReport::Valid { parsed } => WasmValidationReport {
                status: WasmValidationStatus::Valid,
                parsed: Some(parsed.to_string()),
                parse_error: None,
                schema_violations: None,
                invalid_schema_message: None,
            },
            ValidationReport::ParseError(err) => WasmValidationReport {
                status: WasmValidationStatus::ParseError,
                parsed: None,
                parse_error: Some(WasmParseErrorInfo::from(err)),
                schema_violations: None,
                invalid_schema_message: None,
            },
            ValidationReport::SchemaErrors { violations } => WasmValidationReport {
                status: WasmValidationStatus::SchemaErrors,
                parsed: None,
                parse_error: None,
                schema_violations: Some(
                    violations.iter().map(WasmSchemaViolation::from).collect(),
                ),
                invalid_schema_message: None,
            },
            ValidationReport::InvalidSchema { message } => WasmValidationReport {
                status: WasmValidationStatus::InvalidSchema,
                parsed: None,
                parse_error: None,
                schema_violations: None,
                invalid_schema_message: Some(message.clone()),
            },
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmEnforcementResult {
    pub status: String,
    pub json: Option<String>,
    pub rules_applied: Option<Vec<WasmRepairRule>>,
    pub prompt: Option<String>,
    pub error: Option<String>,
}

impl From<EnforcementResult> for WasmEnforcementResult {
    fn from(res: EnforcementResult) -> Self {
        match res {
            EnforcementResult::Valid { json } => Self {
                status: "Valid".to_string(),
                json: Some(json),
                rules_applied: None,
                prompt: None,
                error: None,
            },
            EnforcementResult::Repaired {
                json,
                rules_applied,
            } => Self {
                status: "Repaired".to_string(),
                json: Some(json),
                rules_applied: Some(rules_applied.into_iter().map(WasmRepairRule::from).collect()),
                prompt: None,
                error: None,
            },
            EnforcementResult::NeedsCorrection { prompt } => Self {
                status: "NeedsCorrection".to_string(),
                json: None,
                rules_applied: None,
                prompt: Some(prompt),
                error: None,
            },
            EnforcementResult::InvalidSchema { err } => Self {
                status: "InvalidSchema".to_string(),
                json: None,
                rules_applied: None,
                prompt: None,
                error: Some(err),
            },
        }
    }
}

#[wasm_bindgen]
pub fn enforce_wasm(input: &str, schema: &str) -> Result<WasmEnforcementResult, JsValue> {
    let schema_val: serde_json::Value = serde_json::from_str(schema)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(enforce(input, &schema_val).into())
}

#[wasm_bindgen]
pub fn validate_wasm(output: &str, schema: &str) -> Result<WasmValidationReport, JsValue> {
    let schema_val: serde_json::Value = serde_json::from_str(schema)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let report = validate(output, &schema_val);
    Ok((&report).into())
}

#[wasm_bindgen]
pub fn generate_prompt_wasm(output: &str, schema: &str) -> Result<String, JsValue> {
    let schema_val: serde_json::Value = serde_json::from_str(schema)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let report = validate(output, &schema_val);
    generate_correction_prompt(&report, &schema_val)
        .map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn repair_wasm(input: &str, schema: &str) -> Result<WasmRepairResult, JsValue> {
    let schema_val: serde_json::Value = serde_json::from_str(schema)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let repaired = repair(input, &schema_val);
    Ok((&repaired).into())
}
