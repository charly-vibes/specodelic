#!/usr/bin/env bash
# run-evidence.sh — reproduces every empirical claim in the bundle's README.
#
# Usage:  SPK=/path/to/spk REPO=/path/to/specodelic-checkout \
#         PACK=/path/to/repo-overlay/packs/svc-contracts.md  bash run-evidence.sh
#
#   SPK   the specodelic binary under test (bundle was produced with v0.8.0)
#   REPO  an unpacked specodelic checkout (for the vocabulary audit and the
#         "no pack, no change" probe); the bundle used the zip's tree
#   PACK  the draft pack file proposed by add-svc-contracts-pack
#
# Everything is built under a fresh temp dir; nothing in REPO is modified.
# Output is plain text, one "== <id> ..." header per evidence item.
set -u
: "${SPK:?set SPK}"; : "${REPO:?set REPO}"; : "${PACK:?set PACK}"
W="$(mktemp -d)"; cd "$W" || exit 2
GIT="git -c user.email=e@x -c user.name=e"
show() { head -"${2:-6}" "$1" | cut -c1-"${3:-300}"; }
run()  { local id="$1"; shift; echo; echo "== $id"; "$@" > "$W/.o" 2>&1; echo "exit=$?"; show "$W/.o" "${N:-6}"; }

echo "spk: $("$SPK" --version 2>&1)"; echo "date: $(date -u +%Y-%m-%d)"

# ---------------------------------------------------------------- fixtures
mkdir -p orders/specs billing/specs
cat > orders/specs/orders-api.md <<'EOF'
---
id: orders.api
kind: intent
statement: "WHEN a client requests the order list, THE orders service SHALL return the caller's orders, and WHEN an order is placed, THE service SHALL publish an order-placed event."
---

# orders.api

## Constraints

| id           | kind            | expr                                              | traces_to      |
|--------------|-----------------|---------------------------------------------------|----------------|
| list_orders  | extension_point | `GET /orders returns the caller's orders`         | [[orders.api]] |
| order_placed | effect          | `an order-placed event is published on placement` | [[orders.api]] |
| valid_order  | invariant       | `only valid orders can be placed`                 | [[orders.api]] |

## Model

### States
- `idle`
- `placed`

### Transitions

| id    | from | to     | guard                                      |
|-------|------|--------|--------------------------------------------|
| place | idle | placed | [[orders.api.valid_order]] ∧ `valid order` |

## Properties

| id       | kind | derives_from                | generator           | predicate                    |
|----------|------|-----------------------------|---------------------|------------------------------|
| p_list   | unit | [[orders.api.list_orders]]  | `sample_orders()`   | `list returns caller orders` |
| p_placed | unit | [[orders.api.order_placed]] | `place_one_order()` | `event published`            |
| p_valid  | unit | [[orders.api.valid_order]]  | `place_invalid()`   | `rejected`                   |
EOF
cat > billing/specs/billing-client.md <<'EOF'
---
id: billing.client
kind: intent
statement: "WHEN billing needs an order list, THE billing service SHALL consume the orders contract, and WHEN an order is placed, THE billing service SHALL observe the order-placed event."
---

# billing.client

## Constraints

| id          | kind      | expr                                    | traces_to          | satisfies                  | observes                    |
|-------------|-----------|-----------------------------------------|--------------------|----------------------------|-----------------------------|
| uses_orders | invariant | `billing reads orders via the contract` | [[billing.client]] | [[orders.api.list_orders]] |                             |
| sees_placed | invariant | `billing invoices on order-placed`      | [[billing.client]] |                            | [[orders.api.order_placed]] |

## Model

### States
- `waiting`
- `invoiced`

### Transitions

| id      | from    | to       | guard                                         |
|---------|---------|----------|-----------------------------------------------|
| invoice | waiting | invoiced | [[billing.client.sees_placed]] ∧ `event seen` |

## Properties

| id     | kind | derives_from                   | generator         | predicate         |
|--------|------|--------------------------------|-------------------|-------------------|
| p_uses | unit | [[billing.client.uses_orders]] | `call_contract()` | `contract used`   |
| p_sees | unit | [[billing.client.sees_placed]] | `emit_placed()`   | `invoice created` |
EOF

