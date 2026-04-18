mod repair_engine;

use pyo3::prelude::*;
use pyo3::{pyfunction, wrap_pyfunction};
use serde_json::Value;

pub use repair_engine::*;

mod confidence;
pub use confidence::*;

mod python;
pub use python::*;

#[derive(Debug)]
pub enum EnforcementResult {
    Valid {
        json: String,
    },
    Repaired {
        json: String,
        rules_applied: Vec<RepairRule>,
    },
    NeedsCorrection {
        prompt: String,
    },
    InvalidSchema {
        err: String,
    },
}

pub fn enforce(initial_input: &str, schema: &Value) -> EnforcementResult {
    let res = repair(initial_input, schema);
    let repaired = res.repaired;
    let report = validate(&repaired, schema);

    match generate_correction_prompt(&report, schema) {
        Err(err) => EnforcementResult::InvalidSchema {
            err: err.to_string(),
        },
        Ok(prompt) if prompt.is_empty() => {
            if res.rule.is_empty() {
                EnforcementResult::Valid { json: repaired }
            } else {
                EnforcementResult::Repaired {
                    json: repaired,
                    rules_applied: res.rule,
                }
            }
        }
        Ok(prompt) => EnforcementResult::NeedsCorrection { prompt },
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
