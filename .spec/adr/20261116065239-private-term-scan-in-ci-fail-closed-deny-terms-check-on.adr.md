---
id: adr:kinekino/20261116065239
title: "Private term scan in CI: fail-closed deny-terms check on every pull request"
status: active
created: 2026-11-16T06:52:39Z
updated: 2026-11-16T07:25:00Z
rel:
  governs: [.github/workflows/public-surface.yml, .github/workflows/ci.yml, xtask/src/deny_terms]
  depends_on: [adr:kinekino/20261116062843]
---

# Private term scan in CI: fail-closed deny-terms check on every pull request

## Context

- Commits and files in this public repository must not carry private traces: names, paths or project terms the user keeps private. Until now the only check was a local scan before a routine push, against a term list in an untracked file (`.agents/rules/git-and-release.md`, "push 직전 비공개 용어 스캔").
- With the pull request flow ([[adr:kinekino/20261116062843]]) work also comes from cloud sessions and Dependabot, which do not have that local file. A check that lives only on one machine no longer covers what reaches `main`.
- The term list itself is private, so it cannot be committed, and CI logs of a public repository are public.

## Decision

1. `cargo xtask deny-terms --range BASE..HEAD [--text-env NAME]... [--coauthor-since TIME]` (module `xtask/src/deny_terms/`) checks three things in the range: commit messages (`git log BASE..HEAD`), added diff lines and changed file names (`git diff BASE...HEAD`, no rename detection). Matching is case-insensitive fixed-string, like `grep -i -F`. One term per line; blank lines and `#` lines are skipped.
2. The list comes from the `KINEKINO_DENY_TERMS` environment variable or `--terms-file`. A missing or empty list is an error (fail-closed): no list means no pass.
3. Reports give the place (commit, `path:line`, file name) and the term's line number in the list, never the term.
4. The workflow `.github/workflows/public-surface.yml` runs it as the check `Public surface` (the job's `name:`; the name is shared with the user's other projects, shared automation rules C3.1). It runs on `pull_request` (`opened`, `synchronize`, `reopened`, `edited`) with `base.sha..head.sha` and on `push` to `main` with `before..after` (checkout `fetch-depth: 0`). It also scans the pull request title and body (`--text-env`), so editing them re-runs only this check, not the full CI. It reads the term list from the repository secret `KINEKINO_DENY_TERMS`.
5. `Public surface` joins the required checks in `.github/rulesets/main-protected.json`. The check name and the ruleset change in the same pull request, and the owner re-applies the ruleset afterwards (shared rules C3.5).
6. No `pull_request_target`: pull requests from forks get no secrets, fail this check and need a person. Dependabot pull requests read Dependabot secrets, so the same list must also be registered there.
7. The same check enforces the trailer rule (user decision 2026-11-16, shared rules C10.2/C4.8): a `Co-authored-by` line whose address does not end in `@users.noreply.github.com` fails it. It applies to commits made at or after `COAUTHOR_SINCE` (2026-11-16T06:55:00Z) in the workflow; earlier history is not rewritten. The address is never printed.
8. The same check enforces the commit identity rule (user decision 2026-11-16, shared rules C10.5): from `IDENTITY_SINCE` (2026-11-16T07:10:00Z) on, a commit whose author or committer address is not `@users.noreply.github.com` fails it. GitHub's own web committer `noreply@github.com` (merge commits, web edits) is allowed as committer. Addresses are never printed.
9. The local pre-push scan stays as an earlier, optional check; the CI check is the one that gates `main`.

## Consequences

- A person must register the secret twice (Actions and Dependabot) and re-import the ruleset. Until then every pull request fails `deny-terms`, which is intended.
- A hit blocks the merge. Fixing it needs a new commit; a hit in a commit message needs the branch rewritten, which agents may do only on their own unmerged topic branch.
- Binary file contents are not checked (as in the local rule); file names are.
- False positives (a term that is also a common word) are fixed by narrowing the list, not by skipping the check.
- The job builds only `kinekino-xtask` (no dependencies), about 1–2 minutes with the toolchain install.
