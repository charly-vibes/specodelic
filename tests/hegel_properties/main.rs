//! Hegel property suite (specodelic-jnnf) — the bb98 scratchbook's
//! approved laws over generated corpora: rename algebra, acset writer
//! apply/spans, merge verdicts, compile determinism.
//!
//! Scope guard rails (ticket + recommendation): tests/ only; no touch of
//! `src/compile.rs`, the generated `specodelic/specodelic/*_props.rs`
//! artifacts, the verify scratch crate, or the `proptest` dependency; no
//! Hegel in shipped code paths. Every generated corpus is driven through
//! `corpus::check_corpus` (parse + model equality + lint + dangling)
//! before any property consumes it, so a red here is evidence about the
//! crate, not about the generator.
//!
//! Oracle validity (bb98): the red-triaged law
//! (`merge_verdict_symmetry_add_add`, enabled after the specodelic-1izc
//! fix) is the seeded-defect exhibit — it fired on HEAD's add/add
//! asymmetry and stays green under
//! a no-op product mutation. The remaining properties were predicted
//! green from source and are executed here; the two guards
//! (`*_reachable`) fail the suite if the generator never reaches the
//! success/middle states it claims to cover.

mod corpus;
mod trees;

use std::collections::BTreeMap;
use std::path::PathBuf;

use hegel::TestCase;
use hegel::generators as gs;
use specodelic::acset::writer::{self, Edit, WriteSet};
use specodelic::compile;
use specodelic::merge;
use specodelic::rename::{self, RenameError, RenameOutcome};
use specodelic::spec::{Span, Spec, parse_str};

use crate::corpus::{Corpus, ModelFile, build_corpus, build_file, check_corpus, render};
use crate::trees::{Tree, add_self_file, delete_file, edit_expr, inject_dangling_traces, placed};

/// Draw a bounded generator seed.
fn seed_of(tc: &TestCase, salt: u64) -> u64 {
    tc.draw(
        gs::integers::<u64>()
            .min_value(salt + 1)
            .max_value(salt + 4096),
    )
}

fn pb(name: &str) -> PathBuf {
    PathBuf::from(name)
}

fn parse_ok(path: &str, text: &str) -> Spec {
    parse_str(text).unwrap_or_else(|e| panic!("{path} does not parse: {e}"))
}

fn spec_with_path(path: &str, text: &str) -> Spec {
    let mut s = parse_ok(path, text);
    s.path = Some(pb(path));
    s
}

fn files_as_pathbuf(files: &[(String, String)]) -> Vec<(PathBuf, String)> {
    files.iter().map(|(p, t)| (pb(p), t.clone())).collect()
}

/// Apply a rename outcome to a path→text map (writes then the removal).
fn materialize(
    files: &BTreeMap<PathBuf, String>,
    out: &RenameOutcome,
) -> BTreeMap<PathBuf, String> {
    let mut m = files.clone();
    for (p, t) in &out.writes {
        m.insert(p.clone(), t.clone());
    }
    if let Some(r) = &out.remove {
        m.remove(r);
    }
    m
}

fn as_files(m: &BTreeMap<PathBuf, String>) -> Vec<(PathBuf, String)> {
    m.iter().map(|(p, t)| (p.clone(), t.clone())).collect()
}

/// The writer's renamable-column contract (rename.md any-cell rule over
/// the deny-set fixed by specodelic-77mq). Deliberately re-stated here:
/// if the product's set drifts, `writer_untouched_bytes_preserved`
/// disagrees with this oracle and the suite goes red.
fn renamable(col: &str) -> bool {
    !matches!(col, "kind" | "expr" | "generator" | "predicate")
}

/// The replacement set a rename edit MUST produce, derived from the
/// parser's own recorded spans (the oracle for untouched bytes).
fn expected_repls(
    spec: &Spec,
    old_q: &str,
    new_q: &str,
    local: Option<(&str, &str)>,
) -> Vec<(Span, String)> {
    let mut repls: Vec<(Span, String)> = Vec::new();
    let (lo, ln) = local.unwrap_or(("", ""));
    // Frontmatter id (intent renames only).
    if spec.intent.id == old_q
        && let Some(sp) = spec.intent.id_span
    {
        repls.push((sp, new_q.to_string()));
    }
    push_row_repls(
        spec.constraints
            .iter()
            .chain(&spec.properties)
            .chain(&spec.states),
        lo,
        ln,
        &mut repls,
    );
    push_transition_repls(&spec.transitions, lo, ln, &mut repls);
    push_link_repls(&spec.links, old_q, new_q, &mut repls);
    repls.sort_by_key(|(sp, _)| sp.start);
    repls.dedup_by_key(|(sp, _)| sp.start);
    repls
}

/// Definition-table and state-bullet replacements: the row's own id
/// cell plus every renamable cell whose value equals the local id.
fn push_row_repls<'a>(
    rows: impl Iterator<Item = &'a specodelic::spec::Row>,
    lo: &str,
    ln: &str,
    repls: &mut Vec<(Span, String)>,
) {
    if lo.is_empty() {
        return;
    }
    for row in rows {
        if row.id == lo
            && let Some(sp) = row.id_span
        {
            repls.push((sp, ln.to_string()));
        }
        for (col, val) in &row.cells {
            // The `id` column rides id_span above; matching it here would
            // duplicate the replacement.
            if col != "id"
                && val == lo
                && renamable(col)
                && let Some(sp) = row.cell_spans.get(col)
            {
                repls.push((*sp, ln.to_string()));
            }
        }
    }
}

/// Transition replacements: id/from/to ride their dedicated spans;
/// guard cells hold links or prose — never a bare row local — and
/// Transition records no other per-column spans.
fn push_transition_repls(
    transitions: &[specodelic::spec::Transition],
    lo: &str,
    ln: &str,
    repls: &mut Vec<(Span, String)>,
) {
    if lo.is_empty() {
        return;
    }
    for t in transitions {
        if t.id == lo
            && let Some(sp) = t.id_span
        {
            repls.push((sp, ln.to_string()));
        }
        if t.from == lo
            && let Some(sp) = t.from_span
        {
            repls.push((sp, ln.to_string()));
        }
        if t.to == lo
            && let Some(sp) = t.to_span
        {
            repls.push((sp, ln.to_string()));
        }
    }
}

