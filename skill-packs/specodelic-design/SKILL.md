---
name: specodelic-design
description: Run design conversations that end in a lint-clean Specodelic spec (a four-layer markdown format - Intent, Constraints, Model, Properties). Use whenever the user wants to design, specify, spec out, or pin down the behavior of a feature, system, or workflow, mentions Specodelic / spk / EARS statements / constraints-and-properties tables, or asks to turn a discussion, requirements list, or OpenSpec file into a spec. Use it even if they only say "let's design X" or "write the requirements for X" and a Specodelic corpus or spec file is in play.
---

# Specodelic design conversations

Specodelic is a markdown spec format: **one file = one spec**, made of four
layers (Intent, Constraints, Model, Properties) that a linter (`spk lint`)
checks. Your job in a design conversation is to help the user think, and to
build the spec up in stages as the thinking firms up - not to dump a finished
file on turn one.

Read `references/format.md` before writing any spec text. Read
`references/example.md` when you need a concrete shape to imitate.

## The conversation protocol

Work through these stages. Move on only when the user has confirmed the
current one. Skip stages the user has already settled.

1. **Listen (no format yet).** Let the user describe the problem. Ask
   about who/what triggers behavior, what must never happen, and what
   happens on failure. Do not mention table schemas yet.
2. **Intent.** Propose ONE statement in an EARS pattern with `SHALL`, plus
   the file `id` (derived from the filename, see format.md). Show just the
   frontmatter. Ask whether it captures the core promise.
3. **Constraints.** Turn the discussion into rows: each is a single,
   checkable claim, with a `kind` and a file-qualified `traces_to`. Probe
   edge cases ("what if two requests race?", "what if it's retried?") -
   good constraints usually come from those answers. Show the full table.
4. **Model.** Propose states and transitions. Every transition needs a
   guard that cites an `invariant` constraint or a state. Show the lifecycle
   as a short list first if the user wants to discuss shape.
5. **Properties.** One or more property per constraint, each deriving from
   it. These are the testable statements.
6. **Assemble.** Present the whole file in one code block, named by its
   filename, and run the self-check below.

At each stage show only the layer being discussed, plus a one-line summary
of what is already agreed. Revise in place and keep row ids stable across
edits so earlier references do not break.

## Self-check before presenting a draft

You usually cannot run the linter in chat, so check these by hand. They are
the rules that most often fail:

- `id` equals the filename stem with `-` read as `.` (`reservation-order.md`
  -> `reservation.order`); `_` stays literal.
- `statement` matches exactly one EARS pattern and contains `SHALL`.
- Every `kind` is from the closed sets in format.md; never invent one.
- Every Constraint has at least one Property whose `derives_from` points at it.
- Every reference is **file-qualified**, e.g. `[[reservation.order.all_or_nothing]]`.
  Bare text like `all_or_nothing` creates no reference.
- Every transition has a non-empty guard; guards cite only `invariant`
  constraints or states; a state's `emits` cites only `effect` constraints.
- Every state appears in at least one transition; every transition endpoint
  is a declared state.
- A literal `|` inside a table cell is written `\|`.
- Ids are unique within the file.

## Honesty about verification

- Label any draft you produced without running the linter **"unlinted"** and
  give the command to check it: `spk lint <filename>`.
- Never say a spec is "verified". Verification is bounded model checking of
  the claims the spec opts into; prose-only invariants stay unchecked, and it
  does not replace the application's own tests.
- If the user pastes linter output, treat each finding's rule id as the
  ground truth and fix exactly that.

## Out of scope for this skill

Domain packs (`kind: profile`) and OpenSpec dual-format files have extra
rules. If the user needs them, say so and point to `spk explain packs` and
`spk explain dual-format` rather than guessing.

If a shell and the `spk` binary are available, prefer `spk explain <topic>`
for authoritative detail (topics: format, ears, kinds, references,
lifecycle, lint-rules, dual-format, packs, graph-views) and run `spk lint` on each draft.
