//! The min-expr kernel — the decidable bounded logic over the acset
//! instances I(k) (openspec change `add-min-expr-kernel`, beads
//! specodelic-7ga, tasks.md §3).
//!
//! Purpose: give the corpus's data-dependent cells — the equational and
//! bounded-quantified ones that are prose-only — defined, decidable
//! semantics. Responsibilities: the closed v0 atomic grammar (D1:
//! `==`, comparisons, bounded ∀/∃ over I(k), ∧/¬, and the
//! reference-typed atomics `resolves`/`unique`/`acyclic`/`reachable`),
//! row-typed parsing against the canonical schema (Schema-as-data), the
//! grounding table mapping each atomic to its proven machinery (D1 —
//! an atomic without a grounding entry cannot ship), evaluation over a
//! `KernelEnv` yielding the slice-1 `ThreeValued` status (reused from
//! the citation algebra — never re-invented), the corpus claim pass
//! (§3.7: complete-corpus kernel evaluation plus `merge_corpus_statuses`,
//! the ONE shared command-path identity/merge helper both command paths
//! call — §3.8 TIDY), and the predicate
//! registry seam for later pack registration (D8's decidability gate —
//! nothing outside the v0 set registers here). Rationale: kernel
//! semantics never outruns proven machinery — every atomic names its
//! grounding (graph traversal, acset traversal, model_check bounded
//! evaluation), so evaluation is total over finite instances and
//! `unknown` stays honest (dl/1 Kleene absorb: unknown never coerces
//! to pass).
//!
//! Grammar of record (v0, closed):
//!
//! ```text
//! expr   := unary ('∧' unary)*
//! unary  := '¬' unary | primary
//! primary:= '∀' var '∈' Object ':' expr
//!         | '∃' var '∈' Object ':' expr
//!         | '(' expr ')'
//!         | 'resolves' '(' Morphism ')' | 'unique' '(' Morphism ')'
//!         | 'acyclic' '(' Morphism ')'
//!         | 'reachable' '(' Id ',' Id ',' Morphism ')'
//!         | term ('==' | '<' | '<=' | '>' | '>=') term
//! term   := '|' Object '|' | Int | '⊥' | var | Id | var '.' Morphism
//! ```
//!
//! Opt-in boundary (pure widening): a cell opts in with the
//! `**kernel:**` marker in fragment position (sd1 rule carried over:
//! cell start or span-opening); anything else — prose, citation
//! expressions, executable `**lang:**` fragments — stays exactly as
//! before. The corpus's existing ∀-led prose cells (specodelic.md's
//! `unique_id`, merge.md's `no_new_id_collision`) carry no marker and
//! compile byte-identically; they are phase 6's migration targets,
//! not phase 3's.

use std::collections::BTreeMap;

use crate::acset::instance::Instance;
use crate::acset::query;
use crate::acset::schema::{self, Schema};
use crate::compile::{self, ThreeValued};
use crate::graph::kind_index;
use crate::spec::Spec;

// ---------------------------------------------------------------------------
// The closed grammar
// ---------------------------------------------------------------------------

/// A kernel term: the bounded value space over an instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    /// A quantifier-bound variable (evaluates to the bound row id).
    Var(String),
    /// A bare id literal (never a bound variable; unqualified ids).
    Id(String),
    /// An integer literal.
    Int(i64),
    /// The cardinality of one object's instance `|I(k)|` — bounded
    /// evaluation reads it off the acset.
    Card(String),
    /// A variable's partial morphism value `var.morphism` — the target
    /// id, or dangling (⊥) when unresolved.
    Proj(String, String),
    /// The dangling value ⊥ (`x.m == ⊥` asks whether a reference
    /// resolves).
    Dangling,
}

/// The comparison operators of the closed set (equality is `==`; these
/// four are the ordering comparisons).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Lt,
    Le,
    Gt,
    Ge,
}

/// A v0 atomic — every variant names its grounding machinery in the
/// grounding table (D1: an atomic without a grounding entry cannot
/// ship).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atomic {
    /// `resolves(m)` — every value of morphism m resolves (no dangling):
    /// acset traversal.
    Resolves(String),
    /// `unique(m)` — the defined values of m are pairwise distinct
    /// (injective): acset traversal.
    Unique(String),
    /// `acyclic(m)` — the endo morphism m admits no cycle: graph
    /// traversal (morphism-restricted `cyclic_nodes`).
    Acyclic(String),
    /// `reachable(a, b, m)` — b is in the forward closure of a through
    /// m: graph traversal (`forward_closure`). An unknown seed is
    /// `unknown`, never a fabricated verdict.
    Reachable(String, String, String),
    /// `t1 == t2` — bounded evaluation over the finite instance.
    Eq(Term, Term),
    /// `t1 op t2` for an ordering comparison — bounded evaluation.
    Cmp(Term, CmpOp, Term),
}

/// A kernel expression over the closed grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelExpr {
    Atomic(Atomic),
    Not(Box<KernelExpr>),
    And(Box<KernelExpr>, Box<KernelExpr>),
    /// `∀ v ∈ k: body` — bounded universal over the instance I(k).
    ForAll(String, String, Box<KernelExpr>),
    /// `∃ v ∈ k: body` — bounded existential over the instance I(k).
    Exists(String, String, Box<KernelExpr>),
}

/// Why the kernel grammar rejected an opted-in cell — always labeled
/// (sd1 discipline): the label names what failed, and for a non-member
/// atomic it names the atomic AND the closed set
/// (`kernel_grammar_closed`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    Labeled { message: String },
}

impl KernelError {
    fn labeled(message: impl Into<String>) -> Self {
        KernelError::Labeled {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for KernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelError::Labeled { message } => write!(f, "{message}"),
        }
    }
}

/// The opt-in marker for the kernel language (add-min-expr-kernel): in
/// an invariant Constraint `expr` cell, the cell text after
/// `**kernel:**`'s first fragment-position occurrence is the kernel
/// expression. Fragment position is the sd1 rule carried over: cell
/// start or immediately after an opening code-span backtick; any other
/// occurrence is a mention and never extracts.
pub const KERNEL_MARKER: &str = "**kernel:**";

/// The closed atomic set, spelled for labels — the set a non-member
/// failure names (`kernel_grammar_closed`).
pub const CLOSED_ATOMIC_SET: &str =
    "{resolves, unique, acyclic, reachable, ==, <, <=, >, >=, ∀, ∃, ∧, ¬}";