/// Wiki-link replacements: the qualified old id and its `.<child>`
/// suffixes, with the emitted link rebuilt from the trimmed target.
fn push_link_repls(
    links: &[specodelic::spec::Link],
    old_q: &str,
    new_q: &str,
    repls: &mut Vec<(Span, String)>,
) {
    for link in links {
        let renamed = if link.target == old_q {
            Some(new_q.to_string())
        } else {
            link.target
                .strip_prefix(&format!("{old_q}."))
                .map(|child| format!("{new_q}.{child}"))
        };
        if let (Some(nt), Some(sp)) = (renamed, link.span) {
            repls.push((sp, format!("[[{nt}]]")));
        }
    }
}

/// Splice replacements into source bytes — every byte outside a replaced
/// span passes through untouched.
fn splice(source: &str, repls: &[(Span, String)]) -> String {
    let mut out = String::new();
    let mut cursor = 0usize;
    for (sp, new) in repls {
        assert!(
            sp.start >= cursor,
            "oracle bug: overlapping replacements at {}",
            sp.start
        );
        out.push_str(&source[cursor..sp.start]);
        out.push_str(new);
        cursor = sp.end;
    }
    out.push_str(&source[cursor..]);
    out
}

/// Bytes outside the replaced spans are identical between source and
/// emission (the untouched-bytes claim, stated directly). The emitted
/// cursor advances by the replacement length — replacements may change
/// total length, so fixed offsets would misalign.
fn assert_untouched_outside(source: &str, emitted: &str, repls: &[(Span, String)]) {
    let mut kept_src = String::new();
    let mut kept_out = String::new();
    let mut sc = 0usize; // cursor in source
    let mut ec = 0usize; // cursor in emitted
    for (sp, new) in repls {
        kept_src.push_str(&source[sc..sp.start]);
        let kept_len = sp.start - sc;
        kept_out.push_str(&emitted[ec..ec + kept_len]);
        sc = sp.end;
        ec += kept_len + new.len();
    }
    kept_src.push_str(&source[sc..]);
    kept_out.push_str(&emitted[ec..]);
    assert_eq!(
        kept_src, kept_out,
        "bytes outside the edit's spans were touched"
    );
}

fn assert_case_fresh(corpus: &Corpus, case: &str) {
    check_corpus(corpus).unwrap_or_else(|e| panic!("generator bug ({case}): {e}"));
}

/// The rename/writer corpus: 2 files, the second tracing the first's c0.
fn two_file_corpus(seed: u64) -> Corpus {
    build_corpus(seed | 1)
}

// ---------------------------------------------------------------------------
// Merge verdict laws (Tier 1)
// ---------------------------------------------------------------------------

/// merge-collision-correctness: the collision presence/absence partition
/// over generated triples (file-text granularity, per Tip.ids).
#[hegel::test(test_cases = 16, derandomize = true)]
fn merge_collision_partition(tc: TestCase) {
    let seed = seed_of(&tc, 100);
    let base_corpus = two_file_corpus(seed);
    assert_case_fresh(&base_corpus, "collision base");
    let base: Tree = base_corpus.files.clone();

    // Leg 1 — newly minted on both branches at the same path: collision.
    let mut a = base.clone();
    add_self_file(&mut a, "demo-delta.md", "demo.delta", seed);
    let mut b = base.clone();
    add_self_file(&mut b, "demo-delta.md", "demo.delta", seed + 7);
    let rep = merge::run(&base, &a, &b);
    assert_eq!(rep.verdict, "failed", "add/add of the same id must fail");
    assert!(
        rep.findings.iter().any(|f| f.kind == "id_collision"),
        "newly minted id on both branches must collide: {:?}",
        rep.findings
    );

    // Leg 2 — both branches edit an inherited id's file, differently:
    // collision for the file's ids plus a textual conflict.
    let alpha_expr = base_corpus.models[0].constraints[0].expr.clone();
    let mut a = base.clone();
    edit_expr(
        &mut a,
        "demo-alpha.md",
        &alpha_expr,
        "rewritten on branch a",
    );
    let mut b = base.clone();
    edit_expr(
        &mut b,
        "demo-alpha.md",
        &alpha_expr,
        "rewritten on branch b",
    );
    let rep = merge::run(&base, &a, &b);
    assert_eq!(rep.verdict, "failed");
    assert!(rep.findings.iter().any(|f| f.kind == "id_collision"));
    assert!(rep.findings.iter().any(|f| f.kind == "textual_conflict"));

    // Leg 3 — inherited id edited on one branch only: no collision.
    let mut a = base.clone();
    edit_expr(&mut a, "demo-alpha.md", &alpha_expr, "edited on a only");
    let rep = merge::run(&base, &a, &base);
    assert!(
        !rep.findings.iter().any(|f| f.kind == "id_collision"),
        "one-sided edit of an inherited id is not a collision: {:?}",
        rep.findings
    );

    // Leg 4 — identical edits on both branches: collision (both differ
    // from base) but NO textual conflict.
    let mut a = base.clone();
    edit_expr(&mut a, "demo-alpha.md", &alpha_expr, "same edit both sides");
    let b = a.clone();
    let rep = merge::run(&base, &a, &b);
    assert!(rep.findings.iter().any(|f| f.kind == "id_collision"));
    assert!(
        !rep.findings.iter().any(|f| f.kind == "textual_conflict"),
        "identical edits are not a textual conflict: {:?}",
        rep.findings
    );
}

