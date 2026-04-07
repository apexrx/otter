use crate::repair_engine::RepairRule;

pub fn compute_confidence(rules: &[RepairRule], repaired: &str) -> f32 {
    let is_valid_json = serde_json::from_str::<serde_json::Value>(repaired).is_ok();

    if rules.is_empty() {
        return if is_valid_json { 1.0 } else { 0.0 };
    }

    let truncation_fired = rules.contains(&RepairRule::FixTruncatedJson);

    let raw_cost: f32 = rules.iter().map(|rule| {
        let cost = rule.cost();

        if truncation_fired && *rule != RepairRule::FixTruncatedJson {
            cost * 1.5
        } else {
            cost
        }
    })
    .sum();

    let mut conf = (1.0 - raw_cost).clamp(0.0, 1.0);

    if is_valid_json {
        conf = conf * 0.7 + 0.3;
    } else {
        conf *= 0.5;
    }

    conf
}