# ------------------------------------------------- E1-E4 two repos, one corpus
run "E1 provider alone"                    "$SPK" lint orders/specs --human
run "E2 consumer alone (dangling = its ports)" "$SPK" lint billing/specs --human
run "E3a combined, provider first"         "$SPK" lint orders/specs billing/specs --human
run "E3b combined, consumer first"         "$SPK" lint billing/specs orders/specs --human
echo; echo "== E4a wiring view (TSV)"; "$SPK" graph orders/specs billing/specs --format edges --view wiring 2>&1 | head -5
echo; echo "== E4b all edges, satisfies/observes only"; "$SPK" graph orders/specs billing/specs --format edges 2>&1 | grep -E "satisfies|observes"

# ------------------------------------------------- E5-E6 duplicate identity
mkdir -p dup/specs; cp orders/specs/orders-api.md dup/specs/orders-api.md
for v in lint graph compile model-check; do N=3 run "E5 $v over orders/specs + verbatim duplicate dup/specs" "$SPK" $v orders/specs dup/specs --human; done
rm -rf specodelic
mkdir -p a/specs b/specs
sed 's/orders\.api/api/g' orders/specs/orders-api.md > a/specs/api.md
sed 's/orders\.api/api/g' orders/specs/orders-api.md > b/specs/api.md
N=4 run "E6a two repos each with specs/api.md (id: api) — lint" "$SPK" lint a/specs b/specs --human
N=4 run "E6b same — compile (output names derive from the file stem)" "$SPK" compile a/specs b/specs --human
echo "compile out dir:"; ls specodelic 2>/dev/null; rm -rf specodelic

# ------------------------------------------------- E7 pack discovery order
mkdir -p P/specs P/packs C/specs
cp "$REPO/packs/data-lineage.md" P/packs/
cp orders/specs/orders-api.md P/specs/orders-api.md
sed 's/| list_orders  | extension_point /| list_orders  | data.dataset    /' orders/specs/orders-api.md | sed 's/orders\.api/ledger.rows/g' > C/specs/ledger-rows.md
for r in P C; do (cd $r && git init -q && git add -A && $GIT commit -qm init); done
N=3 run "E7a consumer repo alone (pack lives in the other repo)" "$SPK" lint C/specs --human
N=3 run "E7b lint P/specs C/specs  (last path = consumer repo)"  "$SPK" lint P/specs C/specs --human
N=3 run "E7c lint C/specs P/specs  (last path = pack repo)"      "$SPK" lint C/specs P/specs --human

# ------------------------------------------------- E8 umbrella + submodule
mkdir -p U/packs U/specs; (cd U && git init -q && cp "$REPO/packs/data-lineage.md" packs/ && git add -A && $GIT commit -qm u \
  && git -c protocol.file.allow=always submodule add -q "$W/C" svc-c >/dev/null 2>&1 && $GIT commit -qm sub)
echo; echo "== E8 git toplevel seen from inside the submodule: $(git -C U/svc-c/specs rev-parse --show-toplevel)"
N=3 run "E8 lint the submodule's specs (umbrella holds the pack)" "$SPK" lint U/svc-c/specs --human
N=3 run "E8b lint from the umbrella root"                          "$SPK" lint U --human