/// merge-blast-radius-intersection: textually disjoint edits whose
/// closures intersect through a cross-file reference land in
/// `needs_review`, naming the intersecting id.
#[hegel::test(test_cases = 16, derandomize = true)]
fn merge_blast_radius_intersection_law(tc: TestCase) {
    let seed = seed_of(&tc, 200);
    let base_corpus = two_file_corpus(seed);
    assert_case_fresh(&base_corpus, "blast base");
    let base = base_corpus.files.clone();
    let alpha_expr = base_corpus.models[0].constraints[0].expr.clone();

    // A edits demo-alpha.md in place (ids unchanged).
    let mut a = base.clone();
    edit_expr(&mut a, "demo-alpha.md", &alpha_expr, "touched on branch a");
    // B adds a new file whose constraint traces [[demo.alpha]] — an
    // Intent-kind target (ref_kind_compatible), defined on both tips.
    let target = "demo.alpha";
    let gamma = build_file("demo.gamma", seed + 31, Some(target.to_string()));
    let mut b = base.clone();
    b.push(placed("demo-gamma.md", &gamma));

    let rep = merge::run(&base, &a, &b);
    assert_eq!(
        rep.verdict, "needs_review",
        "intersecting blast radii must not report clean: {:?}",
        rep.findings
    );
    let blast = rep
        .findings
        .iter()
        .filter(|f| f.kind == "blast_radius_intersection")
        .count();
    assert_eq!(
        blast, 1,
        "exactly one intersection finding: {:?}",
        rep.findings
    );
    assert!(
        rep.findings[0].message.contains("demo.alpha"),
        "the finding must name the intersecting id: {:?}",
        rep.findings
    );
    // Swapped argument order reports the same verdict and kind multiset.
    let swapped = merge::run(&base, &b, &a);
    assert_eq!(swapped.verdict, rep.verdict);
    let mut kinds_a: Vec<&str> = rep.findings.iter().map(|f| f.kind.as_str()).collect();
    let mut kinds_b: Vec<&str> = swapped.findings.iter().map(|f| f.kind.as_str()).collect();
    kinds_a.sort();
    kinds_b.sort();
    assert_eq!(kinds_a, kinds_b, "finding kinds must be order-symmetric");
}

/// merge-modify-delete-flagged: delete vs untouched deletes cleanly;
/// delete vs edit flags a conflict and the edited text stays in the
/// relint'd union — in both argument orders.
#[hegel::test(test_cases = 16, derandomize = true)]
fn merge_modify_delete_flagged_law(tc: TestCase) {
    let seed = seed_of(&tc, 300);
    let base_corpus = two_file_corpus(seed);
    assert_case_fresh(&base_corpus, "modify/delete base");
    let base = base_corpus.files.clone();
    let beta_expr = base_corpus.models[1].constraints[0].expr.clone();

    // Leg 1 — one branch deletes, the other leaves the file untouched:
    // deletion wins, no CONFLICT-class finding names the file. (A pure
    // deletion does fire rename_replay flags — the deleted file's own
    // guard citations are references on the untouched branch — but those
    // are needs_review-class flags, never a conflict.)
    let mut a = base.clone();
    delete_file(&mut a, "demo-beta.md");
    let rep = merge::run(&base, &a, &base);
    assert!(
        !rep.findings.iter().any(|f| matches!(
            f.kind.as_str(),
            "textual_conflict" | "id_collision" | "relint_failure" | "unparsable"
        )),
        "untouched counterpart must not conflict: {:?}",
        rep.findings
    );
    assert!(
        !rep.findings
            .iter()
            .any(|f| f.kind == "textual_conflict" && f.message.contains("demo-beta.md"))
    );
    // Symmetric order.
    let rep_swapped = merge::run(&base, &base, &a);
    assert!(
        !rep_swapped
            .findings
            .iter()
            .any(|f| f.kind == "textual_conflict" && f.message.contains("demo-beta.md"))
    );

    // Leg 2 — one branch deletes, the other EDITS (and the edit carries a
    // dangling reference): textual_conflict names the file AND the edited
    // text is what relint saw (the relint_failure proves it).
    let mut b = base.clone();
    edit_expr(&mut b, "demo-beta.md", &beta_expr, "edited while a deleted");
    inject_dangling_traces(&mut b, "demo-beta.md", &["c0"]);
    let rep = merge::run(&base, &a, &b);
    assert_eq!(rep.verdict, "failed");
    assert!(
        rep.findings
            .iter()
            .any(|f| f.kind == "textual_conflict" && f.message.contains("demo-beta.md"))
    );
    assert!(
        rep.findings.iter().any(|f| f.kind == "relint_failure"),
        "the kept edited text must be relint'd: {:?}",
        rep.findings
    );
    // Both orders flag.
    let rep_swapped = merge::run(&base, &b, &a);
    assert!(
        rep_swapped
            .findings
            .iter()
            .any(|f| f.kind == "textual_conflict" && f.message.contains("demo-beta.md"))
    );
}

/// merge-textual-conflict-flagged: both-tips-different-edits on a base
/// file is a named `textual_conflict` and failed; identical edits and
/// one-sided edits are not.
#[hegel::test(test_cases = 16, derandomize = true)]
fn merge_textual_conflict_flagged_law(tc: TestCase) {
    let seed = seed_of(&tc, 400);
    let base_corpus = two_file_corpus(seed);
    assert_case_fresh(&base_corpus, "textual base");
    let base = base_corpus.files.clone();
    let alpha_expr = base_corpus.models[0].constraints[0].expr.clone();

    // Different edits on both branches: named conflict, failed verdict.
    let mut a = base.clone();
    edit_expr(&mut a, "demo-alpha.md", &alpha_expr, "branch a wording");
    let mut b = base.clone();
    edit_expr(&mut b, "demo-alpha.md", &alpha_expr, "branch b wording");
    let rep = merge::run(&base, &a, &b);
    assert_eq!(rep.verdict, "failed");
    assert!(
        rep.findings
            .iter()
            .any(|f| f.kind == "textual_conflict" && f.message.contains("demo-alpha.md"))
    );

    // Identical edits: both differ from base but agree with each other —
    // no textual conflict (the id-collision law owns this case).
    let mut a2 = base.clone();
    edit_expr(&mut a2, "demo-alpha.md", &alpha_expr, "shared new wording");
    let b2 = a2.clone();
    let rep = merge::run(&base, &a2, &b2);
    assert!(!rep.findings.iter().any(|f| f.kind == "textual_conflict"));

    // One-sided edit: no textual conflict at all.
    let mut a3 = base.clone();
    edit_expr(&mut a3, "demo-alpha.md", &alpha_expr, "one-sided wording");
    let rep = merge::run(&base, &a3, &base);
    assert!(!rep.findings.iter().any(|f| f.kind == "textual_conflict"));
}

