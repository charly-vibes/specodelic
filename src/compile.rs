//! The Compile functor — translate a linted-and-covered spec file's
//! structured layers into machine artifacts.
//!
//! Purpose: implement `specs/compile.md` as one functor with three target
//! categories — Constraints → TOML (`constraint_table_to_toml`), Model →
//! backend-neutral IR (`ModelIR`; TLA+/Alloy emission is deferred to beads
//! `specodelic-mp1` row 6), Properties → proptest! sources
//! (`properties_to_proptest`). Responsibilities: gate on lint cleanliness
//! (`precondition_satisfied`), enforce `compile_is_total` (labeled single
//! failure, never a silent partial), `compile_preserves_ids`, and
//! `no_semantic_drift` (TOML round-trip byte-identical). Rationale: prose
//! is never parsed (`prose_untouched`) and ids are never minted, dropped,
//! or altered — the emitted artifacts are the machine half of the
//! self-hosting contract.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::lint::{self, Report};
use crate::spec::Spec;

// ---------------------------------------------------------------------------
// TOML translation (constraint_table_to_toml)
// ---------------------------------------------------------------------------

/// One Constraint row compiled field-for-field into a TOML entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstraintToml {
    pub id: String,
    pub kind: String,
    pub expr: String,
    pub traces_to: String,
}

/// The TOML document shape: which spec it came from + one entry per row.
/// The fixed field order (serde struct order) is what makes re-emission
/// byte-identical under `no_semantic_drift`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstraintsDoc {
    /// The source file's Intent id — anchors `compile_preserves_ids`.
    pub source: String,
    pub constraints: Vec<ConstraintToml>,
}

/// Compile the Constraints table to a TOML document: one entry per row,
/// `{id, kind, expr, traces_to}`, no lossy transformation.
pub fn constraints_to_toml(spec: &Spec) -> String {
    let doc = constraints_doc(spec);
    toml::to_string(&doc).expect("ConstraintsDoc serializes to TOML")
}

fn constraints_doc(spec: &Spec) -> ConstraintsDoc {
    ConstraintsDoc {
        source: spec.intent.id.clone(),
        constraints: spec
            .constraints
            .iter()
            .map(|r| ConstraintToml {
                id: r.id.clone(),
                kind: r.cells.get("kind").cloned().unwrap_or_default(),
                expr: r.cells.get("expr").cloned().unwrap_or_default(),
                traces_to: r.cells.get("traces_to").cloned().unwrap_or_default(),
            })
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Model extraction (backend-neutral ModelIR — emission deferred to mp1 row 6)
// ---------------------------------------------------------------------------

/// A Model state (value in the future module's state range).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateIr {
    pub id: String,
}

/// A guarded transition — one disjunct of the future `Next` action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitionIr {
    pub id: String,
    pub from: String,
    pub to: String,
    /// `None` when the guard cell is empty (`null` guard).
    pub guard: Option<String>,
}

/// Backend-neutral intermediate representation of the Model section.
/// Both candidate backends (native stateright interpreter, opt-in TLC)
/// consume this same structure; the follow-up change adds the emitter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelIr {
    pub states: Vec<StateIr>,
    pub transitions: Vec<TransitionIr>,
    /// `emits` → effect-Constraint mapping: state id → the referenced
    /// constraint. Exactly one entry per state with an `emits` field —
    /// no entry for the rest, never a default or null
    /// (`output_function_covers_emitting_states_only`).
    pub emits: BTreeMap<String, String>,
}

