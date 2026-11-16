#!/usr/bin/env python3
"""Python reference evaluator for the closed kernel vocabulary.

Purpose: provide an implementation independent of specodelic's Rust kernel
evaluator, so the shared agreement corpus (tests/common/kernel_fixtures.rs)
can be evaluated twice and the three-valued statuses compared. This is an
agreement-test backend only: never a production `--backend` selector, never
a user-facing Python emitter, never a Rust-to-Python predicate translator.

Responsibilities: read one JSON request from stdin, build the finite acset
snapshot from the verbatim markdown corpus, evaluate the marker-stripped
kernel expression over it, and print exactly one JSON response on stdout.
The closed v0 vocabulary is the kernel grammar: the atomic predicates
`acyclic`, `reachable`, `resolves` and `unique`; the comparisons `==`, `<`,
`<=`, `>` and `>=` over bounded terms; bounded `∀`/`∃` over a named object's
instance domain; conjunction `∧`; and negation `¬` — all evaluated under
Kleene three-valued logic. Parse and evaluation failures exit nonzero with
a labeled diagnostic on stderr; a fabricated status is never acceptable.

Wire contract (pinned by tests/kernel_agreement_py.rs):

  stdin:  {"specs": ["<verbatim markdown spec file>", ...],
           "expr": "<kernel cell text, marker-stripped>"}

  stdout: {"status": "verified" | "counterexample" | "unknown"}

The corpus arrives as raw file sources, not as Rust serializations: this
adapter parses frontmatter, the Constraints/Properties/Transitions tables
and the kernel expression itself, builds the acset instance (nodes, typed
morphism values, dangling references), and evaluates independently.

Rationale: the Rust kernel is the production evaluator; a second, independent
implementation over the same frozen fixtures catches semantic drift that a
single implementation's unit tests cannot (a shared wrong reading is only
visible when the two implementations must agree on expected statuses).

Semantics mirror src/kernel.rs and the acset substrate: `resolves(m)` is
vacuously verified over an empty population and refuted by any dangling
value; `unique(m)` compares the defined targets; `acyclic(m)` asks whether
any defined edge's source is reachable from its target through m;
`reachable(a, b, m)` is unknown for an interned-missing seed or an
undeclared target and otherwise refuted when b is outside the closure;
bounded ∀ is vacuously verified on an empty domain, bounded ∃ is refuted
on one; `unknown` absorbs under ∧ and ¬ per Kleene. Id literals resolve
only against declared corpus ids (file ids and file-qualified row ids).
"""

import json
import sys
from dataclasses import dataclass
from typing import Dict, Iterable, List, Optional, Sequence, Set, Tuple

VERIFIED = "verified"
COUNTEREXAMPLE = "counterexample"
UNKNOWN = "unknown"

EXIT_FAILURE = 1
EXIT_REQUEST = 2

OBJECTS = ("Constraint", "Intent", "Property", "State", "Transition")

REFERENCE_ATOMICS = frozenset(("resolves", "unique", "acyclic", "reachable"))

UNCHECKED = "unchecked"
APPEARS_ON = "appears_on"
SAME_KIND = "same_kind"


class AdapterError(Exception):
    """A labeled adapter failure; `label` prefixes the stderr diagnostic."""

    label = "error"


class RequestError(AdapterError):
    label = "request"


class SpecError(AdapterError):
    label = "spec parse"


class KernelError(AdapterError):
    label = "kernel parse"


class EvaluationError(AdapterError):
    label = "evaluation"


# ---------------------------------------------------------------------------
# The canonical schema (the Reference Typing rows the instance build reads)
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class MorphismRow:
    """One Reference Typing row: a typed reference field's target object."""

    column: str
    name: str
    source: str
    target: str
    source_rule: str
    target_kind: Optional[str] = None
    source_kind: Optional[str] = None


# Canonical order: sorted by (source object, morphism name), matching
# src/acset/schema.rs's canonical schema. The order decides which row a
# resolved edge is stored under when a column splits across target kinds.
SCHEMA: Tuple[MorphismRow, ...] = (
    MorphismRow(
        "observes", "observes", "Constraint", "Constraint", UNCHECKED, target_kind="effect"
    ),
    MorphismRow(
        "satisfies", "satisfies", "Constraint", "Constraint", UNCHECKED,
        target_kind="extension_point",
    ),
    MorphismRow("supersedes", "supersedes", "Constraint", "Constraint", SAME_KIND),
    MorphismRow("traces_to", "traces_to", "Constraint", "Intent", UNCHECKED),
    MorphismRow("uses", "uses", "Constraint", "Intent", UNCHECKED, target_kind="profile"),
    MorphismRow("derives_from", "derives_from", "Property", "Constraint", APPEARS_ON),
    MorphismRow(
        "derives_from",
        "derives_from_law",
        "Property",
        "Property",
        APPEARS_ON,
        source_kind="law",
    ),
    MorphismRow("supersedes", "supersedes", "Property", "Property", SAME_KIND),
    MorphismRow("emits", "emits", "State", "Constraint", UNCHECKED, target_kind="effect"),
    MorphismRow("from", "from", "Transition", "State", UNCHECKED),
    MorphismRow("guard", "guard", "Transition", "Constraint", UNCHECKED, target_kind="invariant"),
    MorphismRow("guard_state", "guard_state", "Transition", "State", UNCHECKED),
    MorphismRow("to", "to", "Transition", "State", UNCHECKED),
)