/// merge-relint-gates-verdict: a relint failure dominates a blast-radius
/// intersection — `failed`, with both findings present.
#[hegel::test(test_cases = 16, derandomize = true)]
fn merge_relint_gates_verdict_law(tc: TestCase) {
    let seed = seed_of(&tc, 500);
    let base_corpus = two_file_corpus(seed);
    assert_case_fresh(&base_corpus, "relint base");
    let base = base_corpus.files.clone();
    let alpha_expr = base_corpus.models[0].constraints[0].expr.clone();

    // A edits demo-alpha.md AND injects a dangling [[ghost]] trace.
    let mut a = base.clone();
    edit_expr(&mut a, "demo-alpha.md", &alpha_expr, "touched and broken");
    inject_dangling_traces(&mut a, "demo-alpha.md", &["c0"]);
    // B adds a referencing file → the blast radii also intersect.
    let target = "demo.alpha".to_string();
    let gamma = build_file("demo.gamma", seed + 41, Some(target));
    let mut b = base.clone();
    b.push(placed("demo-gamma.md", &gamma));

    let rep = merge::run(&base, &a, &b);
    assert_eq!(
        rep.verdict, "failed",
        "relint failure must dominate needs_review: {:?}",
        rep.findings
    );
    assert!(rep.findings.iter().any(|f| f.kind == "relint_failure"));
    assert!(
        rep.findings
            .iter()
            .any(|f| f.kind == "blast_radius_intersection"),
        "both findings must be reported (additive): {:?}",
        rep.findings
    );
}

/// merge-needs-review-reachable (generator guard): the constructed
/// blast-radius scenario must land exactly on the middle verdict —
/// if the generator stopped reaching it, this guard goes red.
#[hegel::test(test_cases = 16, derandomize = true)]
fn merge_needs_review_reachable_guard(tc: TestCase) {
    let seed = seed_of(&tc, 600);
    let base_corpus = two_file_corpus(seed);
    assert_case_fresh(&base_corpus, "guard base");
    let base = base_corpus.files.clone();
    let alpha_expr = base_corpus.models[0].constraints[0].expr.clone();

    let mut a = base.clone();
    edit_expr(&mut a, "demo-alpha.md", &alpha_expr, "disjoint edit a");
    let target = "demo.alpha".to_string();
    let gamma = build_file("demo.gamma", seed + 53, Some(target));
    let mut b = base.clone();
    b.push(placed("demo-gamma.md", &gamma));

    let rep = merge::run(&base, &a, &b);
    tc.event("needs_review_reached");
    assert_eq!(rep.verdict, "needs_review", "{:?}", rep.findings);
    let mut kinds: Vec<&str> = rep.findings.iter().map(|f| f.kind.as_str()).collect();
    kinds.sort();
    assert_eq!(
        kinds,
        vec!["blast_radius_intersection"],
        "no failed-class finding may ride along: {:?}",
        rep.findings
    );
}

/// merge-verdict-symmetry: RED at HEAD (specodelic-1izc); enabled once
/// the add/add hole is fixed — the ticket's remedy: flag differing-text
/// add/add as a conflict before union_tree picks a side, and keep the
/// relint off the contested paths so the finding set cannot inherit
/// the silent choice.
#[test]
fn merge_verdict_symmetry_add_add() {
    // Empty ancestor; both branches create demo-x.md with the same intent
    // id; branch b's copy carries a dangling [[ghost]] trace. Filenames
    // match their intent so the only asymmetry source is the union
    // tree's side choice on add/add.
    let x = render(&build_file("demo.x", 11, None));
    let mut ytree: Tree = vec![("demo-x.md".into(), render(&build_file("demo.x", 11, None)))];
    inject_dangling_traces(&mut ytree, "demo-x.md", &["c0"]);
    let y = ytree.remove(0).1;
    let base: Tree = vec![];
    let a: Tree = vec![("demo-x.md".into(), x)];
    let b: Tree = vec![("demo-x.md".into(), y)];

    let ab = merge::run(&base, &a, &b);
    let ba = merge::run(&base, &b, &a);
    assert_eq!(
        ab.verdict, ba.verdict,
        "same history, opposite verdicts (a→b: {:?}, b→a: {:?})",
        ab.findings, ba.findings
    );
    let mut kinds_ab: Vec<&str> = ab.findings.iter().map(|f| f.kind.as_str()).collect();
    let mut kinds_ba: Vec<&str> = ba.findings.iter().map(|f| f.kind.as_str()).collect();
    kinds_ab.sort();
    kinds_ba.sort();
    assert_eq!(kinds_ab, kinds_ba);
}

// ---------------------------------------------------------------------------
// Rename laws (Tier 1 + Tier 2)
// ---------------------------------------------------------------------------

/// A corpus whose row ids collide with closed-enum kind keywords — the
/// exact shape specodelic-77mq corrupted before the deny-set fix.
fn kind_colliding_corpus(seed: u64) -> Corpus {
    let mut corpus = build_corpus(seed | 1);
    let m = &mut corpus.models[0];
    m.constraints[0].local = "unit".to_string();
    if m.constraints.len() > 1 {
        m.constraints[1].local = "invariant".to_string();
    }
    if !m.properties.is_empty() {
        m.properties[0].local = "law".to_string();
    }
    if !m.transitions.is_empty() {
        m.transitions[0].local = "effect".to_string();
    }
    corpus.files = corpus
        .models
        .iter()
        .map(|mm| (format!("{}.md", mm.stem()), render(mm)))
        .collect();
    corpus
}

