# Status quo

Keep the bimodal design as-is at Revision 16: cells with a `**rust:**` fragment become executable (invariant-kind Constraints only); everything else is structurally linted, semantically opaque prose. External tools bind through byte-stable artifacts + contract-TOML `flags` only. Every matrix starts with this column — it must show what is wrong with today, not only what works.
