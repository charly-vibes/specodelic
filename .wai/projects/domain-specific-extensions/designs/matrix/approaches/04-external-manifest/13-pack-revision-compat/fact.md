The manifest schema can carry a revision field and the CLI can compare it
against the corpus mechanically — but nothing in-corpus checks the claim,
so a stale manifest is trusted config until code says otherwise.