MORPHISM_NAMES = frozenset(row.name for row in SCHEMA)

# The graph columns that are typed reference fields (schema columns minus
# from/to/uses: the transitions walk owns from/to, and `uses` is an outbound
# leaf the graph deliberately never records).
TYPED_REFERENCE_COLUMNS = frozenset(row.column for row in SCHEMA) - frozenset(
    ("from", "to", "uses")
)

MORPH_BY_COLUMN: Dict[str, int] = {}
for _index, _row in enumerate(SCHEMA):
    MORPH_BY_COLUMN.setdefault(_row.column, _index)
ROWS_BY_COLUMN: Dict[str, List["MorphismRow"]] = {}
for _row in SCHEMA:
    ROWS_BY_COLUMN.setdefault(_row.column, []).append(_row)
APPEARS_BY_COLUMN: Dict[str, List["MorphismRow"]] = {
    column: [row for row in rows if row.source_rule == APPEARS_ON]
    for column, rows in ROWS_BY_COLUMN.items()
}
FROM_MORPH_INDEX = next(i for i, row in enumerate(SCHEMA) if row.name == "from")
TO_MORPH_INDEX = next(i for i, row in enumerate(SCHEMA) if row.name == "to")

TABLE_FIELDS = frozenset(("constraints", "properties", "states", "transitions"))


class _Forbidden:
    """Sentinel: the Reference Typing rows forbid a resolved edge."""


FORBIDDEN = _Forbidden()


@dataclass(frozen=True)
class Endpoint:
    """A node's schema object plus its own `kind` cell (empty when none)."""

    object: str
    kind: str


def _canonical_id(value: str) -> str:
    """Strip a display qualifier (`file.row (member)`) to the node id."""
    if value.endswith(")") and " (" in value:
        return value.rsplit(" (", 1)[0]
    return value


def _classify(
    column: str, source: Optional[Endpoint], target: Optional[Endpoint]
) -> object:
    """Decide the morphism row for one resolved edge (Rust schema::classify)."""
    rows = ROWS_BY_COLUMN.get(column, [])
    if target is None or not rows:
        return None
    if _source_rejected(column, source):
        return FORBIDDEN
    for row in rows:
        if _row_accepts(row, source, target):
            return row
    return FORBIDDEN


def _source_rejected(column: str, source: Optional[Endpoint]) -> bool:
    """The AppearsOn source-side rule fires before the target is considered."""
    appears = APPEARS_BY_COLUMN.get(column, [])
    if not appears:
        return False
    if source is None:
        return True
    return not any(source.object == row.source for row in appears)


def _row_accepts(
    row: MorphismRow, source: Optional[Endpoint], target: Endpoint
) -> bool:
    if target.object != row.target:
        return False
    if not _target_kind_accepts(row, target):
        return False
    if not _source_kind_accepts(row, source):
        return False
    return _source_rule_accepts(row, source, target)


def _target_kind_accepts(row: MorphismRow, target: Endpoint) -> bool:
    return row.target_kind is None or target.kind == row.target_kind


def _source_kind_accepts(row: MorphismRow, source: Optional[Endpoint]) -> bool:
    if row.source_kind is None:
        return True
    return source is not None and source.kind == row.source_kind


def _source_rule_accepts(
    row: MorphismRow, source: Optional[Endpoint], target: Endpoint
) -> bool:
    if row.source_rule == UNCHECKED:
        return True
    if row.source_rule == APPEARS_ON:
        return source is not None and source.object == row.source
    # SAME_KIND: an untyped source is not policed; a typed source must be
    # the same object as the target.
    return source is None or source.object == target.object


# ---------------------------------------------------------------------------
# Markdown spec parsing (frontmatter + structured tables/bullets)
# ---------------------------------------------------------------------------


@dataclass
class TableRow:
    id: str
    kind: str
    cells: Dict[str, str]


@dataclass
class TransitionRow:
    id: str
    from_cell: str
    to_cell: str
    guard: str


@dataclass
class Link:
    target: str
    field: str
    column: str
    source: str


@dataclass
class ParsedSpec:
    file_id: str
    intent_kind: str
    statement: str
    constraints: List[TableRow]
    properties: List[TableRow]
    states: List[TableRow]
    transitions: List[TransitionRow]
    links: List[Link]

    def defined_ids(self) -> List[str]:
        ids = [self.file_id]
        for row in self.constraints:
            ids.append(row.id)
        for row in self.properties:
            ids.append(row.id)
        for row in self.states:
            ids.append(row.id)
        for row in self.transitions:
            ids.append(row.id)
        return ids


def _yaml_scalar(raw: str) -> str:
    raw = raw.strip()
    if len(raw) >= 2 and raw[0] == '"' and raw[-1] == '"':
        try:
            return json.loads(raw)
        except ValueError:
            return raw[1:-1]
    if len(raw) >= 2 and raw[0] == "'" and raw[-1] == "'":
        return raw[1:-1].replace("''", "'")
    return raw