/// The v0 reference atomics — the opt-in keywords at fragment position.
const REFERENCE_ATOMICS: &[&str] = &["resolves", "unique", "acyclic", "reachable"];

/// The objects of the canonical schema, spelled — the bounding objects a
/// quantifier may name (Schema-as-data; the closed five).
const OBJECTS: &[&str] = &["Constraint", "Intent", "Property", "State", "Transition"];

// ---------------------------------------------------------------------------
// Parsing — row-typed against the canonical schema (Schema-as-data)
// ---------------------------------------------------------------------------

/// The opt-in test at fragment position — `Some((content_start,
/// content_end))` when the cell carries `**kernel:**` at cell start or
/// immediately after an opening code-span backtick (the sd1 fragment
/// position); `None` otherwise (a mention mid-span, or no marker at
/// all — prose stays prose, pure widening). The expression runs to the
/// cell's end or the span's closing backtick, exactly like the
/// executable-fragment shaping.
fn kernel_marker_span(cell: &str) -> Option<(usize, usize)> {
    let lead = cell.len() - cell.trim_start().len();
    let mut in_span = false;
    let mut prev: Option<char> = None;
    for (i, c) in cell.char_indices() {
        let prev_backtick = prev == Some('`');
        if c == '`' {
            in_span = !in_span;
        } else if cell[i..].starts_with(KERNEL_MARKER) {
            let at_cell_start = i == lead;
            let opens_span = in_span && prev_backtick;
            if at_cell_start || opens_span {
                let content_start = i + KERNEL_MARKER.len();
                let after = &cell[content_start..];
                let content_end = match after.find('`') {
                    Some(j) => content_start + j,
                    None => cell.len(),
                };
                return Some((content_start, content_end));
            }
        }
        prev = Some(c);
    }
    None
}

/// Parse a Constraints `expr` cell as a kernel expression — `Ok(None)`
/// when the cell carries no `**kernel:**` marker in fragment position
/// (prose stays prose, pure widening; citation expressions and
/// executable `**lang:**` fragments are untouched), `Ok(Some(expr))`
/// for a kernel expression, `Err(labeled)` when the cell opted in but
/// breaks the closed grammar — the label names the offending token
/// and, for a non-member atomic, the atomic and the closed set.
pub fn parse_kernel_expr(cell: &str) -> Result<Option<KernelExpr>, KernelError> {
    // A kernel marker and an executable fragment marker in one cell is
    // a double opt-in — labeled, never a silent pick of one.
    if let Ok(Some(_)) = crate::compile::fragment_of_strict(cell)
        && kernel_marker_span(cell).is_some()
    {
        return Err(KernelError::labeled(
            "cell carries both **kernel:** and an executable **lang:** marker — at most one language claim per cell",
        ));
    }
    let Some((start, end)) = kernel_marker_span(cell) else {
        return Ok(None);
    };
    let s = cell[start..end].trim().trim_matches('`').trim();
    if s.is_empty() {
        return Err(KernelError::labeled(
            "empty **kernel:** fragment — write the kernel expression after the marker",
        ));
    }
    parse_kernel_str(s)
}

/// The verbatim kernel expression an opted-in cell carries (marker
/// stripped, whitespace/backtick-shaped like the citation path's cell
/// shaping) — `None` when the cell carries no fragment-position marker.
/// The extraction path carries this verbatim into the IR.
pub fn kernel_cell_content(cell: &str) -> Option<String> {
    kernel_marker_span(cell).map(|(s, e)| cell[s..e].trim().trim_matches('`').trim().to_string())
}

/// Parse a bare kernel expression string — the grammar-level entry the
/// cell-level parser calls after stripping the `**kernel:**` marker,
/// and the seam grammar tests address directly.
pub fn parse_kernel_str(s: &str) -> Result<Option<KernelExpr>, KernelError> {
    if s.is_empty() {
        return Ok(None);
    }
    let schema = schema::canonical();
    let mut p = Parser {
        s,
        pos: 0,
        schema: &schema,
        vars: Vec::new(),
    };
    let expr = p.expr()?;
    p.skip_ws();
    if p.pos != p.s.len() {
        return Err(KernelError::labeled(format!(
            "kernel expression has trailing input `{}` — the cell must be exactly one kernel expression over {}",
            &p.s[p.pos..],
            CLOSED_ATOMIC_SET
        )));
    }
    Ok(Some(expr))
}

