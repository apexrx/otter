#[cfg(not(target_arch = "wasm32"))]
use crate::repair_engine::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::{EnforcementResult, enforce};
#[cfg(not(target_arch = "wasm32"))]
use pyo3::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[pyclass(skip_from_py_object)]
pub struct PyRepairRule {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub description: Option<String>,
    #[pyo3(get)]
    pub cost: f32,
}

impl From<RepairRule> for PyRepairRule {
    fn from(rule: RepairRule) -> Self {
        match rule {
            RepairRule::Custom {
                name,
                description,
                cost,
            } => PyRepairRule {
                name,
                description: Some(description),
                cost,
            },
            other => PyRepairRule {
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

#[derive(Debug, Clone)]
#[pyclass(skip_from_py_object)]
pub struct PyRepairResult {
    #[pyo3(get)]
    pub repaired: String,
    #[pyo3(get)]
    pub rules: Vec<PyRepairRule>,
    #[pyo3(get)]
    pub confidence_level: f32,
}

impl From<&RepairResult> for PyRepairResult {
    fn from(res: &RepairResult) -> Self {
        PyRepairResult {
            repaired: res.repaired.clone(),
            rules: res
                .rule
                .clone()
                .into_iter()
                .map(PyRepairRule::from)
                .collect(),
            confidence_level: res.confidence_level,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[pyclass(skip_from_py_object)]
pub struct PySchemaViolation {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub message: String,
    #[pyo3(get)]
    pub invalid_value: Option<String>,
}

impl From<&SchemaViolation> for PySchemaViolation {
    fn from(v: &SchemaViolation) -> Self {
        PySchemaViolation {
            path: v.path.clone(),
            message: v.message.clone(),
            invalid_value: v.invalid_value.as_ref().map(|val| val.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[pyclass(skip_from_py_object)]
pub enum ValidationStatus {
    Valid,
    ParseError,
    SchemaErrors,
    InvalidSchema,
}

impl From<&crate::repair_engine::ValidationReport> for ValidationStatus {
    fn from(report: &crate::repair_engine::ValidationReport) -> Self {
        match report {
            crate::repair_engine::ValidationReport::Valid { .. } => ValidationStatus::Valid,
            crate::repair_engine::ValidationReport::ParseError(_) => ValidationStatus::ParseError,
            crate::repair_engine::ValidationReport::SchemaErrors { .. } => {
                ValidationStatus::SchemaErrors
            }
            crate::repair_engine::ValidationReport::InvalidSchema { .. } => {
                ValidationStatus::InvalidSchema
            }
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(skip_from_py_object)]
pub struct PyValidationReport {
    #[pyo3(get)]
    pub status: ValidationStatus,
    #[pyo3(get)]
    pub parsed: Option<String>,
    #[pyo3(get)]
    pub parse_error: Option<PyParseErrorInfo>,
    #[pyo3(get)]
    pub schema_violations: Option<Vec<PySchemaViolation>>,
    #[pyo3(get)]
    pub invalid_schema_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[pyclass(skip_from_py_object)]
pub struct PyParseErrorInfo {
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub column: usize,
    #[pyo3(get)]
    pub message: String,
}

impl From<&ParseErrorInfo> for PyParseErrorInfo {
    fn from(err: &ParseErrorInfo) -> Self {
        PyParseErrorInfo {
            line: err.line,
            column: err.column,
            message: err.message.clone(),
        }
    }
}

impl From<&crate::repair_engine::ValidationReport> for PyValidationReport {
    fn from(report: &crate::repair_engine::ValidationReport) -> Self {
        match report {
            crate::repair_engine::ValidationReport::Valid { parsed } => PyValidationReport {
                status: ValidationStatus::Valid,
                parsed: Some(parsed.to_string()),
                parse_error: None,
                schema_violations: None,
                invalid_schema_message: None,
            },
            crate::repair_engine::ValidationReport::ParseError(err) => PyValidationReport {
                status: ValidationStatus::ParseError,
                parsed: None,
                parse_error: Some(PyParseErrorInfo::from(err)),
                schema_violations: None,
                invalid_schema_message: None,
            },
            crate::repair_engine::ValidationReport::SchemaErrors { violations } => {
                PyValidationReport {
                    status: ValidationStatus::SchemaErrors,
                    parsed: None,
                    parse_error: None,
                    schema_violations: Some(
                        violations.iter().map(PySchemaViolation::from).collect(),
                    ),
                    invalid_schema_message: None,
                }
            }
            crate::repair_engine::ValidationReport::InvalidSchema { message } => {
                PyValidationReport {
                    status: ValidationStatus::InvalidSchema,
                    parsed: None,
                    parse_error: None,
                    schema_violations: None,
                    invalid_schema_message: Some(message.clone()),
                }
            }
        }
    }
}

#[pyclass]
pub struct PyEnforcementResult {
    #[pyo3(get)]
    pub status: String,

    #[pyo3(get)]
    pub json: Option<String>,

    #[pyo3(get)]
    pub rules_applied: Option<Vec<PyRepairRule>>,

    #[pyo3(get)]
    pub prompt: Option<String>,

    #[pyo3(get)]
    pub error: Option<String>,
}

impl From<EnforcementResult> for PyEnforcementResult {
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
                rules_applied: Some(rules_applied.into_iter().map(PyRepairRule::from).collect()),
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

#[pyfunction]
pub fn enforce_py(input: &str, schema: &str) -> PyResult<PyEnforcementResult> {
    let schema_val = serde_json::from_str(schema)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    Ok(enforce(input, &schema_val).into())
}

#[pyfunction]
pub fn validate_py(output: &str, schema: &str) -> PyResult<PyValidationReport> {
    let schema_val = serde_json::from_str(schema)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let report = validate(output, &schema_val);
    Ok((&report).into())
}

#[pyfunction]
pub fn generate_prompt_py(output: &str, schema: &str) -> PyResult<String> {
    let schema_val = serde_json::from_str(schema)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let report = validate(output, &schema_val);
    generate_correction_prompt(&report, &schema_val)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
}

#[pyfunction]
pub fn repair_py(input: &str, schema: &str) -> PyResult<PyRepairResult> {
    let schema_val = serde_json::from_str(schema)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let repaired = repair(input, &schema_val);
    Ok((&repaired).into())
}

#[pymodule]
fn otter(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(enforce_py, m)?)?;
    m.add_function(wrap_pyfunction!(validate_py, m)?)?;
    m.add_function(wrap_pyfunction!(generate_prompt_py, m)?)?;
    m.add_function(wrap_pyfunction!(repair_py, m)?)?;
    m.add_class::<ValidationStatus>()?;
    Ok(())
}
