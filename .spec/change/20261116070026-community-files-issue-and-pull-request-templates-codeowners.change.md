---
id: change:kinekino/20261116070026
title: "Community files: issue and pull request templates, CODEOWNERS, SECURITY.md"
status: active
created: 2026-11-16T07:00:26Z
updated: 2026-11-16T07:00:26Z
rel:
  depends_on: [adr:kinekino/20261029201516, adr:kinekino/20261116062843]
---

# Community files: issue and pull request templates, CODEOWNERS, SECURITY.md

## Change

Adds the community files the shared automation rules ask for (C11), so issues and pull requests carry the state that local files held before:

- `.github/ISSUE_TEMPLATE/bug.yml` (label `bug`; severity is a form field, not a label) and `needs-owner.yml` (label `needs-owner`: work that waits for the owner). Blank issues are off; security reports go to a private advisory.
- `.github/pull_request_template.md`: summary, the gate commands, what was not checked, and when to add `needs-human`.
- `.github/CODEOWNERS`: the owner owns `.github/`.
- `SECURITY.md`: private reporting through GitHub security advisories. `.gitignore` allows it at the root.

## Migration

A person creates the labels (agent sessions have no label API) and turns on private vulnerability reporting (Settings → Code security):

```
gh label create needs-owner --color FBCA04 --description "Waits for an owner decision; automation skips it"
gh label create needs-human --color D93F0B --description "Automation must not merge or build on this"
```