def _parse_frontmatter(text: str) -> Tuple[Dict[str, str], int]:
    lines = text.splitlines()
    index = 0
    while index < len(lines) and lines[index].strip() == "":
        index += 1
    if index >= len(lines) or lines[index].strip() != "---":
        raise SpecError("no YAML frontmatter found — a spec starts with a `---` block")
    index += 1
    fields: Dict[str, str] = {}
    while index < len(lines) and lines[index].strip() != "---":
        stripped = lines[index].strip()
        if stripped and not stripped.startswith("#"):
            key, sep, value = stripped.partition(":")
            if not sep:
                raise SpecError(
                    "frontmatter line {} is not `key: value`: {!r}".format(
                        index + 1, stripped
                    )
                )
            fields[key.strip()] = _yaml_scalar(value)
        index += 1
    if index >= len(lines):
        raise SpecError("unclosed frontmatter fence `---`")
    for required in ("id", "kind", "statement"):
        if required not in fields:
            raise SpecError("frontmatter is missing the `{}` field".format(required))
    return fields, index + 1


def _is_separator(line: str) -> bool:
    stripped = line
    for char in "|-: ":
        stripped = stripped.replace(char, "")
    return stripped == "" and "-" in line


def _parse_row(line: str) -> List[str]:
    text = line.strip()
    if not text.startswith("|"):
        raise SpecError("table row does not start with `|`: {!r}".format(line))
    text = text[1:]
    if text.endswith("|"):
        text = text[:-1]
    cells: List[str] = []
    current: List[str] = []
    in_backtick = False
    for char in text:
        if char == "`":
            in_backtick = not in_backtick
            current.append(char)
        elif char == "|" and not in_backtick:
            cells.append("".join(current).strip())
            current = []
        else:
            current.append(char)
    cells.append("".join(current).strip())
    return cells


def _collect_link_targets(cell: str) -> Iterable[str]:
    rest = cell
    while True:
        start = rest.find("[[")
        if start < 0:
            return
        after = rest[start + 2 :]
        end = after.find("]]")
        if end < 0:
            return
        target = after[:end].strip()
        if target:
            yield target
        rest = after[end + 2 :]


def _parse_state_bullet(bullet: str) -> Tuple[str, Optional[str]]:
    text = bullet.strip().lstrip("`").strip()
    positions = [pos for pos in (text.find("`"), text.find(" "), text.find("(")) if pos >= 0]
    if not positions:
        return text, None
    split = min(positions)
    head = text[:split]
    rest = text[split:]
    if rest.startswith("`"):
        rest = rest[1:]
    head = head.strip().strip("`")
    tail = rest.strip().lstrip("(—-:").strip()
    if tail.startswith("emits:"):
        value = tail[len("emits:") :]
    elif tail.startswith("emits"):
        value = tail[len("emits") :]
    else:
        value = tail
    value = value.strip().rstrip(")").strip().strip("`").strip()
    return head, value or None


def _push_table_row(
    spec: ParsedSpec, table: str, headers: List[str], raw_cells: List[str]
) -> None:
    cells: Dict[str, str] = {}
    for index, header in enumerate(headers):
        if index < len(raw_cells):
            cells[header] = raw_cells[index]
    row_id = cells.get("id", "")
    if not row_id:
        raise SpecError("a {} row has no `id` cell".format(table))
    field = table
    for header in sorted(cells):
        if header in ("id", "kind"):
            continue
        for target in _collect_link_targets(cells[header]):
            spec.links.append(Link(target, field, header, row_id))
    if table == "transitions":
        guard = cells.get("guard", "")
        spec.transitions.append(
            TransitionRow(
                row_id,
                cells.get("from", ""),
                cells.get("to", ""),
                guard if guard.strip() else "",
            )
        )
    else:
        row = TableRow(row_id, cells.get("kind", ""), cells)
        if table == "constraints":
            spec.constraints.append(row)
        else:
            spec.properties.append(row)


def _push_state(spec: ParsedSpec, line: str) -> None:
    row_id, emits = _parse_state_bullet(line[2:])
    cells: Dict[str, str] = {"id": row_id}
    if emits is not None:
        cells["emits"] = emits
        for target in _collect_link_targets(emits):
            spec.links.append(Link(target, "states", "emits", row_id))
    spec.states.append(TableRow(row_id, "", cells))


H2_TABLES = {"Constraints": "constraints", "Properties": "properties", "Model": "model"}


def _heading(line: str) -> Optional[Tuple[int, str]]:
    if line.startswith("### "):
        return 3, line[4:].strip()
    if line.startswith("## "):
        return 2, line[3:].strip()
    return None


def _advance_section(
    table: str, in_states: bool, level: int, heading: str
) -> Tuple[str, bool]:
    if level == 2:
        return H2_TABLES.get(heading, "none"), False
    in_states = heading == "States"
    if table == "model" and heading == "Transitions":
        return "transitions", in_states
    if heading == "States" and table in ("model", "transitions"):
        return "model", in_states
    return "none", in_states


def _consume_table_line(
    spec: ParsedSpec, table: str, line: str, headers: List[str]
) -> List[str]:
    if not line.startswith("|") or _is_separator(line):
        return headers
    cells = _parse_row(line)
    if not headers:
        return cells
    _push_table_row(spec, table, headers, cells)
    return headers


