# honest-failure

Are the honest-failure mechanics preserved: `todo_predicate!` panics at execution, never at emission (`src/verify.rs:15`, `src/compile.rs:675,771`); no-emitter tags fail labeled with remediation (`src/compile.rs:389`); prose guards report `invariants_checked: []` and never fabricate a counterexample (`src/model_check.rs:21,116,204`)? Every new pass must fail open the same way.
