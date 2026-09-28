//! The EARS statement grammar — pattern conformance for Intent statements.
//!
//! Purpose: implement `specs/linter-ears_syntax.md`'s `ears_pattern_match`
//! and `has_shall`. Responsibilities: classify a statement into one of the
//! five EARS patterns (Ubiquitous / Event-Driven / State-Driven /
//! Unwanted-Behavior / Optional-Feature) and require an imperative
//! `SHALL`. Rationale: EARS conformance is a grammar fact — shape only,
//! never word-choice judgment (prose stays prose).

/// Which EARS pattern a statement matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum EarsPattern {
    /// `THE SYSTEM SHALL …`
    Ubiquitous,
    /// `WHEN <trigger> THE SYSTEM SHALL …`
    EventDriven,
    /// `WHILE <state> THE SYSTEM SHALL …`
    StateDriven,
    /// `IF <condition> THEN THE SYSTEM SHALL …` (also `WHEN … THEN … SHALL`)
    UnwantedBehavior,
    /// `WHERE <feature> IS INCLUDED THE SYSTEM SHALL …`
    OptionalFeature,
}

/// Check a statement against the EARS grammar.
///
/// Returns `None` when the statement matches no pattern.
pub fn classify(statement: &str) -> Option<EarsPattern> {
    let s = collapse(statement);
    // Imperative mood is required in every pattern.
    if !s.contains("SHALL") {
        return None;
    }
    let must_have_subject_shall = |rest: &str| -> bool {
        // After the trigger, some `THE <subject> SHALL` must follow.
        rest.contains("SHALL") && rest.to_uppercase().contains("THE ")
    };

    let upper = s.to_uppercase();
    if let Some(rest) = upper.strip_prefix("WHILE ") {
        if must_have_subject_shall(rest) {
            return Some(EarsPattern::StateDriven);
        }
        return None;
    }
    if let Some(rest) = upper.strip_prefix("WHERE ") {
        if must_have_subject_shall(rest) {
            return Some(EarsPattern::OptionalFeature);
        }
        return None;
    }
    if let Some(rest) = upper.strip_prefix("IF ") {
        if rest.contains("THEN") && must_have_subject_shall(rest) {
            return Some(EarsPattern::UnwantedBehavior);
        }
        return None;
    }
    if let Some(rest) = upper.strip_prefix("WHEN ") {
        if rest.contains(" THEN")
            && rest
                .split(" THEN")
                .nth(1)
                .is_some_and(|a| a.contains("SHALL"))
        {
            return Some(EarsPattern::UnwantedBehavior);
        }
        if must_have_subject_shall(rest) {
            return Some(EarsPattern::EventDriven);
        }
        return None;
    }
    if upper.starts_with("THE ") && upper.contains(" SHALL") {
        return Some(EarsPattern::Ubiquitous);
    }
    None
}

/// `has_shall` — the statement carries an imperative SHALL.
pub fn has_shall(statement: &str) -> bool {
    statement.to_uppercase().contains("SHALL")
}

/// Collapse frontmatter folded-line continuations into single spaces.
fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubiquitous() {
        assert_eq!(
            classify("THE specodelic SHALL represent a feature's intent as one parseable file."),
            Some(EarsPattern::Ubiquitous)
        );
    }

    #[test]
    fn event_driven() {
        assert_eq!(
            classify(
                "WHEN a customer cancels a paid order, THE system SHALL refund the payment in full."
            ),
            Some(EarsPattern::EventDriven)
        );
    }

    #[test]
    fn unwanted_behavior() {
        assert_eq!(
            classify("IF the guard is empty THEN THE system SHALL treat the guard as null."),
            Some(EarsPattern::UnwantedBehavior)
        );
    }

    #[test]
    fn state_driven() {
        assert_eq!(
            classify(
                "WHILE the orchestrator is in lint_stage, THE system SHALL only run checkers."
            ),
            Some(EarsPattern::StateDriven)
        );
    }

    #[test]
    fn optional_feature() {
        assert_eq!(
            classify(
                "WHERE an external checklist IS INCLUDED, THE system SHALL require every item to carry a claim."
            ),
            Some(EarsPattern::OptionalFeature)
        );
    }

    #[test]
    fn no_shall_fails() {
        assert_eq!(classify("THE system SHOULD maybe work."), None);
    }

    #[test]
    fn folded_frontmatter_lines_match() {
        // statements in frontmatter wrap across lines; whitespace collapse
        // must not break the match
        let folded = "WHEN two branches derived from a common ancestor spec repo are\n  joined, THE merge tool SHALL detect id collisions and reference breaks.";
        assert_eq!(classify(folded), Some(EarsPattern::EventDriven));
    }

    #[test]
    fn has_shall_works() {
        assert!(has_shall("THE system SHALL work"));
        assert!(has_shall("the system shall not work"));
        assert!(!has_shall("the system works"));
    }
}