def parse_spec(text: str, index: int) -> ParsedSpec:
    """Parse one markdown spec file into the shape the instance build needs."""
    try:
        fields, body_start = _parse_frontmatter(text)
    except SpecError as error:
        raise SpecError("file {}: {}".format(index, error))
    spec = ParsedSpec(
        fields["id"],
        fields["kind"],
        fields["statement"],
        [],
        [],
        [],
        [],
        [],
    )
    lines = text.splitlines()
    table = "none"
    in_states = False
    headers: List[str] = []
    for line in lines[body_start:]:
        stripped = line.strip()
        heading = _heading(stripped)
        if heading is not None:
            table, in_states = _advance_section(table, in_states, *heading)
            headers = []
            continue
        if table in ("constraints", "properties", "transitions"):
            headers = _consume_table_line(spec, table, stripped, headers)
            continue
        if in_states and stripped.startswith("- "):
            _push_state(spec, stripped)
    return spec


# ---------------------------------------------------------------------------
# The acset instance and its typed morphism values
# ---------------------------------------------------------------------------


def _resolve(
    file_rows: Dict[str, List[str]], source_file: str, target: str
) -> Optional[str]:
    """Resolve a link target relative to its source file (graph::resolve)."""
    if target in ("model.state", "model.transition"):
        return "{}.{}".format(source_file, target)
    if target in file_rows:
        return target
    if "." not in target and target in file_rows.get(source_file, []):
        return "{}.{}".format(source_file, target)
    for index in reversed([i for i, char in enumerate(target) if char == "."]):
        file_id, rest = target[:index], target[index + 1 :]
        rows = file_rows.get(file_id)
        if rows is None:
            continue
        if rest in rows or rest in ("model.state", "model.transition"):
            return target
        if "." in rest:
            row_id, member = rest.split(".", 1)
            if row_id in rows:
                return "{}.{} ({})".format(file_id, row_id, member)
    return None


def _is_metasyntactic(target: str, file_rows: Dict[str, List[str]]) -> bool:
    """Format-doc example links are never graph structure."""
    if target in ("...", "\u2026"):
        return True
    return "." not in target and target not in file_rows


def _negate(status: str) -> str:
    if status == VERIFIED:
        return COUNTEREXAMPLE
    if status == COUNTEREXAMPLE:
        return VERIFIED
    return UNKNOWN


