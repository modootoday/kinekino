---
id: change:kinekino/20261116135248
title: needs-human pull requests auto-merge after measurement
status: active
created: 2026-11-16T13:52:48Z
updated: 2026-11-16T13:52:48Z
rel:
  depends_on: [sot:kinekino/shared-automation-rules, adr:kinekino/20261029201516, adr:kinekino/20261116062843]
  governs: [AGENTS.md, .agents/rules/routine-operations.md, .agents/rules/git-and-release.md, .github/labels.json]
---

# needs-human pull requests auto-merge after measurement

## Change

Owner decision 2026-11-16: "needs-human pull requests also proceed after measurement."

A `needs-human` pull request now gets GitHub auto-merge once it is measured:
- the Linux required checks (`lint`, `Public surface`, `test (ubuntu-26.04)`) pass;
- when the change reaches the build of a Windows gate crate (the crates `.github/scripts/windows-gate.sh` checks), `cargo check --target x86_64-pc-windows-msvc` also passes.

The commands and results go in the pull request.

Two exceptions stay with the owner:
- changes that remove or weaken a required check;
- releases and tags.

Applying settings files on the owner's computer (shared rule C2.7) is unchanged.

Verification that needs a real Windows run is left on the pull request or issue with the new label `needs-windows-run` and the exact command. The owner's Windows OSS Desktop session runs it.

Updated together:
- `AGENTS.md`
- `.agents/rules/routine-operations.md`
- `.agents/rules/git-and-release.md`
- the profile of [[sot:kinekino/shared-automation-rules]] (merging phase and label rows)
- `.github/labels.json`: adds `needs-windows-run` and rewords `needs-human`

The label list is a settings file, so the owner applies it after merge (C2.7).

## Migration