struct Parser<'a> {
    s: &'a str,
    pos: usize,
    schema: &'a Schema,
    /// Bound variables, scoping order — the row-typing context: a Var
    /// term must be bound, and a projection's morphism must be sourced
    /// on the variable's bounding object.
    vars: Vec<(String, String)>,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        let rest = &self.s[self.pos..];
        let n = rest.len() - rest.trim_start().len();
        self.pos += n;
    }

    fn peek(&self) -> Option<char> {
        self.s[self.pos..].chars().next()
    }

    fn eat(&mut self, tok: &str) -> bool {
        self.skip_ws();
        if self.s[self.pos..].starts_with(tok) {
            self.pos += tok.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, tok: &str, what: &str) -> Result<(), KernelError> {
        if self.eat(tok) {
            Ok(())
        } else {
            Err(KernelError::labeled(format!(
                "kernel grammar expects {what} `{tok}` at `{}` — {CLOSED_ATOMIC_SET}",
                self.rest_head()
            )))
        }
    }

    /// The next few characters, for labels — never the whole remainder.
    fn rest_head(&self) -> String {
        let rest = &self.s[self.pos..];
        let mut n = 0;
        for (i, c) in rest.char_indices() {
            if i >= 24 {
                break;
            }
            n = i + c.len_utf8();
        }
        rest[..n].to_string()
    }

    /// An unqualified identifier: `[A-Za-z_][A-Za-z0-9_-]*` — object
    /// names, morphism names, variables, unqualified id literals. A
    /// dot is never part of an unqualified identifier (the projection
    /// and file-qualification syntax owns it).
    fn ident(&mut self) -> Option<String> {
        self.skip_ws();
        let rest = &self.s[self.pos..];
        let mut end = 0;
        for (i, c) in rest.char_indices() {
            let ok = c.is_ascii_alphabetic()
                || c == '_'
                || (i > 0 && (c.is_ascii_alphanumeric() || c == '_' || c == '-'));
            if ok {
                end = i + c.len_utf8();
            } else {
                break;
            }
        }
        if end == 0 {
            return None;
        }
        let id = rest[..end].to_string();
        self.pos += end;
        Some(id)
    }

    /// A file-qualified id: dot-chained unqualified identifiers
    /// (`file.row`) — the shape reachability seeds address rows by.
    /// A malformed chain (`a..b`, a trailing dot) is not an id.
    fn qualified_ident(&mut self) -> Option<String> {
        let start = self.pos;
        let first = self.ident()?;
        let mut id = first;
        while self.s[self.pos..].starts_with('.') {
            self.pos += 1;
            match self.ident() {
                Some(part) => id = format!("{id}.{part}"),
                None => {
                    self.pos = start;
                    return None;
                }
            }
        }
        Some(id)
    }

    /// expr := And-chain of unary (∧ left-associative, matching the
    /// citation algebra's fold).
    fn expr(&mut self) -> Result<KernelExpr, KernelError> {
        let first = self.unary()?;
        let mut acc = first;
        loop {
            self.skip_ws();
            if self.s[self.pos..].starts_with('∧') {
                self.pos += '∧'.len_utf8();
                let rhs = self.unary()?;
                acc = KernelExpr::And(Box::new(acc), Box::new(rhs));
            } else {
                return Ok(acc);
            }
        }
    }

    /// unary := '¬' unary | primary
    fn unary(&mut self) -> Result<KernelExpr, KernelError> {
        self.skip_ws();
        if self.s[self.pos..].starts_with('¬') {
            self.pos += '¬'.len_utf8();
            return Ok(KernelExpr::Not(Box::new(self.unary()?)));
        }
        self.primary()
    }

    /// primary := quantifier | '(' expr ')' | atomic
    fn primary(&mut self) -> Result<KernelExpr, KernelError> {
        self.skip_ws();
        match self.peek() {
            Some('∀') => self.quantifier(true),
            Some('∃') => self.quantifier(false),
            Some('(') => {
                self.expect("(", "an opening paren")?;
                let inner = self.expr()?;
                self.expect(")", "a closing paren")?;
                Ok(inner)
            }
            _ => self.atomic(),
        }
    }

    /// quantifier := ('∀'|'∃') ident '∈' Object ':' expr — the bounding
    /// object is row-typed against the schema (an unknown object is a
    /// labeled failure); the variable scopes over the body.
    fn quantifier(&mut self, forall: bool) -> Result<KernelExpr, KernelError> {
        let tok = if forall { "∀" } else { "∃" };
        self.expect(tok, "a quantifier")?;
        let var = self.ident().ok_or_else(|| {
            KernelError::labeled(
                "kernel grammar expects a bound variable name after the quantifier",
            )
        })?;
        self.expect("∈", "the membership symbol in a quantifier")?;
        let object = self
            .ident()
            .ok_or_else(|| KernelError::labeled("kernel grammar expects an object after `∈`"))?;
        if !OBJECTS.contains(&object.as_str()) {
            return Err(KernelError::labeled(format!(
                "unknown quantifier object `{object}` — the schema's objects are {{{}}}",
                OBJECTS.join(", ")
            )));
        }
        self.expect(":", "the colon opening a quantifier body")?;
        self.vars.push((var.clone(), object.clone()));
        let body = self.expr()?;
        self.vars.pop();
        Ok(if forall {
            KernelExpr::ForAll(var, object, Box::new(body))
        } else {
            KernelExpr::Exists(var, object, Box::new(body))
        })
    }

    /// atomic := reference-atomic | comparison | non-member (labeled)
    fn atomic(&mut self) -> Result<KernelExpr, KernelError> {
        self.skip_ws();
        // A call shape `name(...)` where name is not a member atomic is
        // THE labeled non-member-atomic failure — never prose, never
        // silence (kernel_grammar_closed names the atomic AND the set).
        if let Some(w) = self.ident() {
            let rest = self.s[self.pos..].trim_start();
            if rest.starts_with('(') {
                if REFERENCE_ATOMICS.contains(&w.as_str()) {
                    self.pos -= w.len(); // re-dispatch to the atomic arms
                } else {
                    return Err(KernelError::labeled(format!(
                        "unknown atomic `{w}` — the closed atomic set is {CLOSED_ATOMIC_SET} (kernel_grammar_closed)"
                    )));
                }
            } else {
                self.pos -= w.len(); // not a call — a term; fall through
            }
        }
        self.skip_ws();
        for (kw, is_acyclic) in [("resolves", false), ("unique", false), ("acyclic", true)] {
            if self.s[self.pos..].starts_with(kw) {
                let rest = &self.s[self.pos + kw.len()..];
                if rest.trim_start().starts_with('(') {
                    self.pos += kw.len();
                    let m = self.morphism_arg(kw)?;
                    if is_acyclic && !self.endo(&m) {
                        return Err(KernelError::labeled(format!(
                            "acyclic requires an endo morphism (source object == target object) — `{m}` is not one"
                        )));
                    }
                    return Ok(KernelExpr::Atomic(match kw {
                        "resolves" => Atomic::Resolves(m),
                        "unique" => Atomic::Unique(m),
                        _ => Atomic::Acyclic(m),
                    }));
                }
            }
        }
        if self.s[self.pos..].starts_with("reachable") {
            let rest = &self.s[self.pos + "reachable".len()..];
            if rest.trim_start().starts_with('(') {
                self.pos += "reachable".len();
                self.expect("(", "the argument list of reachable")?;
                let from = self
                    .qualified_ident()
                    .ok_or_else(|| KernelError::labeled("reachable expects a from id"))?;
                self.expect(",", "the comma after reachable's from id")?;
                let to = self
                    .qualified_ident()
                    .ok_or_else(|| KernelError::labeled("reachable expects a to id"))?;
                self.expect(",", "the comma after reachable's to id")?;
                let m = self
                    .ident()
                    .ok_or_else(|| KernelError::labeled("reachable expects a morphism name"))?;
                if !self.schema.morphisms.iter().any(|row| row.name == m) {
                    return Err(KernelError::labeled(format!(
                        "unknown morphism `{m}` in reachable — the schema's morphisms are the Reference Typing rows (traces_to, derives_from, guard, supersedes, emits, satisfies, observes, uses, from, to)"
                    )));
                }
                self.expect(")", "the closing paren of reachable")?;
                return Ok(KernelExpr::Atomic(Atomic::Reachable(from, to, m)));
            }
        }
        // Comparison/equality: term op term.
        let lhs = self.term()?;
        self.skip_ws();
        let rest = &self.s[self.pos..];
        let (len, op) = if rest.starts_with("==") {
            (2, None)
        } else if rest.starts_with("<=") {
            (2, Some(CmpOp::Le))
        } else if rest.starts_with(">=") {
            (2, Some(CmpOp::Ge))
        } else if rest.starts_with('<') {
            (1, Some(CmpOp::Lt))
        } else if rest.starts_with('>') {
            (1, Some(CmpOp::Gt))
        } else {
            return Err(KernelError::labeled(format!(
                "kernel grammar expects an equality or comparison operator at `{}` — {CLOSED_ATOMIC_SET}",
                self.rest_head()
            )));
        };
        self.pos += len;
        let rhs = self.term()?;
        Ok(match op {
            None => KernelExpr::Atomic(Atomic::Eq(lhs, rhs)),
            Some(op) => KernelExpr::Atomic(Atomic::Cmp(lhs, op, rhs)),
        })
    }

    /// Is `name` an endo morphism (source object == target object — the
    /// shape `acyclic` quantifies cycles through)? Rows sharing a name
    /// are checked together.
    fn endo(&self, name: &str) -> bool {
        self.schema
            .morphisms
            .iter()
            .filter(|m| m.name == name)
            .all(|m| m.source.0 == m.target.0)
    }

    /// One morphism-name argument — row-typed: the name must be a
    /// schema morphism (Schema-as-data).
    fn morphism_arg(&mut self, atomic: &str) -> Result<String, KernelError> {
        self.expect("(", &format!("the argument list of {atomic}"))?;
        let m = self
            .ident()
            .ok_or_else(|| KernelError::labeled(format!("{atomic} expects a morphism name")))?;
        self.expect(")", &format!("the closing paren of {atomic}"))?;
        if !self.schema.morphisms.iter().any(|row| row.name == m) {
            return Err(KernelError::labeled(format!(
                "unknown morphism `{m}` in {atomic} — the schema's morphisms are the Reference Typing rows (traces_to, derives_from, guard, supersedes, emits, satisfies, observes, uses, from, to)"
            )));
        }
        Ok(m)
    }

    /// term := '|' Object '|' | Int | '⊥' | ident['.' morphism]
    fn term(&mut self) -> Result<Term, KernelError> {
        self.skip_ws();
        if self.s[self.pos..].starts_with('|') {
            self.pos += 1;
            let object = self
                .ident()
                .ok_or_else(|| KernelError::labeled("cardinality term expects an object"))?;
            self.expect("|", "the closing bar of a cardinality term")?;
            if !OBJECTS.contains(&object.as_str()) {
                return Err(KernelError::labeled(format!(
                    "unknown cardinality object `{object}` — the schema's objects are {{{}}}",
                    OBJECTS.join(", ")
                )));
            }
            return Ok(Term::Card(object));
        }
        if self.s[self.pos..].starts_with('⊥') {
            self.pos += '⊥'.len_utf8();
            return Ok(Term::Dangling);
        }
        // An integer literal — the bounded comparison space.
        self.skip_ws();
        if let Some(rest) = self.s[self.pos..].strip_prefix('-') {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if !digits.is_empty() {
                let n: i64 = format!("-{digits}").parse().map_err(|_| {
                    KernelError::labeled(format!("integer literal out of range at `-{digits}`"))
                })?;
                self.pos += 1 + digits.len();
                return Ok(Term::Int(n));
            }
        }
        {
            let rest = &self.s[self.pos..];
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if !digits.is_empty() {
                let n: i64 = digits.parse().map_err(|_| {
                    KernelError::labeled(format!("integer literal out of range at `{digits}`"))
                })?;
                self.pos += digits.len();
                return Ok(Term::Int(n));
            }
        }
        let Some(word) = self.ident() else {
            return Err(KernelError::labeled(format!(
                "kernel grammar expects a term at `{}` — {CLOSED_ATOMIC_SET}",
                self.rest_head()
            )));
        };
        // A projection `var.morphism` — row-typed: `var` must be a
        // bound variable and the morphism must be sourced on its
        // bounding object (Schema-as-data supplies the typing). A dot
        // after an UNBOUND identifier is an id literal's
        // file-qualification, not a projection — the binding decides
        // (rule of record).
        self.skip_ws();
        if self.s[self.pos..].starts_with('.') && self.vars.iter().any(|(v, _)| v == &word) {
            self.pos += 1;
            let m = self.ident().ok_or_else(|| {
                KernelError::labeled("projection expects a morphism name after `.`")
            })?;
            let bound_object = self
                .vars
                .iter()
                .rev()
                .find(|(v, _)| v == &word)
                .map(|(_, o)| o.clone());
            let Some(object) = bound_object else {
                unreachable!("the binding was just checked")
            };
            if !self
                .schema
                .morphisms
                .iter()
                .any(|row| row.name == m && row.source.0 == object)
            {
                return Err(KernelError::labeled(format!(
                    "morphism `{m}` is not sourced on {object} — the Reference Typing rows type `{word}.{m}` ill-formed here"
                )));
            }
            return Ok(Term::Proj(word, m));
        }
        // A dot after an unbound identifier: a file-qualified id
        // literal — consume the dot-chained continuation.
        if self.s[self.pos..].starts_with('.') {
            let mut id = word;
            while self.s[self.pos..].starts_with('.') {
                self.pos += 1;
                let part = self.ident().ok_or_else(|| {
                    KernelError::labeled("a file-qualified id literal continues after every dot")
                })?;
                id = format!("{id}.{part}");
            }
            return Ok(Term::Id(id));
        }
        // A bare identifier: a bound variable, else an id literal.
        if self.vars.iter().any(|(v, _)| v == &word) {
            Ok(Term::Var(word))
        } else {
            Ok(Term::Id(word))
        }
    }
}

