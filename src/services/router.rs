use crate::db::rules::RoutingRule;
use crate::models::sentry::SentryWebhookPayload;

/// Match a Sentry payload against routing rules
pub fn match_rule<'a>(payload: &SentryWebhookPayload, rules: &'a [RoutingRule]) -> Option<&'a RoutingRule> {
    let project = payload.get_project();
    let environment = payload.get_environment();
    let level = payload.get_level();

    for rule in rules {
        if matches_rule(rule, &project, &environment, &level) {
            return Some(rule);
        }
    }

    None
}

fn matches_rule(
    rule: &RoutingRule,
    project: &Option<String>,
    environment: &Option<String>,
    level: &Option<String>,
) -> bool {
    // Check project match
    if let Some(pattern) = &rule.project_match {
        if !pattern.is_empty() {
            if let Some(proj) = project {
                if !matches_pattern(pattern, proj) {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    // Check environment match
    if let Some(pattern) = &rule.environment_match {
        if !pattern.is_empty() {
            if let Some(env) = environment {
                if !matches_pattern(pattern, env) {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    // Check level match
    if let Some(pattern) = &rule.level_match {
        if !pattern.is_empty() {
            if let Some(lvl) = level {
                if !matches_pattern(pattern, lvl) {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    true
}

/// Simple wildcard pattern matching
/// Supports:
/// - * matches any sequence of characters
/// - ? matches any single character
/// - exact match
fn matches_pattern(pattern: &str, value: &str) -> bool {
    // Handle comma-separated values (e.g., "production,staging")
    if pattern.contains(',') {
        return pattern.split(',')
            .map(|p| p.trim())
            .any(|p| matches_single_pattern(p, value));
    }

    matches_single_pattern(pattern, value)
}

fn matches_single_pattern(pattern: &str, value: &str) -> bool {
    // Exact match
    if pattern == value {
        return true;
    }

    // Wildcard match
    if pattern == "*" {
        return true;
    }

    // Prefix match (e.g., "my-project-*")
    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return value.starts_with(prefix);
    }

    // Suffix match (e.g., "*-production")
    if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        return value.ends_with(suffix);
    }

    // Contains match (e.g., "*api*")
    if pattern.starts_with('*') && pattern.ends_with('*') && pattern.len() > 2 {
        let middle = &pattern[1..pattern.len() - 1];
        return value.contains(middle);
    }

    // Case-insensitive match
    pattern.to_lowercase() == value.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("*", "anything"));
        assert!(matches_pattern("production", "production"));
        assert!(matches_pattern("my-project-*", "my-project-web"));
        assert!(matches_pattern("*-api", "user-api"));
        assert!(matches_pattern("production,staging", "staging"));
        assert!(!matches_pattern("production", "staging"));
    }
}