/// rename-kind-preserved: a row rename changes the renamed row's id cell
/// and no `kind` cell of any row — post-77mq this must hold universally,
/// including for ids equal to kind keywords.
#[hegel::test(test_cases = 16, derandomize = true)]
fn rename_kind_preserved_law(tc: TestCase) {
    let seed = seed_of(&tc, 700);
    let corpus = kind_colliding_corpus(seed);
    assert_case_fresh(&corpus, "kind-colliding corpus");

    let files = files_as_pathbuf(&corpus.files);
    let before: Vec<Spec> = corpus
        .files
        .iter()
        .map(|(p, t)| spec_with_path(p, t))
        .collect();

    let outcome = rename::run(&files, "demo.alpha.unit", "demo.alpha.renamed")
        .unwrap_or_else(|e| panic!("rename must be accepted: {e:?}"));
    let after_map = materialize(&files.into_iter().collect::<BTreeMap<_, _>>(), &outcome);

    for (path, text) in &after_map {
        let p = path.display().to_string();
        let before_spec = before
            .iter()
            .find(|s| s.path.as_ref().unwrap().to_str() == Some(&p))
            .or_else(|| {
                // The renamed row's file has identical bytes except the
                // id cell, so index by stem (no intent renames here).
                before
                    .iter()
                    .find(|s| s.path.as_ref().unwrap().file_stem() == path.file_stem())
            })
            .unwrap_or_else(|| panic!("no pre-state for {p}"));
        let after_spec = parse_ok(&p, text);
        assert_eq!(
            after_spec.intent.id, before_spec.intent.id,
            "row renames never move the intent"
        );
        assert_table_rows_preserved(
            &p,
            "constraints",
            &before_spec.constraints,
            &after_spec.constraints,
        );
        assert_table_rows_preserved(&p, "states", &before_spec.states, &after_spec.states);
        assert_table_rows_preserved(
            &p,
            "properties",
            &before_spec.properties,
            &after_spec.properties,
        );
        for (b, a) in before_spec.transitions.iter().zip(&after_spec.transitions) {
            assert_transition_preserved(&p, b, a);
        }
    }
}

/// Per-table half of the kind-preserved law: `kind` cells byte-identical
/// for every row; every other cell either is untouched or follows the
/// rename (the row's own id cell, typed-reference cells naming the old
/// id).
fn assert_table_rows_preserved(
    p: &str,
    table: &str,
    rows_before: &[specodelic::spec::Row],
    rows_after: &[specodelic::spec::Row],
) {
    assert_eq!(
        rows_before.len(),
        rows_after.len(),
        "{p}: {table} row count drifted"
    );
    for (b, a) in rows_before.iter().zip(rows_after) {
        assert_eq!(
            a.cells.get("kind"),
            b.cells.get("kind"),
            "{p}: {table} row {} kind cell changed",
            b.id,
        );
        for (col, val) in &b.cells {
            let expected = if col == "kind" {
                val.clone()
            } else if val == "unit" {
                "renamed".to_string()
            } else {
                val.replace("demo.alpha.unit", "demo.alpha.renamed")
            };
            assert_eq!(
                a.cells.get(col),
                Some(&expected),
                "{p}: {table} row {} col {col} changed unexpectedly",
                b.id
            );
        }
        let expected_id = if b.id == "unit" {
            "renamed"
        } else {
            b.id.as_str()
        };
        assert_eq!(a.id, expected_id, "{p}: {table} id drifted");
    }
}

/// Transition half of the kind-preserved law: endpoints and the cited
/// guard follow the rename, nothing else moves.
fn assert_transition_preserved(
    p: &str,
    b: &specodelic::spec::Transition,
    a: &specodelic::spec::Transition,
) {
    assert_eq!(a.id, b.id, "{p}: transition id changed");
    assert_eq!(a.from, b.from, "{p}: transition {} from changed", b.id);
    assert_eq!(a.to, b.to, "{p}: transition {} to changed", b.id);
    let expected_guard = b
        .guard
        .as_ref()
        .map(|g| g.replace("demo.alpha.unit", "demo.alpha.renamed"));
    assert_eq!(
        a.guard, expected_guard,
        "{p}: transition {} guard changed unexpectedly",
        b.id
    );
}

/// rename-collision-refused: any existing id is refused with
/// `Collision` naming the holder; a fresh id in the same family is
/// accepted.
#[hegel::test(test_cases = 16, derandomize = true)]
fn rename_collision_refused_law(tc: TestCase) {
    let seed = seed_of(&tc, 800);
    let corpus = two_file_corpus(seed);
    assert_case_fresh(&corpus, "collision corpus");
    let files = files_as_pathbuf(&corpus.files);

    // Leg 1 — the other file's Intent id.
    let err = rename::run(&files, "demo.alpha", "demo.beta")
        .expect_err("renaming onto a defined intent must be refused");
    match err {
        RenameError::Collision { new_id, holder } => {
            assert_eq!(new_id, "demo.beta");
            assert!(
                holder.contains("demo-beta.md"),
                "holder must name the defining file: {holder}"
            );
        }
        other => panic!("expected Collision, got {other:?}"),
    }

    // Leg 2 — a qualified row id of the other file.
    let err = rename::run(&files, "demo.alpha.c0", "demo.beta.c0")
        .expect_err("renaming onto a defined row must be refused");
    assert!(matches!(err, RenameError::Collision { .. }), "{err:?}");

    // Leg 3 — a transition id (states/transitions are definitions too).
    let err = rename::run(&files, "demo.alpha.c0", "demo.alpha.t0")
        .expect_err("renaming onto a defined transition must be refused");
    assert!(matches!(err, RenameError::Collision { .. }), "{err:?}");

    // Leg 4 — a fresh id in the same family is accepted.
    let outcome = rename::run(&files, "demo.alpha.c0", "demo.alpha.c9")
        .unwrap_or_else(|e| panic!("fresh target must be accepted: {e:?}"));
    assert!(!outcome.writes.is_empty());
}