class Env:
    """The finite acset instance plus per-object quantifier membership."""

    def __init__(self, specs: Sequence[ParsedSpec]) -> None:
        self.specs = specs
        self.nodes: Set[str] = set()
        self.file_rows: Dict[str, List[str]] = {}
        self.kinds: Dict[str, Endpoint] = {}
        for spec in specs:
            file_id = spec.file_id
            self.nodes.add(file_id)
            for row in spec.constraints:
                self.nodes.add("{}.{}".format(file_id, row.id))
            for row in spec.properties:
                self.nodes.add("{}.{}".format(file_id, row.id))
            for row in spec.states:
                self.nodes.add("{}.{}".format(file_id, row.id))
            for row in spec.transitions:
                self.nodes.add("{}.{}".format(file_id, row.id))
            self.file_rows.setdefault(file_id, []).extend(spec.defined_ids())
        for spec in specs:
            file_id = spec.file_id
            self.kinds.setdefault(file_id, Endpoint("Intent", ""))
            for row in spec.constraints:
                self.kinds.setdefault(
                    "{}.{}".format(file_id, row.id), Endpoint("Constraint", row.kind)
                )
            for row in spec.properties:
                self.kinds.setdefault(
                    "{}.{}".format(file_id, row.id), Endpoint("Property", row.kind)
                )
            for row in spec.states:
                self.kinds.setdefault(
                    "{}.{}".format(file_id, row.id), Endpoint("State", "")
                )
            for row in spec.transitions:
                self.kinds.setdefault(
                    "{}.{}".format(file_id, row.id), Endpoint("Transition", "")
                )
        # Model anchors are legal typed targets; add the ones a link
        # resolves to so the node set carries every addressable node.
        for spec in specs:
            for link in spec.links:
                resolved = _resolve(self.file_rows, spec.file_id, link.target)
                if resolved is not None and resolved.endswith(
                    (".model.state", ".model.transition")
                ):
                    self.nodes.add(resolved)
        self.members: Dict[str, List[str]] = {obj: [] for obj in OBJECTS}
        for node_id in sorted(self.kinds):
            self.members[self.kinds[node_id].object].append(node_id)
        self.declared = set(self.kinds)
        self.vectors: List[List[Tuple[str, Optional[str]]]] = [
            [] for _ in SCHEMA
        ]
        self._walk()

    def _interned(self, node: str) -> str:
        if node not in self.nodes:
            raise EvaluationError("uninterned node id {!r}".format(node))
        return node

    def _walk(self) -> None:
        for spec in self.specs:
            self._walk_transitions(spec)
            for link in spec.links:
                self._walk_link(spec, link)

    def _walk_transitions(self, spec: ParsedSpec) -> None:
        for transition in spec.transitions:
            self._store_transition(
                spec, transition.id, transition.from_cell, FROM_MORPH_INDEX
            )
            self._store_transition(
                spec, transition.id, transition.to_cell, TO_MORPH_INDEX
            )

    def _store_transition(
        self, spec: ParsedSpec, row_id: str, cell: str, morph: int
    ) -> None:
        state = cell.strip().strip("`").strip()
        if not state:
            return
        anchor = self._interned("{}.{}".format(spec.file_id, row_id))
        target = None
        if any(row.id == state for row in spec.states):
            target = self._interned("{}.{}".format(spec.file_id, state))
        self.vectors[morph].append((anchor, target))

    def _walk_link(self, spec: ParsedSpec, link: Link) -> None:
        if link.column not in TYPED_REFERENCE_COLUMNS:
            return
        if _is_metasyntactic(link.target, self.file_rows):
            return
        resolved = _resolve(self.file_rows, spec.file_id, link.target)
        source_node = self._interned(self._link_source_node(spec.file_id, link))
        if resolved is None:
            morph = MORPH_BY_COLUMN.get(link.column, FROM_MORPH_INDEX)
            self.vectors[morph].append((source_node, None))
            return
        morph = self._link_morph(spec, link, resolved)
        if morph is None:
            return
        target = self._interned(_canonical_id(resolved))
        self.vectors[morph].append((source_node, target))

    def _link_morph(self, spec: ParsedSpec, link: Link, resolved: str) -> Optional[int]:
        source_kind = None
        if link.field in TABLE_FIELDS:
            source_kind = self.kinds.get("{}.{}".format(spec.file_id, link.source))
        target_kind = self.kinds.get(resolved)
        row = _classify(link.column, source_kind, target_kind)
        if row is FORBIDDEN:
            return None
        if row is None:
            return MORPH_BY_COLUMN.get(link.column, FROM_MORPH_INDEX)
        return SCHEMA.index(row)

    @staticmethod
    def _link_source_node(file_id: str, link: Link) -> str:
        if link.source == file_id:
            return file_id
        return "{}.{}".format(file_id, link.source)

    def domain(self, obj: str) -> List[str]:
        return self.members.get(obj, [])

    def morphism_values(self, name: str) -> List[Tuple[str, Optional[str]]]:
        values: List[Tuple[str, Optional[str]]] = []
        for row, vector in zip(SCHEMA, self.vectors):
            if row.name == name:
                values.extend(vector)
        return values

    # --- kernel evaluation -------------------------------------------------

    def evaluate(self, expr: tuple) -> str:
        return self._eval(expr, {})

    def _term(self, term: tuple, scope: Dict[str, str]) -> tuple:
        tag = term[0]
        if tag == "var":
            node = scope.get(term[1])
            return ("unknown",) if node is None else ("id", node)
        if tag == "id":
            return ("id", term[1]) if term[1] in self.declared else ("unknown",)
        if tag == "int":
            return ("int", term[1])
        if tag == "card":
            return ("int", len(self.domain(term[1])))
        if tag == "dangling":
            return ("dangling",)
        if tag == "proj":
            _, var, morph = term
            source = scope.get(var)
            if source is None or source not in self.nodes:
                return ("unknown",)
            for from_id, to_id in self.morphism_values(morph):
                if from_id == source:
                    return ("dangling",) if to_id is None else ("id", to_id)
            return ("dangling",)
        raise EvaluationError("unknown term {!r}".format(term))

    @staticmethod
    def _eq(left: tuple, right: tuple) -> str:
        if left[0] == "int" and right[0] == "int":
            return VERIFIED if left[1] == right[1] else COUNTEREXAMPLE
        if left[0] == "id" and right[0] == "id":
            return VERIFIED if left[1] == right[1] else COUNTEREXAMPLE
        if left[0] == "dangling" and right[0] == "dangling":
            return VERIFIED
        if {left[0], right[0]} == {"id", "dangling"}:
            return COUNTEREXAMPLE
        return UNKNOWN

    @staticmethod
    def _cmp(op: str, left: tuple, right: tuple) -> str:
        if left[0] != "int" or right[0] != "int":
            return UNKNOWN
        a, b = left[1], right[1]
        holds = {
            "<": a < b,
            "<=": a <= b,
            ">": a > b,
            ">=": a >= b,
        }[op]
        return VERIFIED if holds else COUNTEREXAMPLE

    def _adjacency(self, morphisms: Iterable[str]) -> Dict[str, Set[str]]:
        adjacency: Dict[str, Set[str]] = {}
        names = set(morphisms)
        for row, vector in zip(SCHEMA, self.vectors):
            if row.name not in names:
                continue
            for from_id, to_id in vector:
                if to_id is not None:
                    adjacency.setdefault(from_id, set()).add(to_id)
        return adjacency

    @staticmethod
    def _closure(
        adjacency: Dict[str, Set[str]], seeds: Iterable[str]
    ) -> Set[str]:
        seen: Set[str] = set(seeds)
        stack = list(seen)
        while stack:
            node = stack.pop()
            for next_node in adjacency.get(node, ()):
                if next_node not in seen:
                    seen.add(next_node)
                    stack.append(next_node)
        return seen

    def _cyclic_nodes(self, morphism: str) -> Set[str]:
        adjacency = self._adjacency((morphism,))
        cyclic: Set[str] = set()
        for from_id, to_id in self.morphism_values(morphism):
            if to_id is None:
                continue
            if from_id in self._closure(adjacency, (to_id,)):
                cyclic.add(from_id)
        return cyclic

    def _atomic(self, atomic: tuple, scope: Dict[str, str]) -> str:
        tag = atomic[0]
        if tag == "resolves":
            values = self.morphism_values(atomic[1])
            return VERIFIED if all(to is not None for _, to in values) else COUNTEREXAMPLE
        if tag == "unique":
            targets = [
                to for _, to in self.morphism_values(atomic[1]) if to is not None
            ]
            return VERIFIED if len(targets) == len(set(targets)) else COUNTEREXAMPLE
        if tag == "acyclic":
            return (
                VERIFIED if not self._cyclic_nodes(atomic[1]) else COUNTEREXAMPLE
            )
        if tag == "reachable":
            _, from_id, to_id, morphism = atomic
            if from_id not in self.nodes:
                return UNKNOWN
            reached = self._closure(self._adjacency((morphism,)), (from_id,))
            if to_id not in self.declared:
                return UNKNOWN
            return VERIFIED if to_id in reached else COUNTEREXAMPLE
        if tag == "eq":
            return self._eq(self._term(atomic[1], scope), self._term(atomic[2], scope))
        if tag == "cmp":
            return self._cmp(
                atomic[1], self._term(atomic[2], scope), self._term(atomic[3], scope)
            )
        raise EvaluationError("unknown atomic {!r}".format(atomic))

    def _eval(self, expr: tuple, scope: Dict[str, str]) -> str:
        tag = expr[0]
        if tag == "atomic":
            return self._atomic(expr[1], scope)
        if tag == "not":
            return _negate(self._eval(expr[1], scope))
        if tag == "and":
            return self._conjunction(expr, scope)
        if tag == "forall":
            return self._for_all(expr, scope)
        if tag == "exists":
            return self._exists(expr, scope)
        raise EvaluationError("unknown expression {!r}".format(expr))

    def _conjunction(self, expr: tuple, scope: Dict[str, str]) -> str:
        left = self._eval(expr[1], scope)
        right = self._eval(expr[2], scope)
        if left == COUNTEREXAMPLE or right == COUNTEREXAMPLE:
            return COUNTEREXAMPLE
        if left == VERIFIED and right == VERIFIED:
            return VERIFIED
        return UNKNOWN

    def _for_all(self, expr: tuple, scope: Dict[str, str]) -> str:
        _, var, obj, body = expr
        saw_unknown = False
        for node in self.domain(obj):
            status = self._eval_bound(body, var, node, scope)
            if status == COUNTEREXAMPLE:
                return COUNTEREXAMPLE
            if status == UNKNOWN:
                saw_unknown = True
        return UNKNOWN if saw_unknown else VERIFIED

    def _exists(self, expr: tuple, scope: Dict[str, str]) -> str:
        _, var, obj, body = expr
        saw_unknown = False
        for node in self.domain(obj):
            status = self._eval_bound(body, var, node, scope)
            if status == VERIFIED:
                return VERIFIED
            if status == UNKNOWN:
                saw_unknown = True
        return UNKNOWN if saw_unknown else COUNTEREXAMPLE

    def _eval_bound(
        self, body: tuple, var: str, node: str, scope: Dict[str, str]
    ) -> str:
        bound = dict(scope)
        bound[var] = node
        return self._eval(body, bound)


