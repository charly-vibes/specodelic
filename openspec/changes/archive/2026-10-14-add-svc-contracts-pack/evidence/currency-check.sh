#!/usr/bin/env bash
# currency-check.sh — how does the analysed source (zip commit) relate to the
# upstream release tag and upstream main, and does upstream already do any of
# what the proposals add? Needs network access to github.com (git protocol).
#
# Usage: [CLONE=/path/to/existing/blobless/clone] bash currency-check.sh
set -u
Z=2939a0b2abff5f7d6d9f9959464431684a728491          # commit from the zip's archive comment
URL=https://github.com/charly-vibes/specodelic.git
echo "checked at (UTC): $(date -u +%Y-%m-%dT%H:%MZ)"
echo "== C1 remote refs"; git ls-remote "$URL" refs/tags/v0.8.0 'refs/tags/v0.8.0^{}' refs/tags/v0.10.0 'refs/tags/v0.10.0^{}' HEAD 2>&1 | cut -c1-90
if [ -z "${CLONE:-}" ]; then CLONE="$(mktemp -d)/sp"; git clone -q --filter=blob:none --no-checkout "$URL" "$CLONE" || exit 2; fi
cd "$CLONE" || exit 2
git fetch -q origin 2>&1 | tail -1
S=49e1a2e06255df8ec50840fa8eff5e9e96347b99          # upstream snapshot the "main" evidence and validation were run against
T=$(git rev-parse 'v0.8.0^{commit}'); M=$(git rev-parse origin/HEAD)
R=$(git rev-parse 'v0.10.0^{commit}')
echo; echo "== C2 relationship of the zip commit"
echo "zip commit exists upstream:        $(git cat-file -t $Z 2>&1)"
echo "v0.8.0 tag commit:                 $T"
echo "upstream default-branch tip (fetched): $M"
echo "analysed upstream snapshot:           $S"
echo "tag is ancestor of zip commit:     $(git merge-base --is-ancestor $T $Z && echo yes || echo no)   (commits tag..zip: $(git rev-list --count $T..$Z))"
echo "zip commit is ancestor of tip:     $(git merge-base --is-ancestor $Z $M && echo yes || echo no)   (commits zip..tip: $(git rev-list --count $Z..$M))"
echo; echo "== C3 files changed under src/ packs/ specs/ openspec/specs/ between tag and zip commit"
git diff --stat $T $Z -- src packs specs openspec/specs | tail -3; echo "(empty = none)"
echo; echo "== C4 upstream tip: do the paths the proposals cite still exist?"
for p in specs/USAGE.md specs/AGENTS.md specs/theory.md specs/CHANGELOG.md specs/specodelic.md specs/packs.md \
         openspec/USAGE.md openspec/AGENTS.md openspec/theory.md openspec/CHANGELOG.md openspec/specs/specodelic/spec.md \
         src/acset/query.rs src/merge.rs src/citation_corpus.rs src/packs.rs openspec/specs/packs/spec.md; do
  git cat-file -e $M:$p 2>/dev/null && s=present || s=MISSING; printf "%-40s %s\n" "$p" "$s"; done
echo; echo "== C5 upstream tip: code anchors the design cites"
git show $M:src/acset/query.rs | grep -n "pub fn blast_radius" | head -1
git show $M:src/merge.rs       | grep -n "blast_radius(&instance, &existing, &instance.morphism_names())" | head -1
echo "discovery anchor (next_back) lines in src/packs.rs:"; git show $M:src/packs.rs | grep -n "fn discovery_root\|\.next_back()" | head -4
echo; echo "== C6 upstream tip: any workspace/affected verb, svc pack, or related capability?"
git grep -n -i -E "Commands::Workspace|Commands::Affected|\"workspace\"" $M -- src/main.rs | head -3; echo "(empty above = none in src/main.rs)"
git ls-tree -r --name-only $M -- openspec/specs packs | grep -i -E "workspace|svc|affected" | head -3; echo "(empty above = none)"
echo "upstream packs: $(git ls-tree --name-only $M -- packs/ | tr '\n' ' ')"
echo "upstream version: $(git show $M:Cargo.toml | grep -m1 '^version')"
echo "upstream ci pin:  $(git show $M:.github/workflows/ci.yml | grep -m1 'openspec@')"

echo; echo "== C7 binary vs analysed upstream snapshot, and snapshot vs fetched tip"
echo "v0.10.0 tag commit:                $R  (ancestor of snapshot: $(git merge-base --is-ancestor $R $S && echo yes || echo no); commits tag..snapshot: $(git rev-list --count $R..$S))"
echo "files changed under src/ packs/ openspec/specs/ between v0.10.0 tag and snapshot:"; git diff --stat $R $S -- src packs openspec/specs | tail -3; echo "(empty = none)"
echo "commits snapshot..fetched tip: $(git rev-list --count $S..$M)"
echo "relevant files changed snapshot..tip (src/packs.rs merge.rs query.rs citation_corpus.rs, openspec/specs/packs, openspec/changes, justfile, .github):"
git diff --stat $S $M -- src/packs.rs src/merge.rs src/acset/query.rs src/citation_corpus.rs openspec/specs/packs openspec/changes justfile .github | tail -4