// ---------------------------------------------------------------------------
// Evaluation — the grounding table (D1) over the acset substrate
// ---------------------------------------------------------------------------

/// The grounding machinery a v0 atomic names (design D1's table) —
/// kernel semantics never outruns proven machinery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grounding {
    /// Graph traversal — `src/graph.rs` / `acset::query` closures and
    /// cycle detection (`acyclic`, `reachable`).
    GraphTraversal,
    /// Acset traversal — instance morphism vectors and their reports
    /// (`resolves`, `unique`).
    AcsetTraversal,
    /// Bounded evaluation over the finite instances I(k) — the
    /// model_check bounded-evaluation discipline (`==`, comparisons,
    /// ∀, ∃).
    BoundedEvaluation,
}

impl std::fmt::Display for Grounding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grounding::GraphTraversal => write!(f, "graph traversal"),
            Grounding::AcsetTraversal => write!(f, "acset traversal"),
            Grounding::BoundedEvaluation => write!(f, "model_check bounded evaluation"),
        }
    }
}

/// The evaluation environment: the acset instance I over the corpus
/// (Schema-as-data supplies the 𝒦-typed instances I(k)) plus the
/// per-object membership the quantifiers range over.
pub struct KernelEnv {
    instance: Instance,
    /// I(k): the instance's node ids, grouped by schema object —
    /// Intent ids are the file ids; row ids are file-qualified.
    members: BTreeMap<String, Vec<String>>,
}

