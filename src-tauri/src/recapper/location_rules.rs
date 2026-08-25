use anyhow::Result;
use std::collections::HashMap;

use crate::pipeline::types::{LocationRule, RuleCondition};

/// A geocoded address with all fields used by the rules engine.
#[derive(Debug, Clone, Default)]
pub struct GeocodedAddress {
    pub city: String,
    pub suburb: String,
    pub state: String,
    pub country: String,
    pub country_code: String,
    pub road: String,
}

impl GeocodedAddress {
    pub fn to_map(&self) -> HashMap<&'static str, &str> {
        let mut m = HashMap::new();
        m.insert("city", self.city.as_str());
        m.insert("suburb", self.suburb.as_str());
        m.insert("state", self.state.as_str());
        m.insert("country", self.country.as_str());
        m.insert("country_code", self.country_code.as_str());
        m.insert("road", self.road.as_str());
        m
    }
}

/// Apply the configured location rules to a geocoded address.
/// Returns the first matching rule's formatted string, or the city as fallback.
/// Ported from BeReal-Recapper.py resolve_location().
pub fn apply_rules(address: &GeocodedAddress, rules: &[LocationRule]) -> String {
    let flat = address.to_map();
    let mut default_result: Option<String> = None;

    for rule in rules {
        let fmt = &rule.format;
        match &rule.condition {
            RuleCondition::Default => {
                // Apply format string substitution and store as fallback
                if let Ok(formatted) = apply_format(fmt, &flat) {
                    let trimmed = formatted.trim().trim_matches(',').trim().to_string();
                    if !trimmed.is_empty() {
                        default_result = Some(trimmed);
                    }
                }
                continue;
            }
            RuleCondition::Match(conditions) => {
                let mut matches = true;
                for (key, expected) in conditions {
                    let actual = flat.get(key.as_str()).copied().unwrap_or("");
                    if actual.to_lowercase() != expected.to_lowercase() {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    if let Ok(formatted) = apply_format(fmt, &flat) {
                        let trimmed = formatted.trim().trim_matches(',').trim().to_string();
                        if !trimmed.is_empty() && trimmed != "," {
                            return trimmed;
                        }
                    }
                }
            }
        }
    }

    // Use default rule result, or city, or empty
    default_result
        .or_else(|| {
            if !address.city.is_empty() {
                Some(address.city.clone())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Apply a Python-style format string like "{city}, {country}" with a flat map.
fn apply_format(template: &str, values: &HashMap<&'static str, &str>) -> Result<String> {
    let mut result = template.to_string();
    for (key, value) in values {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::types::{LocationRule, RuleCondition};
    use std::collections::HashMap;

    fn make_address() -> GeocodedAddress {
        GeocodedAddress {
            city: "London".to_string(),
            suburb: "Brixton".to_string(),
            state: "England".to_string(),
            country: "United Kingdom".to_string(),
            country_code: "gb".to_string(),
            road: "Brixton Road".to_string(),
        }
    }

    #[test]
    fn test_gb_rule_matches() {
        let rules = vec![
            LocationRule {
                comment: None,
                condition: RuleCondition::Match({
                    let mut m = HashMap::new();
                    m.insert("country_code".to_string(), "gb".to_string());
                    m
                }),
                format: "{city}".to_string(),
            },
            LocationRule {
                comment: None,
                condition: RuleCondition::Default,
                format: "{city}, {country}".to_string(),
            },
        ];
        let addr = make_address();
        let result = apply_rules(&addr, &rules);
        assert_eq!(result, "London");
    }

    #[test]
    fn test_default_fallback() {
        let rules = vec![LocationRule {
            comment: None,
            condition: RuleCondition::Default,
            format: "{city}, {country}".to_string(),
        }];
        let addr = make_address();
        let result = apply_rules(&addr, &rules);
        assert_eq!(result, "London, United Kingdom");
    }
}
