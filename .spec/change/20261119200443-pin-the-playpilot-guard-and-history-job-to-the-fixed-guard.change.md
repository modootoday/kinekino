---
id: change:kinekino/20261119200443
title: Pin the playpilot guard and History job to the fixed guard parser
status: active
created: 2026-11-19T20:04:43Z
updated: 2026-11-19T20:04:43Z
rel:
  governs: [.claude/settings.json, .github/workflows/ci.yml, .agents/rules/git-and-release.md]
  depends_on: [adr:kinekino/20261119194255, change:kinekino/20261119195313, change:kinekino/20261119194431, adr:kinekino/20261029201516, adr:kinekino/20261116062843, adr:kinekino/20261116065239, change:kinekino/20261116135248, evidence:kinekino/20261109092655, evidence:kinekino/20261117193255, sot:kinekino/shared-automation-rules]
---

# Pin the playpilot guard and History job to the fixed guard parser

## Change

This is a follow-up to `change:kinekino/20261119195313`. It moves the pinned playpilot commit from `0ddc0e1dfd1bac378901817a5fb106935fd5fba1` to `4ed7daee4b5bfa4a3b0a89e7f5e2281690efec48`. The new commit includes playpilot's guard fix, which reads redirections, heredocs and lease variables correctly.

- **`.claude/settings.json`.** The hook installs the new commit. The rest of the command is unchanged.
- **`.github/workflows/ci.yml`.** The `History` job installs the new commit.
- **`.agents/rules/git-and-release.md` (H6).**
  - The three workarounds are replaced by what the guard now allows.
  - A new line says how to replace an existing install, because the hook installs only when the CLI is missing.

## Verification

Measured on x86_64 Linux. Each command was sent as hook JSON to `playpilot guard` with `PLAYPILOT_PROTECTED_BRANCHES=main`, once for each pinned commit.

| Command | Expected | `4ed7dae` | `0ddc0e1` |
|---|---|---|---|
| `git push origin main` | 2 | 2 | 2 |
| `git push -u origin claude/x` | 0 | 0 | 0 |
| `git push -u origin claude/x 2>&1 \| tail -1` | 0 | 0 | 0 |
| `git push origin main 2>&1` | 2 | 2 | 2 |
| `git push --force-with-lease=claude/x:$old origin claude/x` | 0 | 0 | **2** |
| `git push --force-with-lease=main:$old origin main` | 2 | 2 | 2 |
| `git push --force-with-lease=feature/x:abcdef1 origin feature/x` | 2 | 2 | 2 |
| `python3 - <<'EOF'` with a triple-quoted body | 0 | 0 | 0 |
| `bash <<'EOF'` with `git push origin main` in the body | 2 | 2 | 2 |
| `cat <<EOF > n.md` with `git push origin main` in the body | 0 | 0 | **2** |
| `git push --force origin claude/x` | 2 | 2 | 2 |
| `git add .` | 2 | 2 | 2 |
| `gh pr merge 59` / `gh pr merge 59 --auto --merge` | 2 / 0 | 2 / 0 | 2 / 0 |
| `git tag v1` | 2 | 2 | 2 |
| `cargo test -q` | 0 | 0 | 0 |

- **Install time.** `cargo install --locked` of the new commit took about 5 s.
- **Diff reviewed.** From `0ddc0e1` to `4ed7dae`, only `policy.rs`, its tests and a CI download script changed.


## Migration

A machine that already has `playpilot` keeps its old build, because the hook installs only when the CLI is missing. To upgrade it, run:

```sh
cargo install --locked --force --git https://github.com/modootoday/playpilot --rev 4ed7daee4b5bfa4a3b0a89e7f5e2281690efec48 playpilot
```