impl KernelEnv {
    /// Build the environment from a parsed corpus — the acset
    /// constructor plus the row-typed membership maps.
    pub fn from_specs(specs: &[Spec]) -> Self {
        let instance = Instance::from_specs(specs);
        let mut members: BTreeMap<String, Vec<String>> = OBJECTS
            .iter()
            .map(|o| (o.to_string(), Vec::new()))
            .collect();
        for (node, kind) in kind_index(specs) {
            let object = match kind {
                crate::graph::NodeKind::Intent => "Intent",
                crate::graph::NodeKind::Constraint(_) => "Constraint",
                crate::graph::NodeKind::Property(_) => "Property",
                crate::graph::NodeKind::State => "State",
                crate::graph::NodeKind::Transition => "Transition",
            };
            members.entry(object.to_string()).or_default().push(node);
        }
        for v in members.values_mut() {
            v.sort();
        }
        KernelEnv { instance, members }
    }

    /// I(k) for one object — the bounded quantifier domain. An unknown
    /// object yields an empty domain (parse-time row-typing makes this
    /// unreachable from accepted expressions).
    pub fn domain(&self, object: &str) -> &[String] {
        self.members.get(object).map(Vec::as_slice).unwrap_or(&[])
    }

    /// The underlying instance (acset traversal groundings read it).
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// The bound variable's scope: var name → bound row id.
    fn bind(&self, scope: &BTreeMap<String, String>, var: &str) -> Option<String> {
        scope.get(var).cloned()
    }

    /// Evaluate a term to its bounded value. A projection on a missing
    /// node or an unresolved reference is the dangling value — a value,
    /// never a crash (`dangling_is_a_value`).
    fn term_value(&self, scope: &BTreeMap<String, String>, term: &Term) -> TermVal {
        match term {
            Term::Var(v) => match self.bind(scope, v) {
                Some(id) => TermVal::Id(id),
                // Unbound vars cannot survive parse-time row-typing; an
                // unknown variable in a hand-built expression is honest
                // unknown, never a fabricated verdict.
                None => TermVal::Unknown,
            },
            Term::Id(id) => TermVal::Id(id.clone()),
            Term::Int(n) => TermVal::Int(*n),
            Term::Card(object) => TermVal::Int(self.members.get(object).map_or(0, Vec::len) as i64),
            Term::Proj(v, m) => match self.bind(scope, v) {
                Some(id) => match self.instance.index_of(&id) {
                    Some(from) => {
                        let hit = self
                            .instance
                            .morphism_values(m)
                            .into_iter()
                            .find(|(f, _)| *f == from)
                            .and_then(|(_, t)| t);
                        match hit {
                            Some(to) => TermVal::Id(self.instance.id_of(to).to_string()),
                            // No stored value from this node — dangling
                            // or absent; both are ⊥ here.
                            None => TermVal::Dangling,
                        }
                    }
                    None => TermVal::Unknown,
                },
                None => TermVal::Unknown,
            },
            Term::Dangling => TermVal::Dangling,
        }
    }

    /// Compare two term values under `==` — Kleene: a type mismatch
    /// (id vs integer) cannot be discharged and is `unknown`, never a
    /// fabricated verdict.
    fn eq(&self, a: &TermVal, b: &TermVal) -> ThreeValued {
        match (a, b) {
            (TermVal::Int(x), TermVal::Int(y)) if x == y => ThreeValued::Verified,
            (TermVal::Int(_), TermVal::Int(_)) => ThreeValued::Counterexample,
            (TermVal::Id(x), TermVal::Id(y)) if x == y => ThreeValued::Verified,
            (TermVal::Id(_), TermVal::Id(_)) => ThreeValued::Counterexample,
            (TermVal::Dangling, TermVal::Dangling) => ThreeValued::Verified,
            // A defined value never equals the dangling value.
            (TermVal::Id(_), TermVal::Dangling) | (TermVal::Dangling, TermVal::Id(_)) => {
                ThreeValued::Counterexample
            }
            _ => ThreeValued::Unknown,
        }
    }

    /// Ordering comparison — integers only in v0; an id has no order
    /// (honest unknown), and anything touching dangling or a
    /// type-mismatch is unknown.
    fn cmp(&self, op: CmpOp, a: &TermVal, b: &TermVal) -> ThreeValued {
        match (a, b) {
            (TermVal::Int(x), TermVal::Int(y)) => {
                let holds = match op {
                    CmpOp::Lt => x < y,
                    CmpOp::Le => x <= y,
                    CmpOp::Gt => x > y,
                    CmpOp::Ge => x >= y,
                };
                if holds {
                    ThreeValued::Verified
                } else {
                    ThreeValued::Counterexample
                }
            }
            _ => ThreeValued::Unknown,
        }
    }