# ---------------------------------------------------------------------------
# The closed kernel grammar (recursive descent, src/kernel.rs)
# ---------------------------------------------------------------------------


def _is_ident_start(char: str) -> bool:
    return char.isascii() and (char.isalpha() or char == "_")


def _is_ident_continue(char: str) -> bool:
    return char.isascii() and (char.isalnum() or char in "_-")


class Parser:
    def __init__(self, text: str) -> None:
        self.s = text
        self.pos = 0
        self.vars: List[Tuple[str, str]] = []

    def skip_ws(self) -> None:
        rest = self.s[self.pos :]
        self.pos += len(rest) - len(rest.lstrip())

    def peek(self) -> Optional[str]:
        return self.s[self.pos] if self.pos < len(self.s) else None

    def eat(self, token: str) -> bool:
        self.skip_ws()
        if self.s.startswith(token, self.pos):
            self.pos += len(token)
            return True
        return False

    def expect(self, token: str, what: str) -> None:
        if not self.eat(token):
            raise KernelError(
                "kernel grammar expects {} `{}` at `{}`".format(
                    what, token, self._rest_head()
                )
            )

    def _rest_head(self) -> str:
        return self.s[self.pos : self.pos + 24]

    def ident(self) -> Optional[str]:
        self.skip_ws()
        end = self.pos
        first = True
        while end < len(self.s):
            char = self.s[end]
            if first:
                ok = _is_ident_start(char)
            else:
                ok = _is_ident_continue(char)
            if not ok:
                break
            end += 1
            first = False
        if end == self.pos:
            return None
        word = self.s[self.pos : end]
        self.pos = end
        return word

    def qualified_ident(self) -> Optional[str]:
        start = self.pos
        first = self.ident()
        if first is None:
            return None
        word = first
        while self.pos < len(self.s) and self.s[self.pos] == ".":
            self.pos += 1
            part = self.ident()
            if part is None:
                self.pos = start
                return None
            word += "." + part
        return word

    def parse(self) -> tuple:
        self.skip_ws()
        if self.pos >= len(self.s):
            raise KernelError("empty kernel expression")
        expr = self.expr()
        self.skip_ws()
        if self.pos != len(self.s):
            raise KernelError(
                "kernel expression has trailing input `{}` — the cell must be "
                "exactly one kernel expression".format(self.s[self.pos :])
            )
        return expr

    def expr(self) -> tuple:
        acc = self.unary()
        while True:
            self.skip_ws()
            if self.pos < len(self.s) and self.s[self.pos] == "∧":
                self.pos += 1
                acc = ("and", acc, self.unary())
            else:
                return acc

    def unary(self) -> tuple:
        self.skip_ws()
        if self.pos < len(self.s) and self.s[self.pos] == "¬":
            self.pos += 1
            return ("not", self.unary())
        return self.primary()

    def primary(self) -> tuple:
        self.skip_ws()
        char = self.peek()
        if char == "∀":
            return self.quantifier(True)
        if char == "∃":
            return self.quantifier(False)
        if char == "(":
            self.expect("(", "an opening paren")
            inner = self.expr()
            self.expect(")", "a closing paren")
            return inner
        return self.atomic()

    def quantifier(self, forall: bool) -> tuple:
        token = "∀" if forall else "∃"
        self.expect(token, "a quantifier")
        var = self.ident()
        if var is None:
            raise KernelError(
                "kernel grammar expects a bound variable name after the quantifier"
            )
        self.expect("∈", "the membership symbol in a quantifier")
        obj = self.ident()
        if obj is None:
            raise KernelError("kernel grammar expects an object after `∈`")
        if obj not in OBJECTS:
            raise KernelError(
                "unknown quantifier object `{}` — the schema's objects are {{{}}}".format(
                    obj, ", ".join(OBJECTS)
                )
            )
        self.expect(":", "the colon opening a quantifier body")
        self.vars.append((var, obj))
        body = self.expr()
        self.vars.pop()
        return ("forall" if forall else "exists", var, obj, body)

    def atomic(self) -> tuple:
        self._reject_unknown_call()
        self.skip_ws()
        reference = self._reference_atomic()
        if reference is not None:
            return ("atomic", reference)
        return self._comparison()

    def _reject_unknown_call(self) -> None:
        self.skip_ws()
        word = self.ident()
        if word is None:
            return
        rest = self.s[self.pos :].lstrip()
        if rest.startswith("(") and word not in REFERENCE_ATOMICS:
            raise KernelError(
                "unknown atomic `{}` — the closed atomic set is "
                "{{resolves, unique, acyclic, reachable, ==, <, <=, >, >=, "
                "∀, ∃, ∧, ¬}}".format(word)
            )
        self.pos -= len(word)

    def _reference_atomic(self) -> Optional[tuple]:
        for keyword, is_acyclic in (
            ("resolves", False),
            ("unique", False),
            ("acyclic", True),
        ):
            if not self._at(keyword):
                continue
            rest = self.s[self.pos + len(keyword) :]
            if rest.lstrip().startswith("("):
                self.pos += len(keyword)
                return self._simple_atomic(keyword, is_acyclic)
        return self._reachable_atomic()

    def _simple_atomic(self, keyword: str, is_acyclic: bool) -> tuple:
        morphism = self.morphism_arg(keyword)
        if is_acyclic and not self._endo(morphism):
            raise KernelError(
                "acyclic requires an endo morphism (source object == "
                "target object) — `{}` is not one".format(morphism)
            )
        return keyword, morphism

    def _reachable_atomic(self) -> Optional[tuple]:
        if not self._at("reachable"):
            return None
        rest = self.s[self.pos + len("reachable") :]
        if not rest.lstrip().startswith("("):
            return None
        self.pos += len("reachable")
        return self._reachable_args()

    def _reachable_args(self) -> tuple:
        self.expect("(", "the argument list of reachable")
        from_id = self.qualified_ident()
        if from_id is None:
            raise KernelError("reachable expects a from id")
        self.expect(",", "the comma after reachable's from id")
        to_id = self.qualified_ident()
        if to_id is None:
            raise KernelError("reachable expects a to id")
        self.expect(",", "the comma after reachable's to id")
        morphism = self.ident()
        if morphism is None:
            raise KernelError("reachable expects a morphism name")
        if morphism not in MORPHISM_NAMES:
            raise KernelError(
                "unknown morphism `{}` in reachable — the schema's "
                "morphisms are the Reference Typing rows".format(morphism)
            )
        self.expect(")", "the closing paren of reachable")
        return "reachable", from_id, to_id, morphism

    def _comparison(self) -> tuple:
        left = self.term()
        self.skip_ws()
        rest = self.s[self.pos :]
        if rest.startswith("=="):
            self.pos += 2
            return ("atomic", ("eq", left, self.term()))
        for token, op in (("<=", "<="), (">=", ">="), ("<", "<"), (">", ">")):
            if rest.startswith(token):
                self.pos += len(token)
                return ("atomic", ("cmp", op, left, self.term()))
        raise KernelError(
            "kernel grammar expects an equality or comparison operator at `{}`".format(
                self._rest_head()
            )
        )

    def _at(self, token: str) -> bool:
        return self.s.startswith(token, self.pos)

    def _endo(self, name: str) -> bool:
        return all(
            row.source == row.target for row in SCHEMA if row.name == name
        )

    def morphism_arg(self, atomic: str) -> str:
        self.expect("(", "the argument list of {}".format(atomic))
        morphism = self.ident()
        if morphism is None:
            raise KernelError("{} expects a morphism name".format(atomic))
        self.expect(")", "the closing paren of {}".format(atomic))
        if morphism not in MORPHISM_NAMES:
            raise KernelError(
                "unknown morphism `{}` in {} — the schema's morphisms are the "
                "Reference Typing rows".format(morphism, atomic)
            )
        return morphism

    def term(self) -> tuple:
        self.skip_ws()
        cardinality = self._cardinality_term()
        if cardinality is not None:
            return cardinality
        if self._at("⊥"):
            self.pos += 1
            return ("dangling",)
        integer = self._integer_term()
        if integer is not None:
            return integer
        return self._identifier_term()

    def _cardinality_term(self) -> Optional[tuple]:
        if not self._at("|"):
            return None
        self.pos += 1
        obj = self.ident()
        if obj is None:
            raise KernelError("cardinality term expects an object")
        self.expect("|", "the closing bar of a cardinality term")
        if obj not in OBJECTS:
            raise KernelError(
                "unknown cardinality object `{}` — the schema's objects are "
                "{{{}}}".format(obj, ", ".join(OBJECTS))
            )
        return ("card", obj)

    def _integer_term(self) -> Optional[tuple]:
        self.skip_ws()
        negative = self._at("-")
        digits_start = self.pos + 1 if negative else self.pos
        digits_end = digits_start
        while digits_end < len(self.s) and self.s[digits_end] in "0123456789":
            digits_end += 1
        if digits_end == digits_start:
            return None
        digits = self.s[digits_start:digits_end]
        literal = "-" + digits if negative else digits
        value = -int(digits) if negative else int(digits)
        if value < -(2**63) or value > 2**63 - 1:
            raise KernelError(
                "integer literal out of range at `{}`".format(literal)
            )
        self.pos = digits_end
        return ("int", value)

    def _identifier_term(self) -> tuple:
        word = self.ident()
        if word is None:
            raise KernelError(
                "kernel grammar expects a term at `{}`".format(self._rest_head())
            )
        self.skip_ws()
        bound = any(var == word for var, _ in self.vars)
        if self._at("."):
            if bound:
                return self._projection(word)
            return self._qualified_id(word)
        return ("var", word) if bound else ("id", word)

    def _projection(self, word: str) -> tuple:
        self.pos += 1
        morphism = self.ident()
        if morphism is None:
            raise KernelError("projection expects a morphism name after `.`")
        obj = next(obj for var, obj in reversed(self.vars) if var == word)
        if not any(row.name == morphism and row.source == obj for row in SCHEMA):
            raise KernelError(
                "morphism `{}` is not sourced on {} — `{}.{}` is ill-formed "
                "here".format(morphism, obj, word, morphism)
            )
        return ("proj", word, morphism)

    def _qualified_id(self, word: str) -> tuple:
        identifier = word
        while self._at("."):
            self.pos += 1
            part = self.ident()
            if part is None:
                raise KernelError(
                    "a file-qualified id literal continues after every dot"
                )
            identifier += "." + part
        return ("id", identifier)