/// Extract the Model section into the IR without re-parsing prose.
pub fn extract_model_ir(spec: &Spec) -> ModelIr {
    let states = spec
        .states
        .iter()
        .map(|s| StateIr { id: s.id.clone() })
        .collect();
    let transitions = spec
        .transitions
        .iter()
        .map(|t| TransitionIr {
            id: t.id.clone(),
            from: t.from.clone(),
            to: t.to.clone(),
            guard: t.guard.clone(),
        })
        .collect();
    let mut emits = BTreeMap::new();
    for s in &spec.states {
        let Some(raw) = s.cells.get("emits") else {
            continue;
        };
        // Prefer the typed [[link]] target over the raw cell text.
        let target = spec
            .links
            .iter()
            .find(|l| l.source == s.id && l.field == "states" && l.column == "emits")
            .map(|l| l.target.clone())
            .unwrap_or_else(|| raw.clone());
        // `file.row` → row id when the row is a local constraint; keep the
        // raw target when it points elsewhere.
        let row_id = target
            .rsplit_once('.')
            .map(|(_, r)| r.to_string())
            .unwrap_or_else(|| target.clone());
        let resolved = if spec.constraints.iter().any(|c| c.id == row_id) {
            row_id
        } else {
            target
        };
        emits.insert(s.id.clone(), resolved);
    }
    ModelIr {
        states,
        transitions,
        emits,
    }
}

// ---------------------------------------------------------------------------
// proptest! emission (properties_to_proptest)
// ---------------------------------------------------------------------------

/// A generated proptest! block (one per property row, one per required
/// case for law-kind rows).
struct PropBlock {
    /// Rust identifier for the block fn (sanitized row id, + `_<case>` for laws).
    fn_name: String,
    /// The row's verbatim id (kept for `compile_preserves_ids`).
    id: String,
    /// Required case name for law rows (`None` for unit rows).
    case: Option<String>,
    /// Verbatim generator cell text.
    generator: String,
    /// Verbatim predicate cell text.
    predicate: String,
    /// Generator function names referenced by the row, in order.
    gens: Vec<String>,
}