    /// Evaluate one atomic under its grounding (D1 table). Total and
    /// terminating over any finite instance (`kernel_decidable`).
    fn atomic(&self, a: &Atomic, scope: &BTreeMap<String, String>) -> ThreeValued {
        match a {
            Atomic::Resolves(m) => {
                // Acset traversal: the morphism vector's `None` values
                // are exactly the dangling references
                // (`dangling_is_a_value`).
                if self
                    .instance
                    .morphism_values(m)
                    .iter()
                    .all(|(_, t)| t.is_some())
                {
                    ThreeValued::Verified
                } else {
                    ThreeValued::Counterexample
                }
            }
            Atomic::Unique(m) => {
                // Acset traversal: the defined targets are pairwise
                // distinct (injective on the defined values).
                let mut targets: Vec<usize> = self
                    .instance
                    .morphism_values(m)
                    .into_iter()
                    .filter_map(|(_, t)| t)
                    .collect();
                targets.sort();
                let distinct = targets.iter().collect::<std::collections::BTreeSet<_>>();
                if targets.len() == distinct.len() {
                    ThreeValued::Verified
                } else {
                    ThreeValued::Counterexample
                }
            }
            Atomic::Acyclic(m) => {
                // Graph traversal: the closure-based cycle membership
                // (`cyclic_nodes`) — empty set means acyclic.
                match query::cyclic_nodes(&self.instance, &[m.as_str()]) {
                    Ok(cyclic) if cyclic.is_empty() => ThreeValued::Verified,
                    Ok(_) => ThreeValued::Counterexample,
                    Err(_) => ThreeValued::Unknown,
                }
            }
            Atomic::Reachable(from, to, m) => {
                // Graph traversal: the forward closure from the seed.
                // An unknown seed cannot be discharged — unknown, never
                // a fabricated verdict.
                let reached =
                    query::forward_closure(&self.instance, &[from.as_str()], &[m.as_str()]);
                match reached {
                    Err(_) => ThreeValued::Unknown,
                    Ok(set) => {
                        if set.contains(to) {
                            ThreeValued::Verified
                        } else {
                            ThreeValued::Counterexample
                        }
                    }
                }
            }
            Atomic::Eq(lhs, rhs) => {
                let a = self.term_value(scope, lhs);
                let b = self.term_value(scope, rhs);
                self.eq(&a, &b)
            }
            Atomic::Cmp(lhs, op, rhs) => {
                let a = self.term_value(scope, lhs);
                let b = self.term_value(scope, rhs);
                self.cmp(*op, &a, &b)
            }
        }
    }

    /// Evaluate a kernel expression — total, terminating, exactly one
    /// status (`kernel_decidable`), Kleene-composed (dl/1 absorb:
    /// unknown never coerces to pass or to counterexample).
    pub fn evaluate(&self, expr: &KernelExpr) -> ThreeValued {
        self.eval_scope(expr, &BTreeMap::new())
    }

    fn eval_scope(&self, expr: &KernelExpr, scope: &BTreeMap<String, String>) -> ThreeValued {
        match expr {
            KernelExpr::Atomic(a) => self.atomic(a, scope),
            KernelExpr::Not(inner) => match self.eval_scope(inner, scope) {
                ThreeValued::Verified => ThreeValued::Counterexample,
                ThreeValued::Counterexample => ThreeValued::Verified,
                ThreeValued::Unknown => ThreeValued::Unknown,
            },
            KernelExpr::And(a, b) => {
                // The citation algebra's conjunction, verbatim: a
                // counterexample dominates; two verifieds verify;
                // otherwise the unknown absorbs (never pass).
                match (self.eval_scope(a, scope), self.eval_scope(b, scope)) {
                    (ThreeValued::Counterexample, _) | (_, ThreeValued::Counterexample) => {
                        ThreeValued::Counterexample
                    }
                    (ThreeValued::Verified, ThreeValued::Verified) => ThreeValued::Verified,
                    _ => ThreeValued::Unknown,
                }
            }
            KernelExpr::ForAll(var, object, body) => {
                // Bounded ∀ over I(k): empty domain is verified (vacuous
                // truth over a finite, empty set — the bounded reading);
                // a counterexample dominates; else unknown absorbs.
                let mut saw_unknown = false;
                for id in self.domain(object) {
                    let mut bound = scope.clone();
                    bound.insert(var.clone(), id.clone());
                    match self.eval_scope(body, &bound) {
                        ThreeValued::Counterexample => return ThreeValued::Counterexample,
                        ThreeValued::Unknown => saw_unknown = true,
                        ThreeValued::Verified => {}
                    }
                }
                if saw_unknown {
                    ThreeValued::Unknown
                } else {
                    ThreeValued::Verified
                }
            }
            KernelExpr::Exists(var, object, body) => {
                // Bounded ∃ over I(k): a verified witness wins; else a
                // counterexample on every element refutes; else unknown.
                let mut saw_unknown = false;
                for id in self.domain(object) {
                    let mut bound = scope.clone();
                    bound.insert(var.clone(), id.clone());
                    match self.eval_scope(body, &bound) {
                        ThreeValued::Verified => return ThreeValued::Verified,
                        ThreeValued::Unknown => saw_unknown = true,
                        ThreeValued::Counterexample => {}
                    }
                }
                if saw_unknown {
                    ThreeValued::Unknown
                } else {
                    ThreeValued::Counterexample
                }
            }
        }
    }
}

/// A term's bounded value during evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TermVal {
    Id(String),
    Int(i64),
    Dangling,
    /// The value could not be determined — honest unknown, never a
    /// fabricated verdict.
    Unknown,
}

// ---------------------------------------------------------------------------
// The command-path corpus pass (§3.7, design D9) — every opted-in claim
// published exactly once, evaluated over the complete invocation corpus
// ---------------------------------------------------------------------------

/// Which backend the invocation selected. The kernel evaluation itself
/// is backend-independent (acset traversal over the corpus instance),
/// but an incapable backend must not imply it supplied the verdict:
/// design D9 — TLC either supplies same-scope kernel evaluation or
/// returns a labeled unsupported result; it cannot omit the claims and
/// imply success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpusBackend {
    /// The native backends — kernel claims evaluate over the corpus.
    Native,
    /// The TLC reference engine — kernel claims come back labeled
    /// unsupported (`kernel_backend_unsupported`).
    Tlc,
}

impl CorpusBackend {
    /// The shared backend selection (§3.8 TIDY — specodelic-k3h): one
    /// constructor both command paths use — the corpus backend is TLC
    /// exactly when the invocation carries a TLC configuration.
    pub fn from_tlc_presence(has_tlc: bool) -> Self {
        if has_tlc { Self::Tlc } else { Self::Native }
    }
}

/// One kernel claim's command-path status: the Constraints-table row id
/// (within the file that declares it), its three-valued status, and the
/// labeled reason when (and only when) the status is unknown for a
/// namable cause. Mirrors the citation path's `ResolvedStatus` shape —
/// the same command-output and persisted-report duality, merged through
/// the one shared identity helper (`merge_corpus_statuses`, §3.8 TIDY).
#[derive(Debug, Clone, PartialEq)]
pub struct CorpusClaimStatus {
    pub id: String,
    pub status: ThreeValued,
    pub reason: Option<String>,
}

