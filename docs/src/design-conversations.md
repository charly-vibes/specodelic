# Design conversations — the `specodelic-design` skill

> The one-line summary: *run design conversations that end in a
> lint-clean Specodelic spec (a four-layer markdown format — Intent,
> Constraints, Model, Properties).*

The `specodelic-design` skill is a guided conversation procedure —
Listen → Intent → Constraints → Model → Properties → Assemble — that
ends in a spec the toolchain accepts. It ships inside the `spk` binary
as a managed skill pack: the protocol prose, a format reference rendered
from the same tables `spk explain` serves, and the book's worked example
embedded verbatim.

**The procedure's single authoring surface** is the tracked source
[`skill-packs/specodelic-design/SKILL.md`](../../skill-packs/specodelic-design/SKILL.md)
— this page deliberately does not duplicate it. The worked example is
[reservation-order](examples/reservation-order.md), the same file the
binary embeds, so the skill and the book can never disagree.

**Install** it into your agent's skill directory:

```console
$ spk skill install            # default: ~/.claude/skills
$ spk skill install --dir ~/.config/agents/skills   # any target
```

Every installed file carries a managed marker; `spk doctor` reports a
stale install (older binary) with the regenerate command, and a missing
one only as an optional hint.

For the thinking behind the four layers the conversation builds, see the
[design canvas](design-canvas.md); for the questions a cold read asks,
see the [FAQ](faq.md).
