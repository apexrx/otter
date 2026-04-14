mod repair_engine;

pub use repair_engine::*;

mod confidence;
pub use confidence::*;

use serde_json::Value;

pub enum EnforcementResult {
    Valid { json: String },
    Repaired { json: String, rules_applied: Vec<RepairRule> },
    NeedsCorrection { prompt: String },
    InvalidSchema { err: String }
}

pub fn enforce(initial_input: &str, schema: &Value) -> EnforcementResult {
    let res = repair(initial_input, schema);
    let repaired = res.repaired;
    let report = validate(&repaired, schema);

    match generate_correction_prompt(&report, schema) {
        Err(err) => EnforcementResult::InvalidSchema { err: err.to_string() },
        Ok(prompt) if prompt.is_empty() => {
            if repaired == initial_input {
                EnforcementResult::Valid { json: repaired }
            } else {
                EnforcementResult::Repaired { json: repaired, rules_applied: res.rule }
            }
        }
        Ok(prompt) => EnforcementResult::NeedsCorrection { prompt },
    }
}
