---
id: change:kinekino/20261116071526
title: "Adopt the shared automation rules: Secret scan, Dependabot labels, Agent-Provider field"
status: active
created: 2026-11-16T07:15:26Z
updated: 2026-11-16T07:15:26Z
rel:
  depends_on: [adr:kinekino/20261029201516, adr:kinekino/20261116062843, adr:kinekino/20261116065239, sot:kinekino/shared-automation-rules]
---

# Adopt the shared automation rules: Secret scan, Dependabot labels, Agent-Provider field

## Change

kinekino adopts the shared automation rules v1.0 (user decision 2026-11-16) and keeps its copy and profile in [[sot:kinekino/shared-automation-rules]], linked from `AGENTS.md`.

- `public-surface.yml` gains the job `Secret scan` (C4.5): gitleaks on the change's range, from a pinned release (`GITLEAKS_VERSION`) checked against the release checksums file by `.github/scripts/install-gitleaks.sh`, with findings redacted. `.github/rulesets/main-protected.json` requires `Secret scan`.
- `Public surface` also checks every commit's author and committer address from `IDENTITY_SINCE` (2026-11-16T07:10:00Z) on (C10.5; `cargo xtask deny-terms --identity-since`). GitHub's web committer `noreply@github.com` is allowed as committer.
- `.github/dependabot.yml` adds the `dependencies` label and the prefixes `chore(deps)` (cargo) and `ci(deps)` (actions) (C5.4).
- The pull request template starts with an `Agent-Provider:` field (C10.2, C11).

## Migration

After merging, the owner re-applies ruleset 27354229 from the file (`gh api -X PUT repos/modootoday/kinekino/rulesets/27354229 --input .github/rulesets/main-protected.json`). Gitleaks has not been run on this history from the session (the release host is not reachable here); the first pull request run shows whether old history in the range needs an allowlist entry.