impl CorpusClaimStatus {
    /// The report shape — status only, no reason (RunReport keeps its
    /// deployed schema; reasons surface in command output).
    pub fn invariant_status(&self) -> crate::model_check::InvariantStatus {
        crate::model_check::InvariantStatus {
            id: self.id.clone(),
            status: self.status,
        }
    }

    /// The command-output shape: id, status, and the reason when present.
    pub fn output_json(&self) -> serde_json::Value {
        let mut entry = serde_json::json!({
            "id": self.id,
            "status": self.status,
        });
        if let Some(reason) = &self.reason {
            entry["reason"] = serde_json::json!(reason);
        }
        entry
    }
}

/// The labeled kernel-grammar pre-check for the read paths that parse a
/// spec after the fact (specodelic-m6k): every invariant-kind Constraint
/// whose expr cell opts in via `**kernel:**` but breaks the closed
/// grammar yields a labeled failure naming the row, the offending
/// token/atomic and the closed set — the same labeled validation compile
/// applies before emitting an artifact (kernel_grammar_closed, "never
/// prose, never silence"). Rationale: a grammar-broken cell never
/// reaches `extract_model_ir`'s `guard_kernel` map, so a reader that
/// classifies claims from the IR alone silently demotes the claim to
/// unchecked — model-check then reports a clean aggregate over a spec
/// compile itself rejects. Returns `(row id, labeled message)` pairs.
pub fn grammar_failures(spec: &Spec) -> Vec<(String, String)> {
    spec.constraints
        .iter()
        .filter(|c| c.cells.get("kind").map(String::as_str) == Some("invariant"))
        .filter_map(|c| {
            let expr = c.cells.get("expr").map(String::as_str).unwrap_or("");
            match parse_kernel_expr(expr) {
                Err(e) => Some((
                    c.id.clone(),
                    format!(
                        "row `{}`: {e} (add-min-expr-kernel kernel_grammar_closed)",
                        c.id
                    ),
                )),
                Ok(_) => None,
            }
        })
        .collect()
}

/// Evaluate every opted-in kernel claim of every spec over the complete
/// explicit invocation corpus (§3.7, design D9): one `KernelEnv` over
/// ALL specs in the invocation — never per-file — so cardinality and
/// quantifier claims see the whole corpus. Claims stay attached to the
/// file that declares them and appear exactly once. The backend tag
/// decides the incapable-backend leg: TLC claims come back unknown with
/// the labeled unsupported reason. No filesystem discovery — only the
/// explicit inputs contribute.
pub fn evaluate_corpus_claims(
    specs: &[Spec],
    backend: CorpusBackend,
) -> Vec<Vec<CorpusClaimStatus>> {
    match backend {
        CorpusBackend::Tlc => specs
            .iter()
            .map(|spec| {
                compile::extract_model_ir(spec)
                    .guard_kernel
                    .keys()
                    .map(|id| CorpusClaimStatus {
                        id: id.clone(),
                        status: ThreeValued::Unknown,
                        reason: Some(
                            "the tlc backend cannot evaluate kernel claims — labeled unsupported result, never omitted or implied pass (kernel_backend_unsupported)"
                                .to_string(),
                        ),
                    })
                    .collect()
            })
            .collect(),
        CorpusBackend::Native => {
            let env = KernelEnv::from_specs(specs);
            specs
                .iter()
                .map(|spec| {
                    compile::extract_model_ir(spec)
                        .guard_kernel
                        .iter()
                        .map(|(id, cell)| match parse_kernel_str(cell) {
                            Ok(Some(expr)) => CorpusClaimStatus {
                                id: id.clone(),
                                status: env.evaluate(&expr),
                                reason: None,
                            },
                            // Proven parseable at extraction time; a failed
                            // re-parse is honest unknown, never a silent drop.
                            _ => CorpusClaimStatus {
                                id: id.clone(),
                                status: ThreeValued::Unknown,
                                reason: Some(
                                    "kernel cell no longer parses — honest unknown, never a silent drop"
                                        .to_string(),
                                ),
                            },
                        })
                        .collect()
                })
                .collect()
        }
    }
}

/// The shared command-path corpus-status merge (§3.8 TIDY —
/// specodelic-k3h): the ONE place the corpus identity rules live —
/// which ids appear, which status each carries, which reason labels an
/// unknown, and the exact command-output entry shape. Both command
/// paths call this (the native CLI's `commands/model_check.rs` and
/// orchestrate's `model_check` stage); the per-call-site merge blocks
/// they duplicated are gone, so CLI output, orchestrate stage detail
/// and the persisted reports cannot drift apart.
///
/// Appends the kernel claims to the report's persisted statuses (status
/// only — the `RunReport` schema keeps its deployed shape) and returns
/// the command-output entries: every merged invariant with its status,
/// the unknown reason attached when (and only when) present. A claim
/// reason wins over a citation reason for the same id — claims are
/// inserted after the resolved citations, matching the historical
/// per-call-site chain order.
pub fn merge_corpus_statuses(
    report: &mut crate::model_check::RunReport,
    resolved: &[crate::citation_corpus::ResolvedStatus],
    claims: &[CorpusClaimStatus],
) -> Vec<serde_json::Value> {
    for k in claims {
        report.invariant_statuses.push(k.invariant_status());
    }
    let mut reasons: BTreeMap<&str, &String> = BTreeMap::new();
    for r in resolved {
        if let Some(reason) = &r.reason {
            reasons.insert(r.id.as_str(), reason);
        }
    }
    for k in claims {
        if let Some(reason) = &k.reason {
            reasons.insert(k.id.as_str(), reason);
        }
    }
    report
        .invariant_statuses
        .iter()
        .map(|s| {
            let mut entry = serde_json::json!({ "id": s.id, "status": s.status });
            if let Some(reason) = reasons.get(s.id.as_str()) {
                entry["reason"] = serde_json::json!(reason);
            }
            entry
        })
        .collect()
}

