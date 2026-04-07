use otter::compute_confidence;
use otter::RepairRule;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_rules_applied_is_full_confidence() {
        let valid = r#"{"key": "value"}"#;
        assert_eq!(compute_confidence(&[], valid), 1.0);
    }

    #[test]
    fn invalid_json_after_repair_floors_to_zero() {
        // FixTrailingCommas has cost 0.05, so conf = (1.0 - 0.05) * 0.5 = 0.475
        assert!((compute_confidence(&[RepairRule::FixTrailingCommas], "not json") - 0.475).abs() < f32::EPSILON);
    }

    #[test]
    fn truncation_applies_cascade_penalty() {
        let valid = r#"{"key": "value"}"#;
        let rules = vec![RepairRule::FixTruncatedJson, RepairRule::FixSingleQuotes];
        let confidence = compute_confidence(&rules, valid);
        // 0.40 + (0.12 * 1.5) = 0.58 cost → 0.42 base → 0.42 * 0.7 + 0.3 = 0.594
        assert!((confidence - 0.594).abs() < 0.001);
    }

    #[test]
    fn custom_rule_cost_is_respected() {
        let valid = r#"{"key": "value"}"#;
        let rules = vec![RepairRule::Custom {
            name: "fix_bom".to_string(),
            description: "Strips UTF-8 BOM".to_string(),
            cost: 0.03,
        }];
        let confidence = compute_confidence(&rules, valid);
        // 1.0 - 0.03 = 0.97 → 0.97 * 0.7 + 0.3 = 0.979
        assert!((confidence - 0.979).abs() < 0.001);
    }
}