/// Compile the Properties table to a proptest! source file: one block per
/// row (`proptest_block_per_property`), one block per required case for
/// law-kind rows (`law_property_compiles_required_cases`).
///
/// Emission is compilable scaffolding, not a pseudo-code translation: the
/// strategy is a call into the generated `spec_gen` helpers and the
/// predicate is carried verbatim through `todo_predicate!`, which compiles
/// here and only fails when executed — execution is `verify`'s (1pv) job.
pub fn properties_to_proptest(spec: &Spec) -> String {
    let file_id = &spec.intent.id;
    let blocks: Vec<PropBlock> = spec
        .properties
        .iter()
        .flat_map(|p| blocks_for_row(p.id.clone(), p.kind.as_deref(), p))
        .collect();

    let mut out = String::new();
    out.push_str(&format!(
        "//! proptest scaffolding for `{file_id}` — generated by `spk compile`.\n\
         //! Source of truth: the spec file itself — edit there, not here.\n\
         //! Each block quotes its row's generator and predicate verbatim;\n\
         //! predicates not yet translated to Rust fail in `verify` via\n\
         //! `todo_predicate!`, never at artifact emission.\n\n"
    ));
    if blocks.is_empty() {
        // No property rows: an empty-but-present artifact beats an absent one.
        out.push_str("//! (no Property rows in the source spec)\n");
        return out;
    }
    out.push_str("use proptest::prelude::*;\n\n");
    out.push_str(
        "/// Placeholder element type for generated values; `verify` replaces\n\
         /// this scaffolding with real strategies and assertion bodies.\n\
         #[derive(Debug, Clone)]\n\
         pub struct GenVal(pub String);\n\n\
         /// Marker for predicates not yet translated to Rust: compiles here,\n\
         /// panics when executed — execution is `verify`'s job.\n\
         macro_rules! todo_predicate {\n    ($reason:expr) => { todo!(\"{}\", $reason) };\n}\n\n",
    );

    // spec_gen helpers: one per distinct generator referenced anywhere.
    let mut gens: Vec<&str> = vec![];
    for b in &blocks {
        for g in &b.gens {
            if !gens.contains(&g.as_str()) {
                gens.push(g);
            }
        }
    }
    if !gens.is_empty() {
        out.push_str(
            "/// Generated strategies — one per generator named in the spec rows.\n\
             /// The real element types and strategies are `verify`'s domain.\n\
             pub mod spec_gen {\n    use super::*;\n\n",
        );
        for g in &gens {
            out.push_str(&format!(
                "    pub fn {g}() -> impl Strategy<Value = GenVal> {{\n        Just(GenVal({g:?}.into()))\n    }}\n\n"
            ));
        }
        out.push_str("}\n\n");
    }

    for b in &blocks {
        out.push_str("proptest! {\n");
        out.push_str(&format!("    // id: {}\n", b.id));
        if let Some(case) = &b.case {
            out.push_str(&format!("    // case: {case}\n"));
        }
        out.push_str(&format!("    // generator: {}\n", b.generator));
        out.push_str(&format!("    // predicate: {}\n", b.predicate));
        out.push_str("    #[test]\n");
        let args = b
            .gens
            .iter()
            .enumerate()
            .map(|(i, g)| format!("v{i} in spec_gen::{g}()"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("    fn {}({}) {{\n", b.fn_name, args));
        out.push_str(&format!("        todo_predicate!({:?});\n", b.predicate));
        out.push_str("    }\n}\n\n");
    }
    out
}

/// One or more blocks for a single Property row: unit rows yield exactly
/// one; law rows yield one per required case, read from the row's own
/// predicate per `law_requires_cases` (floor: identity, associativity).
fn blocks_for_row(id: String, kind: Option<&str>, row: &crate::spec::Row) -> Vec<PropBlock> {
    let generator = row.cells.get("generator").cloned().unwrap_or_default();
    let predicate = row.cells.get("predicate").cloned().unwrap_or_default();
    let gens = generator_names(&generator);
    let base = sanitize_ident(&id);
    match kind {
        Some("law") => required_law_cases(&predicate)
            .into_iter()
            .map(|case| PropBlock {
                fn_name: format!("{base}_{}", sanitize_ident(&case)),
                id: id.clone(),
                case: Some(case),
                generator: generator.clone(),
                predicate: predicate.clone(),
                gens: gens.clone(),
            })
            .collect(),
        _ => vec![PropBlock {
            fn_name: base,
            id,
            case: None,
            generator,
            predicate,
            gens,
        }],
    }
}

/// Generator identifiers in a generator cell: every `name(` occurrence.
fn generator_names(cell: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").expect("static regex");
    let mut names: Vec<String> = vec![];
    for caps in re.captures_iter(cell) {
        let name = caps[1].to_string();
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// Required cases for a law row: `**case:**` labels in the predicate;
/// falls back to the `law_requires_cases` floor (identity, associativity).
fn required_law_cases(predicate: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\*\*([a-zA-Z][a-zA-Z _-]*?):\*\*").expect("static regex");
    let cases: Vec<String> = re
        .captures_iter(predicate)
        .map(|c| c[1].trim().to_string())
        .filter(|c| !c.is_empty())
        .collect();
    if cases.is_empty() {
        vec!["identity".into(), "associativity".into()]
    } else {
        cases
    }
}

/// Sanitize a spec id into a valid Rust identifier (ids are never altered —
/// the verbatim id travels in the block's comment for id preservation).
fn sanitize_ident(id: &str) -> String {
    let s: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{s}")
    } else {
        s
    }
}

// ---------------------------------------------------------------------------
// Pipeline contract
// ---------------------------------------------------------------------------

/// One complete compilation — every artifact due under the current backend
/// decision (TOML + proptest source + ModelIR; the model module lands after
/// `mp1` row 6). `compile_is_total`: either this, or a labeled failure.
#[derive(Debug, Clone, Serialize)]
pub struct Compiled {
    /// The `constraint_table_to_toml` artifact.
    pub toml: String,
    /// The `properties_to_proptest` artifact.
    pub props: String,
    /// The extracted Model IR (stands in for the model module until the
    /// backend decision; preserves state/transition ids).
    pub model_ir: ModelIr,
}

/// A labeled, single failure — never a silent partial result.
#[derive(Debug, Clone, Serialize)]
pub struct CompileError {
    /// Which stage failed: `precondition_satisfied`, `no_semantic_drift`,
    /// `compile_preserves_ids`.
    pub stage: String,
    pub message: String,
}

/// `precondition_satisfied` — the file must pass lint with zero issues
/// (linted and covered). Implemented by reusing the existing lint pass,
/// not a second implementation.
pub fn precondition_satisfied(spec: &Spec, report: &Report) -> Result<(), CompileError> {
    let path = spec
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let file_id = &spec.intent.id;
    let issues: Vec<&lint::Issue> = report
        .issues
        .iter()
        .filter(|i| i.file == *file_id || (!path.is_empty() && i.file == path))
        .collect();
    if issues.is_empty() {
        return Ok(());
    }
    let names: Vec<&str> = issues.iter().map(|i| i.rule.as_str()).collect();
    Err(CompileError {
        stage: "precondition_satisfied".into(),
        message: format!(
            "file has {} lint finding(s) — compile requires a linted-and-covered file (fired: {})",
            issues.len(),
            names.join(", ")
        ),
    })
}

/// Compile one spec file through the full contract. Idempotent and
/// byte-stable: the same input yields byte-identical artifacts.
pub fn compile_spec(spec: &Spec) -> Result<Compiled, CompileError> {
    let toml = constraints_to_toml(spec);
    let model_ir = extract_model_ir(spec);
    let props = properties_to_proptest(spec);

    // no_semantic_drift — re-parsing the TOML and re-emitting it yields
    // output identical to the original compile.
    let doc: ConstraintsDoc = toml::from_str(&toml).map_err(|e| CompileError {
        stage: "no_semantic_drift".into(),
        message: format!("emitted TOML failed to re-parse: {e}"),
    })?;
    let re_emitted = toml::to_string(&doc).map_err(|e| CompileError {
        stage: "no_semantic_drift".into(),
        message: format!("re-emission failed: {e}"),
    })?;
    if re_emitted != toml {
        return Err(CompileError {
            stage: "no_semantic_drift".into(),
            message: "re-parsed TOML re-emits to different bytes".into(),
        });
    }

    // compile_preserves_ids — every source id appears, unchanged, in at
    // least one artifact (states/transitions ride in the ModelIR).
    let artifact_text = format!("{toml}{props}");
    let missing: Vec<String> = spec
        .defined_ids()
        .into_iter()
        .filter(|id| !artifact_text.contains(id.as_str()) && !ir_contains_id(&model_ir, id))
        .collect();
    if !missing.is_empty() {
        return Err(CompileError {
            stage: "compile_preserves_ids".into(),
            message: format!("ids dropped in translation: {}", missing.join(", ")),
        });
    }

    Ok(Compiled {
        toml,
        props,
        model_ir,
    })
}

fn ir_contains_id(ir: &ModelIr, id: &str) -> bool {
    ir.states.iter().any(|s| s.id == id)
        || ir.transitions.iter().any(|t| t.id == id)
        || ir.emits.keys().any(|k| k == id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::parse_str;

    const SAMPLE: &str = "---\nid: demo.thing\nkind: intent\nstatement: \"THE system SHALL work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x holds` | [[demo.thing]] |\n| b | effect | `y fires` | [[demo.thing]] |\n\n## Model\n\n### States\n\n- s1\n- s2 (emits: `[[demo.thing.b]]`)\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[demo.thing.a]] |\n| t2 | s2 | s1 | `done` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.thing.a]] | `arbitrary_row()` | `check(x) == ok` |\n| p2 | law | [[demo.thing.a]] | `arbitrary_row(), other()` | **identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)` |\n";

    fn sample() -> Spec {
        parse_str(SAMPLE).expect("sample parses")
    }

    // --- 1.2 / 1.3 Constraints → TOML ---

    #[test]
    fn toml_roundtrips_field_for_field() {
        let spec = sample();
        let toml = constraints_to_toml(&spec);
        let doc: ConstraintsDoc = toml::from_str(&toml).expect("re-parses");
        assert_eq!(doc.source, "demo.thing");
        assert_eq!(doc.constraints.len(), 2);
        let a = &doc.constraints[0];
        assert_eq!(a.id, "a");
        assert_eq!(a.kind, "invariant");
        assert_eq!(a.expr, "`x holds`");
        assert_eq!(a.traces_to, "[[demo.thing]]");
    }

    #[test]
    fn toml_ids_preserved_verbatim() {
        let spec = sample();
        let toml = constraints_to_toml(&spec);
        assert!(toml.contains("id = \"a\""));
        assert!(toml.contains("id = \"b\""));
    }

    // --- 2.1 / 2.2 Model → ModelIR ---

    #[test]
    fn ir_transition_count_matches() {
        let spec = sample();
        let ir = extract_model_ir(&spec);
        assert_eq!(ir.transitions.len(), 2);
        let t1 = &ir.transitions[0];
        assert_eq!(t1.from, "s1");
        assert_eq!(t1.to, "s2");
        assert_eq!(t1.guard.as_deref(), Some("[[demo.thing.a]]"));
    }

    #[test]
    fn ir_emits_covers_exactly_emitting_states() {
        let spec = sample();
        let ir = extract_model_ir(&spec);
        // exactly one entry — no entry for s1, never a default or null
        assert_eq!(ir.emits.len(), 1);
        assert_eq!(ir.emits.get("s2").map(String::as_str), Some("b"));
        assert!(!ir.emits.contains_key("s1"));
    }

    // --- 3.2 / 3.3 Properties → proptest! ---

    #[test]
    fn unit_property_yields_at_least_one_block() {
        let spec = sample();
        let src = properties_to_proptest(&spec);
        assert!(src.contains("proptest!"));
        assert!(src.contains("fn p1("));
        assert!(src.contains("todo_predicate!"));
    }

    #[test]
    fn law_expands_to_required_cases() {
        let spec = sample();
        let src = properties_to_proptest(&spec);
        // p1 (unit) → 1 block; p2 (law, identity+associativity) → 2 blocks
        assert_eq!(src.matches("#[test]").count(), 3);
        assert!(src.contains("fn p2_identity("));
        assert!(src.contains("fn p2_associativity("));
    }

    #[test]
    fn law_without_labels_falls_back_to_floor() {
        let spec = parse_str(
            "---\nid: demo.law\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[demo.law]] |\n\n## Model\n\n### States\n\n- s\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s | s | [[demo.law.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p | law | [[demo.law.a]] | `g()` | `f holds` |\n",
        )
        .unwrap();
        let src = properties_to_proptest(&spec);
        // floor: identity + associativity
        assert_eq!(src.matches("#[test]").count(), 2);
        assert!(src.contains("fn p_identity("));
        assert!(src.contains("fn p_associativity("));
    }

    #[test]
    fn generators_get_spec_gen_helpers() {
        let spec = sample();
        let src = properties_to_proptest(&spec);
        assert!(src.contains("pub fn arbitrary_row()"));
        assert!(src.contains("pub fn other()"));
        assert!(src.contains("v0 in spec_gen::arbitrary_row()"));
        assert!(src.contains("v1 in spec_gen::other()"));
    }

    // --- 4.2 contract invariants ---

    #[test]
    fn compile_preserves_every_source_id() {
        let spec = sample();
        let compiled = compile_spec(&spec).expect("compiles");
        let text = format!("{}{}{:?}", compiled.toml, compiled.props, compiled.model_ir);
        for id in spec.defined_ids() {
            assert!(
                text.contains(&id) || ir_contains_id(&compiled.model_ir, &id),
                "id `{id}` dropped"
            );
        }
    }

    #[test]
    fn no_semantic_drift_roundtrip_is_byte_identical() {
        let spec = sample();
        let first = compile_spec(&spec).expect("compiles");
        let second = compile_spec(&spec).expect("compiles");
        assert_eq!(first.toml, second.toml);
        assert_eq!(first.props, second.props);
        // re-parse + re-emit == original
        let doc: ConstraintsDoc = toml::from_str(&first.toml).unwrap();
        assert_eq!(toml::to_string(&doc).unwrap(), first.toml);
    }
}