# ------------------------------------------------- E9 merge over member trees
for t in base ours theirs; do mkdir -p wsm/$t/orders/specs wsm/$t/billing/specs; cp billing/specs/billing-client.md wsm/$t/billing/specs/; done
for t in base ours; do cp orders/specs/orders-api.md wsm/$t/orders/specs/; done
grep -v "list_orders\|p_list" orders/specs/orders-api.md > wsm/theirs/orders/specs/orders-api.md
N=5 run "E9a provider branch removes a contract row a consumer cites" "$SPK" merge wsm/ours --branch wsm/theirs --base wsm/base --human
cp -r wsm wsm2; cp orders/specs/orders-api.md wsm2/theirs/orders/specs/orders-api.md
python3 - <<'EOF'
p='wsm2/theirs/orders/specs/orders-api.md'; s=open(p).read()
s=s.replace("| valid_order  | invariant ","| extra_note   | invariant       | `extra`                                           | [[orders.api]] |\n| valid_order  | invariant ")
s=s.replace("| p_valid  | unit |","| p_extra  | unit | [[orders.api.extra_note]]   | `x()`               | `y`                          |\n| p_valid  | unit |")
open(p,'w').write(s)
EOF
N=3 run "E9b provider branch ADDS a row (additive)" "$SPK" merge wsm2/ours --branch wsm2/theirs --base wsm2/base --human
cp -r wsm wsm3
python3 - <<'EOF'
base=open('orders/specs/orders-api.md').read()
w=base.replace("| valid_order  | invariant ","| extra_note   | invariant       | `extra`                                           | [[orders.api]] |\n| valid_order  | invariant ").replace("| p_valid  | unit |","| p_extra  | unit | [[orders.api.extra_note]]   | `x()`               | `y`                          |\n| p_valid  | unit |")
for t in ('base','ours'): open(f'wsm3/{t}/orders/specs/orders-api.md','w').write(w)
open('wsm3/theirs/orders/specs/orders-api.md','w').write(base)
EOF
N=3 run "E9c provider branch removes a row NO other repo cites (limits of the claim)" "$SPK" merge wsm3/ours --branch wsm3/theirs --base wsm3/base --human

# ------------------------------------------------- E10 svc.contracts pack probes
mkdir -p pk/provider/specs pk/provider/packs pk/consumer/specs
cp "$PACK" pk/provider/packs/svc-contracts.md
N=3 run "E10a the pack file lints clean on its own" "$SPK" lint pk/provider/packs --human
cat > pk/provider/specs/orders-api.md <<'EOF'
---
id: orders.api
kind: intent
statement: "WHEN a client requests the order list, THE orders service SHALL return the caller's orders, and WHEN an order is placed, THE service SHALL publish an order-placed event."
---

# orders.api

## Interfaces

| name          | kind    | repo        | binding                             | evolution |
|---------------|---------|-------------|-------------------------------------|-----------|
| orders_list   | request | acme/orders | openapi/orders.yaml#/paths/~1orders | additive  |
| orders_placed | event   | acme/orders | asyncapi/events.yaml#/order-placed  | additive  |

## Constraints

| id           | kind            | expr                                              | traces_to      | published_as  |
|--------------|-----------------|---------------------------------------------------|----------------|---------------|
| list_orders  | extension_point | `GET /orders returns the caller's orders`         | [[orders.api]] | orders_list   |
| order_placed | effect          | `an order-placed event is published on placement` | [[orders.api]] | orders_placed |
| valid_order  | invariant       | `only valid orders can be placed`                 | [[orders.api]] |               |

## Model

### States
- `idle`
- `placed`

### Transitions

| id    | from | to     | guard                                      |
|-------|------|--------|--------------------------------------------|
| place | idle | placed | [[orders.api.valid_order]] ∧ `valid order` |

## Properties

| id       | kind       | derives_from                | generator           | predicate                                                                                                       |
|----------|------------|-----------------------------|---------------------|-----------------------------------------------------------------------------------------------------------------|
| p_list   | unit       | [[orders.api.list_orders]]  | `sample_orders()`   | `list returns caller orders`                                                                                    |
| p_placed | unit       | [[orders.api.order_placed]] | `place_one_order()` | `event published`                                                                                              |
| p_valid  | unit       | [[orders.api.valid_order]]  | `place_invalid()`   | `rejected`                                                                                                      |
| p_compat | svc.compat | [[orders.api.list_orders]]  | `change_cases()`    | `**added:** new optional field keeps consumers working; **removed:** dropping a field breaks consumers using it` |
EOF
cp billing/specs/billing-client.md pk/consumer/specs/
for r in provider consumer; do (cd pk/$r && git init -q && git add -A && $GIT commit -qm init); done
N=3 run "E10b pack in same repo: vocabulary activates the pack (draft status named)" "$SPK" lint pk/provider/specs --human
N=4 run "E10c RED: pack absent -> property_kind_closed + orphan_vocabulary" bash -c "cp -r pk/provider pk/red && rm pk/red/packs/svc-contracts.md && '$SPK' lint pk/red/specs --human"
python3 - <<'EOF'
import shutil
shutil.copytree('pk/provider','pk/dang')
p='pk/dang/specs/orders-api.md'; s=open(p).read()
s=s.replace("| [[orders.api]] | orders_list   |","| [[orders.api]] | no_such_row   |"); open(p,'w').write(s)
shutil.copytree('pk/provider','pk/nofloor')
p='pk/nofloor/specs/orders-api.md'; s=open(p).read()
s=s.replace("`**added:** new optional field keeps consumers working; **removed:** dropping a field breaks consumers using it`","`consumers keep working`"); open(p,'w').write(s)
EOF
N=3 run "E10d dangling published_as (no_such_row)" "$SPK" lint pk/dang/specs --human
N=3 run "E10e svc.compat row WITHOUT the floor's case labels" "$SPK" lint pk/nofloor/specs --human
N=3 run "E10f cross-repo, pack repo first (last path = consumer)" "$SPK" lint pk/provider/specs pk/consumer/specs --human
N=3 run "E10g cross-repo, pack repo last" "$SPK" lint pk/consumer/specs pk/provider/specs --human

