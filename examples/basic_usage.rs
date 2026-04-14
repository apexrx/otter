use otter::{enforce, EnforcementResult};
use serde_json::Value;

fn call_llm(prompt: &str) -> String {
    // own API call — openai, ollama, whatever
    todo!()
}

fn run_with_correction(initial_input: &str, schema: &Value, max_retries: u32) -> Option<String> {
    let mut current_input = initial_input.to_string();

    for attempt in 0..max_retries {
        match enforce(&current_input, schema) {
            EnforcementResult::Valid { json } => {
                println!("Valid after {} attempt(s)", attempt + 1);
                return Some(json);
            }
            EnforcementResult::Repaired { json, rules_applied } => {
                println!("Auto-repaired ({} rules applied)", rules_applied.len());
                return Some(json);
            }
            EnforcementResult::NeedsCorrection { prompt } => {
                println!("Attempt {}/{} — sending correction prompt to LLM", attempt + 1, max_retries);
                current_input = call_llm(&prompt); // feed LLM output back in next iteration
            }
            EnforcementResult::InvalidSchema { err } => {
                // developer error — no point retrying
                eprintln!("Schema is broken, aborting: {err}");
                return None;
            }
        }
    }

    eprintln!("Exhausted {max_retries} retries without valid output");
    None
}

fn main() {
    todo!()
}
