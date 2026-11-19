---
id: change:kinekino/20261119195313
title: Add the playpilot guard hook to project settings
status: active
created: 2026-11-19T19:53:13Z
updated: 2026-11-19T19:53:13Z
rel:
  governs: [.claude/settings.json]
  depends_on: [adr:kinekino/20261119194255, change:kinekino/20261116135248, change:kinekino/20261119194431, sot:kinekino/shared-automation-rules]
---

# Add the playpilot guard hook to project settings

## Change

This is stage 2 of `adr:kinekino/20261119194255` (H6).

- **`.claude/settings.json` (new).** A `PreToolUse` hook for `Bash|PowerShell` runs this command, with a 240 s timeout:

  ```sh
  sh -c 'command -v playpilot >/dev/null 2>&1 || cargo install --locked --quiet --git https://github.com/modootoday/playpilot --rev 0ddc0e1dfd1bac378901817a5fb106935fd5fba1 playpilot >&2 || { echo "playpilot CLI is missing and could not be installed" >&2; exit 2; }; exec playpilot guard'
  ```

  The settings also set `env.PLAYPILOT_PROTECTED_BRANCHES=main`.
- **`.gitignore`.** It now admits `/.claude/settings.json`. The rest of `.claude/` stays denied, apart from the existing links.
- **Rules.** `AGENTS.md` and `.agents/rules/git-and-release.md` (H6) describe the hook and how commands must be written for it.

## Verification

All results below were measured on x86_64 Linux. The hook input is `{"tool_name":…,"tool_input":{"command":…}}` on stdin, and the command is the exact string from `.claude/settings.json`.

| Input | Exit | Guard message |
|---|---|---|
| `git push origin main` | 2 | `main` is protected |
| `git push -u origin claude/vibrant-wright-5zq0qi-guard` | 0 | |
| `PowerShell` tool, `git push origin main` | 2 | `main` is protected |
| `git push -q` (no branch) | 2 | name the branch explicitly |
| `git push --force origin claude/x` | 2 | not allowed |
| `git push --force-with-lease=claude/x:abcdef1 origin claude/x` | 0 | |
| `git add .` / `git add AGENTS.md` | 2 / 0 | |
| `gh pr merge 58` / `gh pr merge 58 --auto --merge` | 2 / 0 | |
| `gh api -X GET search/issues -f q=…` | 0 | |
| `git tag v1` / `git tag -l` | 2 / 0 | |
| `cargo test -q` | 0 | |
| No `playpilot` and no `cargo` on `PATH` | 2 | "playpilot CLI is missing and could not be installed" |

- **First run without the CLI.** The pinned commit installed into `~/.cargo/bin` in about 5 s, and the call went through.
- **This session after the hook loaded.** Ordinary git, spec and cargo commands ran, including the rebase and the lease push of this branch. Three commands were refused, and the rules now say how to avoid each:
  - **A Python heredoc with triple quotes.** The guard could not split it into shell words. The workaround is to write the script to a file and run it.
  - **`git push --force-with-lease=<branch>:$old …`.** The guard does not expand variables, so `$old` is not a SHA. The workaround is to write the SHA out.
  - **`git push … <branch> 2>&1 | …`.** The guard read `2>` as a lease target. The workaround is to push with no redirection or pipe.
- **Windows.** A run there is pending, with the command on the `needs-windows-run` issue.

## Migration
