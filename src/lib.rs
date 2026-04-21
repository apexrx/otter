mod repair_engine;
mod preprocess;

use serde_json::Value;

pub use repair_engine::*;

mod confidence;
pub use confidence::*;

#[cfg(not(target_arch = "wasm32"))]
mod python;
#[cfg(not(target_arch = "wasm32"))]
pub use python::*;

#[cfg(target_arch = "wasm32")]
mod wasm;

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
    let cleaned_input = preprocess::sanitize_json(initial_input);

    let res = repair(&cleaned_input, schema);
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
