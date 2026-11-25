---
id: change:kinekino/20261125004857
title: Public surface reads an optional second deny-terms secret
status: active
created: 2026-11-25T00:48:57Z
updated: 2026-11-25T00:48:57Z
rel:
  governs: [.github/workflows/public-surface.yml, xtask/src/deny_terms]
  depends_on: [adr:kinekino/20261116065239, adr:kinekino/20261116062843, adr:kinekino/20261029201516, sot:kinekino/shared-automation-rules]
---

# Public surface reads an optional second deny-terms secret

## Change

This amends point 2 of `adr:kinekino/20261116065239`. The `Public surface` check now reads two lists:

- **`KINEKINO_DENY_TERMS`** is still required. If it is unset or empty, the check fails (fail-closed), as before.
- **`KINEKINO_DENY_TERMS_EXTRA`** is optional. Its terms are added to the main list, and the check uses both. If it is unset or empty, it adds nothing.

Why: a secret's value cannot be read back. New terms therefore cannot be merged into the existing secret without its current value. A second secret holds them instead.

Details:

- **Reports.** They still name only the place and a list number. A hit from the extra list is numbered with an `x` in front (`term #x2`), so the log shows which list matched without naming the term.
- **`--terms-file`.** It still reads a single list, for the optional local pre-push scan.

## Verification

- **Unit tests.** `cargo test -p kinekino-xtask deny_terms`: 12 of 12 pass. The new tests cover four cases:
  - the main list is required;
  - an empty main list fails even when the extra list has terms;
  - an unset or empty extra list adds nothing;
  - the union matches terms from both lists, and extra-list numbers are tagged.
- **CI.** The pull request's own `Public surface` run checks that the main secret still works.

## Migration

None in the repository. The owner side registers the `KINEKINO_DENY_TERMS_EXTRA` repository secret. Dependabot pull requests read Dependabot secrets, so the extra list should be registered there too (ADR point 6). Until it is set, the check behaves as before.
