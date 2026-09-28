//! Unit tests for the spec parser — synthetic four-layer files.

use specodelic::spec::parse_str;

const SIMPLE: &str = "---\nid: order.cancel\nkind: intent\nstatement: \"WHEN a customer cancels a paid order, THE system SHALL refund in full.\"\n---\n\n# Order Cancellation\n\nProse.\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| refund_bounded | invariant | `refund_amount == paid_amount` | [[order.cancel]] |\n\n## Model\n\n### States\n\n- `active`\n- `refunded`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| refund | active | refunded | [[order.cancel.refund_bounded]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| full_refund_only | unit | [[order.cancel.refund_bounded]] | `arbitrary_paid_order()` | `refund(cancel(o)) == o.paid` |\n";

#[test]
fn parses_all_four_layers() {
    let spec = parse_str(SIMPLE).unwrap();
    assert_eq!(spec.intent.id, "order.cancel");
    assert_eq!(spec.constraints.len(), 1);
    assert_eq!(spec.states.len(), 2);
    assert_eq!(spec.transitions.len(), 1);
    assert_eq!(spec.properties.len(), 1);
    let t = &spec.transitions[0];
    assert_eq!(t.from, "active");
    assert_eq!(t.to, "refunded");
    assert_eq!(t.guard.as_deref(), Some("[[order.cancel.refund_bounded]]"));
}

#[test]
fn missing_frontmatter_is_parse_error() {
    let err = parse_str("# no frontmatter\n").unwrap_err();
    assert!(err.to_string().contains("no YAML frontmatter"));
}

#[test]
fn missing_statement_field_is_parse_error() {
    let text = "---\nid: x\nkind: intent\n---\n";
    assert!(parse_str(text).is_err());
}

#[test]
fn collects_links_from_structured_fields_only() {
    let spec = parse_str(SIMPLE).unwrap();
    let targets: Vec<_> = spec.links.iter().map(|l| l.target.as_str()).collect();
    assert!(targets.contains(&"order.cancel")); // frontmatter expr? no — constraints.traces_to
    assert!(targets.contains(&"order.cancel.refund_bounded"));
}

#[test]
fn guards_can_be_null() {
    let text = SIMPLE.replace("[[order.cancel.refund_bounded]] |", "|");
    let spec = parse_str(&text).unwrap();
    assert_eq!(spec.transitions[0].guard, None);
}

#[test]
fn states_with_emits_yield_typed_links() {
    let text = "---\nid: m\nkind: intent\nstatement: \"THE system SHALL work.\"\n---\n\n## Model\n\n### States\n\n- `published` (emits: `[[m.advisory_finding_emitted]]`)\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| go | a | published | [[m.ok]] |\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| ok | advisory | `always` | [[m]] |\n| advisory_finding_emitted | effect | `report` | [[m]] |\n";
    let spec = parse_str(text).unwrap();
    let emits: Vec<_> = spec.links.iter().filter(|l| l.field == "states").collect();
    assert_eq!(emits.len(), 1);
    assert_eq!(emits[0].target, "m.advisory_finding_emitted");
    assert_eq!(emits[0].column, "emits");
}

#[test]
fn backticked_state_bullets_do_not_leak_backticks_into_ids() {
    // beads specodelic-cl3: a lone backticked bullet `- `active`` used to
    // parse the state id as "active`" (trailing backtick kept), which made
    // state ids incoherent with the bare from/to cells of transitions.
    let spec = parse_str(SIMPLE).unwrap();
    let ids: Vec<_> = spec.states.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["active", "refunded"]);
}

#[test]
fn subsection_under_constraints_does_not_swallow_its_table() {
    // mirrors specodelic.md's `### Reference Typing` under `## Constraints`
    let text = "---\nid: s\nkind: intent\nstatement: \"THE system SHALL work.\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[s]] |\n\n### Reference Typing\n\n| Field | Appears on | Must resolve to |\n|-------|-----------|-----------------|\n| `traces_to` | Constraint | Intent |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[s.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[s.a]] | `g()` | `x` |\n";
    let spec = parse_str(text).unwrap();
    // only the real constraint row is captured, not the Reference Typing rows
    assert_eq!(spec.constraints.len(), 1);
    assert_eq!(spec.constraints[0].id, "a");
}

#[test]
fn backtick_commas_and_pipes_in_expr_cells_survive() {
    let text = "---\nid: s\nkind: intent\nstatement: \"THE system SHALL work.\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `∀ x: f(x) ∧ g(y)` — nested `kind` mentions | [[s]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[s.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[s.a]] | `g()` | `x` |\n";
    let spec = parse_str(text).unwrap();
    assert!(
        spec.constraints[0]
            .cells
            .get("expr")
            .unwrap()
            .contains("f(x) ∧ g(y)")
    );
}