# ---------------------------------------------------------------------------
# The request boundary
# ---------------------------------------------------------------------------


def evaluate_request(request: object) -> str:
    if not isinstance(request, dict):
        raise RequestError("the request must be a JSON object")
    specs_raw = request.get("specs")
    expr_raw = request.get("expr")
    if not isinstance(specs_raw, list) or not all(
        isinstance(item, str) for item in specs_raw
    ):
        raise RequestError('the request must carry `"specs": ["<markdown>", ...]`')
    if not isinstance(expr_raw, str):
        raise RequestError('the request must carry a string `"expr"`')
    specs = [parse_spec(text, index) for index, text in enumerate(specs_raw)]
    expr = Parser(expr_raw).parse()
    return Env(specs).evaluate(expr)


def main() -> int:
    """Evaluate one kernel request from stdin and print the JSON response."""
    try:
        request = json.loads(sys.stdin.read())
    except ValueError as error:
        sys.stderr.write("kernel_reference.py: request: invalid JSON: {}\n".format(error))
        return EXIT_REQUEST
    try:
        status = evaluate_request(request)
    except RequestError as error:
        sys.stderr.write(
            "kernel_reference.py: {}: {}\n".format(error.label, error)
        )
        return EXIT_REQUEST
    except AdapterError as error:
        sys.stderr.write(
            "kernel_reference.py: {}: {}\n".format(error.label, error)
        )
        return EXIT_FAILURE
    except Exception as error:  # noqa: BLE001 - never fabricate a verdict
        sys.stderr.write(
            "kernel_reference.py: internal error: {}: {}\n".format(
                type(error).__name__, error
            )
        )
        return EXIT_FAILURE
    sys.stdout.write(json.dumps({"status": status}) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())