/// rename-composition-associative: rename(a→b) then rename(b→c) equals
/// the direct rename(a→c), byte for byte, for Intent and row renames.
#[hegel::test(test_cases = 16, derandomize = true)]
fn rename_composition_associative_law(tc: TestCase) {
    let seed = seed_of(&tc, 900);
    let corpus = two_file_corpus(seed);
    assert_case_fresh(&corpus, "composition corpus");

    // Leg 1 — Intent renames (file moves compose).
    let files = files_as_pathbuf(&corpus.files);
    let start: BTreeMap<PathBuf, String> = files.iter().cloned().collect();
    let step1 = rename::run(&files, "demo.alpha", "demo.bravo")
        .unwrap_or_else(|e| panic!("intent rename a→b refused: {e:?}"));
    let mid = materialize(&start, &step1);
    for (p, t) in &mid {
        parse_ok(&p.display().to_string(), t);
    }
    let step2 = rename::run(&as_files(&mid), "demo.bravo", "demo.charlie")
        .unwrap_or_else(|e| panic!("intent rename b→c refused: {e:?}"));
    let composed = materialize(&mid, &step2);
    let direct_out = rename::run(&files, "demo.alpha", "demo.charlie")
        .unwrap_or_else(|e| panic!("direct intent rename refused: {e:?}"));
    let direct = materialize(&start, &direct_out);
    assert_eq!(composed, direct, "chained intent rename != direct rename");

    // Leg 2 — row renames (same intent namespace).
    let step1 = rename::run(&files, "demo.alpha.c0", "demo.alpha.mid")
        .unwrap_or_else(|e| panic!("row rename a→b refused: {e:?}"));
    let mid = materialize(&start, &step1);
    for (p, t) in &mid {
        parse_ok(&p.display().to_string(), t);
    }
    let step2 = rename::run(&as_files(&mid), "demo.alpha.mid", "demo.alpha.fin")
        .unwrap_or_else(|e| panic!("row rename b→c refused: {e:?}"));
    let composed = materialize(&mid, &step2);
    let direct_out = rename::run(&files, "demo.alpha.c0", "demo.alpha.fin")
        .unwrap_or_else(|e| panic!("direct row rename refused: {e:?}"));
    let direct = materialize(&start, &direct_out);
    assert_eq!(composed, direct, "chained row rename != direct rename");
}

// ---------------------------------------------------------------------------
// Writer laws (Tier 1 + Tier 2)
// ---------------------------------------------------------------------------

/// writer-untouched-bytes-preserved: the emitted text differs from the
/// source only at the replacement spans the parser's own recording
/// names; prose, CRLF terminators, and padding pass through byte for
/// byte, and columns are never realigned.
#[hegel::test(test_cases = 16, derandomize = true)]
fn writer_untouched_bytes_preserved_law(tc: TestCase) {
    let seed = seed_of(&tc, 1000);
    let corpus = two_file_corpus(seed);
    assert_case_fresh(&corpus, "untouched-bytes corpus");
    let path = pb("demo-alpha.md");
    let source = corpus
        .files
        .iter()
        .find(|(p, _)| p == "demo-alpha.md")
        .unwrap()
        .1
        .clone();
    let spec = spec_with_path("demo-alpha.md", &source);
    let crlf = source.contains("\r\n");

    // Leg 1 — row rename through the writer.
    let edit = Edit::Rename {
        old: "demo.alpha.c0".into(),
        new: "demo.alpha.cx".into(),
        local: Some(("c0".into(), "cx".into())),
    };
    let ws: WriteSet = writer::apply(&edit, &path, &source, &spec)
        .unwrap_or_else(|e| panic!("writer apply refused: {e:?}"));
    assert_eq!(ws.writes.len(), 1);
    assert!(ws.removals.is_empty());
    let emitted = &ws.writes[0].1;
    let repls = expected_repls(&spec, "demo.alpha.c0", "demo.alpha.cx", Some(("c0", "cx")));
    assert!(
        !repls.is_empty(),
        "oracle produced no replacements — spans unrecorded"
    );
    assert_eq!(*emitted, splice(&source, &repls), "emission != span splice");
    assert_untouched_outside(&source, emitted, &repls);
    assert_eq!(
        emitted.contains("\r\n"),
        crlf,
        "line terminators must not be normalized"
    );
    // Prose canary: the paragraphs mention demo.alpha/c0 as bare words.
    for para in &corpus.models[0].prose {
        assert!(
            emitted.contains(para),
            "prose paragraph was touched by the edit"
        );
    }

    // Leg 2 — intent rename through the writer (frontmatter id + links;
    // the file moves, prose and bullets stay).
    let edit = Edit::Rename {
        old: "demo.alpha".into(),
        new: "demo.zeta".into(),
        local: None,
    };
    let ws = writer::apply(&edit, &path, &source, &spec)
        .unwrap_or_else(|e| panic!("writer apply (intent) refused: {e:?}"));
    assert_eq!(ws.writes.len(), 1, "intent rename writes the moved file");
    assert_eq!(ws.removals, vec![path.clone()], "old path is removed");
    assert_eq!(
        ws.writes[0].0,
        pb("demo-zeta.md"),
        "filename follows the id"
    );
    let emitted = &ws.writes[0].1;
    let repls = expected_repls(&spec, "demo.alpha", "demo.zeta", None);
    assert_eq!(*emitted, splice(&source, &repls));
    assert_untouched_outside(&source, emitted, &repls);
    for para in &corpus.models[0].prose {
        assert!(emitted.contains(para), "prose touched by intent rename");
    }
}