/// The shared claim-gate aggregate (define-verification-claim-gates
/// design D1/D2/D4 — the ONE implementation, called by both command
/// paths right after the corpus-status merge): the merged required-claim
/// statuses — executable fragments, resolved citations, kernel claims —
/// govern the aggregate verdict. Prose-only invariant rows never appear
/// in `invariant_statuses`, so they stay explicitly unchecked (D1);
/// merely having a deriving Property never implies verification.
///
/// D2 rules, in order:
///
/// 1. the backend's own counterexample verdict is kept with its trace
///    and violated id (claim evidence, never rewritten);
/// 2. a refuted required claim yields `counterexample_found`, naming
///    the claim id with NO invented state trace;
/// 3. an exhausted budget stays `timed_out` — a truncated exploration
///    proves nothing about the claims;
/// 4. any unknown or unsupported required claim prevents clean and
///    reports `exploration_only` (the labeled reasons surface in
///    command output via `merge_corpus_statuses`);
/// 5. an empty required set also reports `exploration_only`;
/// 6. a nonempty complete all-verified set over a completed bounded
///    exploration yields `no_counterexample` — the BREAKING acceptance
///    flip: discharging every opted-in claim is the clean verdict.
///
/// `verify` consumes the persisted outcome, so both command paths and
/// the verify gate cannot drift apart (D4).
pub fn aggregate_required_claims(report: &mut crate::model_check::RunReport) {
    use crate::model_check::Outcome;
    // Rule 1: the backend's own counterexample verdict stands.
    if report.outcome == Outcome::CounterexampleFound {
        return;
    }
    // Rule 2: a refuted required claim governs — claim evidence, no
    // invented state trace.
    if let Some(refuted) = report
        .invariant_statuses
        .iter()
        .find(|s| s.status == ThreeValued::Counterexample)
    {
        report.outcome = Outcome::CounterexampleFound;
        report.violated_invariant_id = Some(refuted.id.clone());
        report.trace = None;
        return;
    }
    // Rule 3: an exhausted budget outranks the remaining legs.
    if report.outcome == Outcome::TimedOut {
        return;
    }
    // Rule 4: any unknown/unsupported required claim prevents clean.
    if report
        .invariant_statuses
        .iter()
        .any(|s| s.status == ThreeValued::Unknown)
    {
        report.outcome = Outcome::ExplorationOnly;
        return;
    }
    // Rule 5: an empty required set is explicitly unchecked.
    if report.invariant_statuses.is_empty() {
        report.outcome = Outcome::ExplorationOnly;
        return;
    }
    // Rule 6: nonempty, complete, all-verified over a completed bounded
    // exploration — the clean verdict.
    report.outcome = Outcome::NoCounterexample;
}

// ---------------------------------------------------------------------------
// The predicate registry seam (3.5 TIDY) — widened in a later phase
// ---------------------------------------------------------------------------

/// One registered predicate: its grounding entry (D1 — no registration
/// without one) and the decidability certificate (D8's gate).
#[derive(Debug, Clone)]
pub struct Registered {
    pub name: String,
    pub grounding: Grounding,
}

/// Why a registration was refused — the D8 gate speaking, labeled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterError {
    /// The predicate declared no grounding entry (D1: an atomic without
    /// a grounding entry cannot ship).
    NoGrounding(String),
    /// The predicate failed the decidability gate over finite instances
    /// (D8: an undecidable predicate cannot register — it stays
    /// pack-side, checked by contract-TOML runners).
    Undecidable(String),
    /// The predicate is already registered.
    Duplicate(String),
}

impl std::fmt::Display for RegisterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterError::NoGrounding(name) => {
                write!(
                    f,
                    "`{name}` declares no grounding entry — an atomic without a grounding entry cannot ship (kernel_grounded_in_machinery)"
                )
            }
            RegisterError::Undecidable(name) => {
                write!(
                    f,
                    "`{name}` failed the decidability gate over finite instances (kernel_decidable) — it stays pack-side, checked by contract-TOML runners"
                )
            }
            RegisterError::Duplicate(name) => {
                write!(f, "`{name}` is already registered")
            }
        }
    }
}

/// The predicate registry — the seam later pack registration rides (D8:
/// the widening law with a decidability gate). The v0 atomics are the
/// base set, each with its grounding entry. NOTHING outside the v0 set
/// registers in this phase: registration code exists; no pack predicate
/// registers (the D8 gate refuses undecidable predicates pack-side).
pub struct PredicateRegistry {
    entries: BTreeMap<String, Registered>,
}

impl Default for PredicateRegistry {
    fn default() -> Self {
        Self::v0()
    }
}

impl PredicateRegistry {
    /// The v0 registry — the D1 grounding table as data. Every v0
    /// atomic names its proven machinery.
    pub fn v0() -> Self {
        let entries = [
            ("resolves", Grounding::AcsetTraversal),
            ("unique", Grounding::AcsetTraversal),
            ("acyclic", Grounding::GraphTraversal),
            ("reachable", Grounding::GraphTraversal),
            ("==", Grounding::BoundedEvaluation),
            ("<", Grounding::BoundedEvaluation),
            ("<=", Grounding::BoundedEvaluation),
            (">", Grounding::BoundedEvaluation),
            (">=", Grounding::BoundedEvaluation),
            ("∀", Grounding::BoundedEvaluation),
            ("∃", Grounding::BoundedEvaluation),
        ]
        .into_iter()
        .map(|(name, grounding)| {
            (
                name.to_string(),
                Registered {
                    name: name.to_string(),
                    grounding,
                },
            )
        })
        .collect();
        PredicateRegistry { entries }
    }

    /// The grounding entry for an atomic — `None` for an unregistered
    /// one (which cannot be part of the closed set).
    pub fn grounding(&self, atomic: &str) -> Option<Grounding> {
        self.entries.get(atomic).map(|r| r.grounding)
    }

    /// The registered predicate names, sorted.
    pub fn names(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    /// D8's gate: a new atomic registers only with a grounding entry
    /// AND a passing decidability certificate over finite instances.
    /// Registration widens the registry's grounding table — the
    /// grammar-level widening rides the widening law in a later phase
    /// (this seam stores the entry; it does not edit the closed parse
    /// set, so no pack predicate can smuggle itself into this
    /// Revision's grammar).
    pub fn try_register(
        &mut self,
        name: &str,
        grounding: Option<Grounding>,
        decidable_over_finite_instances: bool,
    ) -> Result<(), RegisterError> {
        if self.entries.contains_key(name) {
            return Err(RegisterError::Duplicate(name.to_string()));
        }
        let Some(grounding) = grounding else {
            return Err(RegisterError::NoGrounding(name.to_string()));
        };
        if !decidable_over_finite_instances {
            return Err(RegisterError::Undecidable(name.to_string()));
        }
        self.entries.insert(
            name.to_string(),
            Registered {
                name: name.to_string(),
                grounding,
            },
        );
        Ok(())
    }
}
