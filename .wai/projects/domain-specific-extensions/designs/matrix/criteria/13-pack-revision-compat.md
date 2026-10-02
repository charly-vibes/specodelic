# pack-revision-compat

Measures whether the mechanism tracks which format_revision a pack extends
and what happens when the corpus moves past it (a pack authored against
Revision N meeting a corpus at N+1) — mistral-science had to *guess* the
next revision number (13); grok-bioimage explicitly deferred Revision work.