/// writer-edit-composition: apply(a→b) reparse apply(b→c) equals the
/// direct apply(a→c), for row and Intent renames (paths included).
#[hegel::test(test_cases = 16, derandomize = true)]
fn writer_edit_composition_law(tc: TestCase) {
    let seed = seed_of(&tc, 1100);
    let corpus = two_file_corpus(seed);
    assert_case_fresh(&corpus, "composition corpus");
    let path = pb("demo-alpha.md");
    let source = corpus
        .files
        .iter()
        .find(|(p, _)| p == "demo-alpha.md")
        .unwrap()
        .1
        .clone();
    let spec = spec_with_path("demo-alpha.md", &source);

    // Row leg.
    let e1 = Edit::Rename {
        old: "demo.alpha.c0".into(),
        new: "demo.alpha.mid".into(),
        local: Some(("c0".into(), "mid".into())),
    };
    let s1 = writer::apply(&e1, &path, &source, &spec)
        .unwrap_or_else(|e| panic!("step 1 refused: {e:?}"));
    let text1 = &s1.writes[0].1;
    let spec1 = parse_ok("demo-alpha.md", text1);
    let e2 = Edit::Rename {
        old: "demo.alpha.mid".into(),
        new: "demo.alpha.fin".into(),
        local: Some(("mid".into(), "fin".into())),
    };
    let s2 = writer::apply(&e2, &s1.writes[0].0, text1, &spec1)
        .unwrap_or_else(|e| panic!("step 2 refused: {e:?}"));
    let e_direct = Edit::Rename {
        old: "demo.alpha.c0".into(),
        new: "demo.alpha.fin".into(),
        local: Some(("c0".into(), "fin".into())),
    };
    let direct = writer::apply(&e_direct, &path, &source, &spec)
        .unwrap_or_else(|e| panic!("direct refused: {e:?}"));
    assert_eq!(s2.writes[0].0, direct.writes[0].0);
    assert_eq!(
        s2.writes[0].1, direct.writes[0].1,
        "row composition drifted"
    );
    assert_eq!(s2.removals, direct.removals);

    // Intent leg (file moves compose: alpha → bravo → charlie).
    let e1 = Edit::Rename {
        old: "demo.alpha".into(),
        new: "demo.bravo".into(),
        local: None,
    };
    let s1 = writer::apply(&e1, &path, &source, &spec)
        .unwrap_or_else(|e| panic!("intent step 1 refused: {e:?}"));
    let text1 = &s1.writes[0].1;
    let path1 = s1.writes[0].0.clone();
    let spec1 = parse_ok(&path1.display().to_string(), text1);
    let e2 = Edit::Rename {
        old: "demo.bravo".into(),
        new: "demo.charlie".into(),
        local: None,
    };
    let s2 = writer::apply(&e2, &path1, text1, &spec1)
        .unwrap_or_else(|e| panic!("intent step 2 refused: {e:?}"));
    let e_direct = Edit::Rename {
        old: "demo.alpha".into(),
        new: "demo.charlie".into(),
        local: None,
    };
    let direct = writer::apply(&e_direct, &path, &source, &spec)
        .unwrap_or_else(|e| panic!("intent direct refused: {e:?}"));
    assert_eq!(s2.writes[0].0, direct.writes[0].0, "composed path drifted");
    assert_eq!(
        s2.writes[0].1, direct.writes[0].1,
        "intent composition drifted"
    );
    // Each step removes its own input path; the composed NET removal of
    // the original alpha path happened at step 1 (s1.removals). Final
    // contents and final path are the composition claim.
    assert_eq!(s2.removals, vec![path1.clone()]);
    assert_eq!(s1.removals, vec![path.clone()]);
    assert_eq!(direct.removals, vec![path.clone()]);
}

/// writer-span-coherence: every recorded span resolves in bounds to the
/// bytes of its parsed element (`emit` == Ok); a source that no longer
/// matches its parse is a labeled refusal, never a splice.
#[hegel::test(test_cases = 16, derandomize = true)]
fn writer_span_coherence_law(tc: TestCase) {
    let seed = seed_of(&tc, 1200);
    let corpus = build_corpus(seed);
    assert_case_fresh(&corpus, "span-coherence corpus");

    for (path, text) in &corpus.files {
        let spec = spec_with_path(path, text);
        writer::emit(text, &spec)
            .unwrap_or_else(|e| panic!("{path}: emit refused a fresh parse: {e:?}"));

        // Corruption legs — same parse, mutated source.
        let mutated_insert = format!("X{text}");
        let err = writer::emit(&mutated_insert, &spec).expect_err("shifted source must refuse");
        assert_eq!(err.label, "spec.span_failure");

        let cut = text.len() / 2;
        let mutated_trunc = &text[..cut];
        if writer::emit(mutated_trunc, &spec).is_ok() {
            panic!("{path}: truncated source emitted cleanly");
        }
    }
}

/// writer-span-failure-labeled: stale-source refusals carry the exact
/// `spec.span_failure` label with non-empty detail and remediation, and
/// never panic — on emit and on non-identity apply.
#[hegel::test(test_cases = 16, derandomize = true)]
fn writer_span_failure_labeled_law(tc: TestCase) {
    let seed = seed_of(&tc, 1300);
    let corpus = two_file_corpus(seed);
    assert_case_fresh(&corpus, "span-failure corpus");
    let (path, text) = corpus.files[0].clone();
    let spec = spec_with_path(&path, &text);

    let check_label = |what: &str, e: &writer::WriterError| {
        assert_eq!(e.label, "spec.span_failure", "{what}: wrong label");
        assert!(!e.detail.is_empty(), "{what}: empty detail");
        assert!(!e.remediation.is_empty(), "{what}: empty remediation");
    };

    // Insert-at-0 shifts every span.
    let mutated = format!("Z{text}");
    match writer::emit(&mutated, &spec) {
        Err(e) => check_label("emit/insert", &e),
        Ok(_) => panic!("emit accepted a shifted source"),
    }
    let edit = Edit::Rename {
        old: "demo.alpha.c0".into(),
        new: "demo.alpha.cx".into(),
        local: Some(("c0".into(), "cx".into())),
    };
    match writer::apply(&edit, &pb(&path), &mutated, &spec) {
        Err(e) => check_label("apply/insert", &e),
        Ok(ws) => panic!("apply accepted a shifted source: {ws:?}"),
    }

    // Truncation drops recorded spans out of bounds.
    let truncated = &text[..text.len() / 3];
    match writer::emit(truncated, &spec) {
        Err(e) => check_label("emit/truncate", &e),
        Ok(_) => panic!("emit accepted a truncated source"),
    }
}

// ---------------------------------------------------------------------------
// Compile laws (Tier 2)
// ---------------------------------------------------------------------------

