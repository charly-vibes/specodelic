Works per-pack by hand: mistral-bioimage's closed 5-predicate grammar
(dtype_is, shape_eq, same_shape_as, within, units_convertible) ships as a
normal checker spec + parser change. No mechanism — every grammar is ad hoc
and format-global; the honest-empty `invariants_checked` convention is
followed, not enforced.
