#!/usr/bin/env bash
# Normalize the change id passed to `just archive-change`.
#
# `just` treats `id=<value>` as a positional value, so the corpus's documented
# invocation form (`just archive-change id=add-check-claim-coverage`) reaches
# the recipe with the literal `id=` prefix attached (specodelic-4yve). The
# prefix is stripped here so the companion always receives the bare change id.
#
# Usage: scripts/archive_change_id.sh <id>
# Prints the normalized change id on stdout; exits 2 with a remediation hint
# when the id is empty.
set -euo pipefail

id="${1-}"
id="${id#id=}"

if [ -z "$id" ]; then
    echo "archive-change: missing change id" >&2
    echo "hint: usage: just archive-change <change-id>   (e.g. just archive-change add-check-claim-coverage); the 'id=<change-id>' form is accepted too" >&2
    exit 2
fi

printf '%s\n' "$id"