/// compile-artifact-determinism: compile is a pure function of the
/// parsed spec — byte-identical artifacts across repeated calls and
/// across a reparse of the same source, with equal IR and warnings.
#[hegel::test(test_cases = 16, derandomize = true)]
fn compile_artifact_determinism_law(tc: TestCase) {
    let seed = seed_of(&tc, 1400);
    let corpus = build_corpus(seed);
    assert_case_fresh(&corpus, "determinism corpus");

    for (path, text) in &corpus.files {
        let spec = spec_with_path(path, text);
        let c1 = compile::compile_spec(&spec)
            .unwrap_or_else(|e| panic!("{path}: first compile failed: {e:?}"));
        let c2 = compile::compile_spec(&spec)
            .unwrap_or_else(|e| panic!("{path}: second compile failed: {e:?}"));
        let reparsed = parse_ok(path, text);
        let c3 = compile::compile_spec(&reparsed)
            .unwrap_or_else(|e| panic!("{path}: reparse compile failed: {e:?}"));
        assert_eq!(c1.toml, c2.toml, "{path}: toml nondeterministic");
        assert_eq!(c1.toml, c3.toml, "{path}: toml differs across reparse");
        assert_eq!(c1.props, c2.props, "{path}: props nondeterministic");
        assert_eq!(c1.props, c3.props, "{path}: props differs across reparse");
        assert_eq!(c1.tla, c2.tla, "{path}: tla nondeterministic");
        assert_eq!(c1.tla, c3.tla, "{path}: tla differs across reparse");
        assert_eq!(c1.model_ir, c2.model_ir, "{path}: IR nondeterministic");
        assert_eq!(
            c1.model_ir, c3.model_ir,
            "{path}: IR differs across reparse"
        );
        assert_eq!(
            c1.warnings, c2.warnings,
            "{path}: warnings nondeterministic"
        );
        assert_eq!(
            c1.warnings, c3.warnings,
            "{path}: warnings differ across reparse"
        );
    }
}

/// compile-artifact-stem-collision-refusal: duplicate artifact stems are
/// refused naming both paths and the escape; a repeated canonical path
/// is one file; the result set is order-independent.
#[hegel::test(test_cases = 8, derandomize = true)]
fn compile_artifact_stem_collision_law(tc: TestCase) {
    let seed = seed_of(&tc, 1500);
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let root = dir.path();

    let write_spec = |rel: &str, intent: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        let m: ModelFile = build_file(intent, seed, None);
        std::fs::write(&p, render(&m)).unwrap();
        let mut s = parse_ok(rel, &std::fs::read_to_string(&p).unwrap());
        s.path = Some(p);
        s
    };

    // Collision: same basename in two directories.
    let a = write_spec("a/pipeline.md", "demo.pipeline");
    let b = write_spec("b/pipeline.md", "demo.pipeline");
    let hit = compile::artifact_stem_collision(&[a.clone(), b.clone()])
        .expect("same-basename pair must collide");
    assert!(hit.0.contains("duplicate_artifact_stem"));
    assert!(hit.0.contains("a/pipeline.md") || hit.0.contains("b/pipeline.md"));
    assert!(!hit.1.is_empty(), "remediation must be non-empty");
    // Order-independent: the named path SET is the same both ways.
    let hit_swapped =
        compile::artifact_stem_collision(&[b.clone(), a.clone()]).expect("swap must still hit");
    let named = |msg: &str| -> Vec<String> {
        msg.split("produced by both ")
            .nth(1)
            .unwrap()
            .split(" — ")
            .next()
            .unwrap()
            .split(" and ")
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    };
    let mut set1 = named(&hit.0);
    let mut set2 = named(&hit_swapped.0);
    set1.sort();
    set2.sort();
    assert_eq!(set1, set2, "collision set must be order-independent");

    // Repeated canonical path: one file, no collision.
    let again = write_spec("a/pipeline.md", "demo.pipeline");
    assert!(
        compile::artifact_stem_collision(&[a.clone(), again]).is_none(),
        "the same canonical path twice is one file"
    );

    // spec.md special case: the stem is the Intent id (dots → dashes) —
    // it collides with a same-stem sibling file.
    let s1 = write_spec("c/spec.md", "demo.pipeline");
    let s2 = write_spec("d/demo-pipeline.md", "demo.other");
    assert!(
        compile::artifact_stem_collision(&[s1, s2]).is_some(),
        "spec.md intent-id stem must collide with a same-stem file"
    );

    // Pathless synthetic specs share the intent-id fallback stem.
    let m = build_file("demo.solo", seed, None);
    let mut p1 = parse_ok("<demo.solo>", &render(&m));
    p1.path = None;
    let m2 = build_file("demo.solo", seed + 3, None);
    let mut p2 = parse_ok("<demo.solo>", &render(&m2));
    p2.path = None;
    assert!(
        compile::artifact_stem_collision(&[p1, p2]).is_some(),
        "two pathless specs with the same stem must collide"
    );
}

// ---------------------------------------------------------------------------
// Generator guards (sometimes legs)
// ---------------------------------------------------------------------------

/// rename-success-reachable (guard): the generated distribution reaches
/// the accepted-rename path for both shapes — intent (file move) and
/// row — with non-empty write-sets.
#[hegel::test(test_cases = 16, derandomize = true)]
fn rename_success_reachable_guard(tc: TestCase) {
    let seed = seed_of(&tc, 1600);
    let corpus = two_file_corpus(seed);
    assert_case_fresh(&corpus, "guard corpus");
    let files = files_as_pathbuf(&corpus.files);

    // Intent leg: file moved, one removal, links rewritten.
    let out = rename::run(&files, "demo.alpha", "demo.zeta")
        .unwrap_or_else(|e| panic!("generated intent rename refused: {e:?}"));
    tc.event("intent_rename_accepted");
    assert!(
        !out.writes.is_empty(),
        "intent rename must write the moved file"
    );
    assert_eq!(out.remove, Some(pb("demo-alpha.md")));

    // Row leg: owner file's id cell + every link change.
    let out = rename::run(&files, "demo.alpha.c0", "demo.alpha.cx")
        .unwrap_or_else(|e| panic!("generated row rename refused: {e:?}"));
    tc.event("row_rename_accepted");
    assert!(
        !out.writes.is_empty(),
        "row rename must write the owner file"
    );
    assert_eq!(out.remove, None);
}

// ---------------------------------------------------------------------------
// Suite self-check (always runs first in practice, and names generator
// defects as generator bugs): a fixed seed sanity corpus must pass the
// full oracle.
// ---------------------------------------------------------------------------

#[test]
fn generator_self_check_fixed_seed() {
    for seed in [1u64, 2, 3, 7, 11, 12, 42] {
        let corpus = build_corpus(seed);
        check_corpus(&corpus)
            .unwrap_or_else(|e| panic!("generator self-check failed (seed {seed}): {e}"));
        let colliding = kind_colliding_corpus(seed);
        check_corpus(&colliding).unwrap_or_else(|e| {
            panic!("kind-colliding corpus self-check failed (seed {seed}): {e}")
        });
    }
}
