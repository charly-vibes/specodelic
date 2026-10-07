# Design: TypeScript property execution

## Test shape first

Reuse generator conformance fixtures from l8l: signed 32-bit endpoints,
Unicode scalars (including non-BMP characters), empty/bounded collections,
choices and named definitions. Add mixed three-language property sets,
a failing fast-check property with a shrunk example, missing tools, stale
registry/artifact, skipped tests, and subprocess timeout. Assert exact
property IDs and block completeness, not random sequence identity.

## Decisions

1. Emit *_props.ts from the shared generator IR. Integer domain stays
   signed 32-bit (exactly representable in JavaScript); string length is
   measured in Unicode scalar values, not UTF-16 code units. Compose scalar
   generators explicitly when the library default differs. Typed list values
   and named expansion follow the existing registry contract unchanged.
   Reuse recursive choice type checking: all branches must have identical
   types after alias expansion, including nested and zero-length lists.
   Predicate v0 is number, string or Array<T>; no union/coercion fallback.
2. Use the project's installed Node and project-local Vitest/fast-check,
   resolved from explicit --project-root. Missing packages fail labeled.
   Never use an npx invocation that downloads on demand. Stage uniquely named
   artifacts in a controlled temporary location resolvable by that project;
   clean up on every terminal path. Document any project configuration loaded.
3. Unit **ts:** predicate expressions opt in. Existing restrictions on
   executable law fragments and invariant/guard positions remain labeled.
   Metadata binds row ID, language, generator IR, predicate and dependencies;
   test discovery cannot silently omit an expected row.
4. Use structured results with fast-check counterexample/shrink metadata.
   Timeouts terminate the process tree; unfinished shrinking is labeled
   unshrunk. Mix Rust, Python, TypeScript only through shared complete-block
   accounting; one failure blocks the overall property gate.
5. Kernel agreement established by 36n/l8l remains mandatory and unweakened.
   This follow-up adds generator-domain conformance and property execution,
   not a TypeScript kernel evaluator. Extending kernel backend agreement to
   TypeScript requires a separately scoped evaluator; don't compare arbitrary
   language-specific predicates as though they were translations.
6. Rebase the complete compile snapshot against deployed Python support
   before implementation/archival. Preserve all kernel, language grammar,
   pure-widening and law-case requirements. No generator vocabulary fork.
