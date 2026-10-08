"""Consumer tests for the schema and state views (add-graph-views 2.3/2.4).

Feeds serialized artifacts — the exact shapes the spk commands emit — to
the graph_views renderers and pins each view's contract. The TestCase
classes live in schema_view_cases, state_view_cases and
state_view_scope_cases (split to honor the script-role structural
ratchet); this module re-imports them so the documented meter
(`python3 -m unittest discover -s scripts -p test_graph_views.py`) runs
the whole consumer suite.

These are consumer-side constructed fixtures; the producer-side pins
(the real TSV bytes over the same-shaped corpora) live in
tests/cli/parse_misc.rs's Rust suite. No Rust source is parsed here, no
corpus markdown is re-walked, and no second reference-typing table is
embedded.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from schema_view_cases import *  # noqa: F401,F403
from state_view_cases import *  # noqa: F401,F403
from state_view_scope_cases import *  # noqa: F401,F403

if __name__ == "__main__":
    import unittest

    unittest.main()