# ------------------------------------------------- E11 no pack, no change
# NOTE: discovery anchors at the git toplevel (or, with no git root, at the lint
# target directory). A copy WITHOUT .git would never scan packs/, making this
# probe vacuous — so the copy gets its own git root, and a positive control
# proves the pack is discovered in this setup before the comparison is trusted.
cp -r "$REPO" repo-copy; rm -rf repo-copy/.git
(cd repo-copy && git init -q && git add -A >/dev/null 2>&1 && $GIT commit -qm snapshot >/dev/null 2>&1)
CORPUS_DIRS=""; for d in specs openspec; do [ -d "repo-copy/$d" ] && CORPUS_DIRS="$CORPUS_DIRS $d"; done
for d in $CORPUS_DIRS; do "$SPK" lint repo-copy/$d --human > base.$d.txt 2>&1; done
cp "$PACK" repo-copy/packs/svc-contracts.md
echo; echo "== E11 repo corpus lint output with vs without the draft pack (git root present; dirs:$CORPUS_DIRS)"
for d in $CORPUS_DIRS; do "$SPK" lint repo-copy/$d --human > with.$d.txt 2>&1; if cmp -s base.$d.txt with.$d.txt; then echo "$d: byte-identical ($(head -1 base.$d.txt | cut -c1-90))"; else echo "$d: DIFFERS"; diff base.$d.txt with.$d.txt | head -5; fi; done
mkdir -p repo-copy/ctl && cp pk/provider/specs/orders-api.md repo-copy/ctl/orders-api.md
N=3 run "E11 positive control: a spec using svc vocabulary, linted from a different dir of the same repo, must activate the pack" "$SPK" lint repo-copy/ctl --human
rm -rf repo-copy/ctl

# ------------------------------------------------- E12 vocabulary audit
echo; echo "== E12 vocabulary audit over $REPO  (ci = case-insensitive files / exact = exact-case files; specs vs openspec/specs)"
# AUDIT_A / AUDIT_B: the two corpus-prose dirs to audit (defaults match the zip
# commit: specs/ and openspec/specs/). A dir that does not exist is reported as
# MISSING instead of silently counting zero.
AUDIT_A="${AUDIT_A:-specs}"; AUDIT_B="${AUDIT_B:-openspec/specs}"
for d in "$AUDIT_A" "$AUDIT_B"; do [ -d "$REPO/$d" ] || echo "NOTE: audit dir $d is MISSING in this tree — its counts below are not meaningful"; done
audit() { local t="$1"; printf "%-16s ci=%s/%s exact=%s/%s\n" "$t" \
  "$(grep -rilw -- "$t" "$REPO/$AUDIT_A" 2>/dev/null | wc -l)" "$(grep -rilw -- "$t" "$REPO/$AUDIT_B" 2>/dev/null | wc -l)" \
  "$(grep -rlw  -- "$t" "$REPO/$AUDIT_A" 2>/dev/null | wc -l)" "$(grep -rlw  -- "$t" "$REPO/$AUDIT_B" 2>/dev/null | wc -l)"; }
for t in Interfaces Interface Services Contracts Rollout Peers Topology Repos svc svc.compat svc.contracts published_as compat evolution; do audit "$t"; done
echo; echo "(work dir: $W)"
