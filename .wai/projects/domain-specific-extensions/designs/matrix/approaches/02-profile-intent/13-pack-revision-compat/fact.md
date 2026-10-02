The manifest carries a base format_revision (approach description), so
doctor/lint can warn when the corpus revision moves past a pack's base —
the knowledge-currency warning pattern already shipped in spk doctor
(Revision N vs embedded FORMAT_REVISION) generalizes to per-pack bases.
