---
id: adr:kinekino/20261119194255
title: Commit history rules H1-H7 (playpilot) for kinekino
status: active
created: 2026-11-19T19:42:55Z
updated: 2026-11-19T19:42:55Z
rel:
  governs: [.agents/rules/git-and-release.md]
  depends_on: [adr:kinekino/20261116062843]
---

# Commit history rules H1-H7 (playpilot) for kinekino

## Context

The owner had playpilot introduced in every project session ("각 세션에 playpilot 도입 지시문 전달", 2026-11-19, relayed by the design session). playpilot is public at `modootoday/playpilot`, and its canonical rule is `.agents/rules/commit-history.md` (H1–H7). Its goal is that the base branch reads as a list of finished, reviewable changes: every commit on it builds, passes the gates, and says what changed and why.

kinekino merges with merge commits (ADR 20261116062843), and its plan and change records cite topic-branch commit hashes. Before this decision:

- agents caught a topic branch up by merging `main` into it;
- force pushes were forbidden outright, apart from the restart allowed after a branch had merged (C1.3).

## Decision

kinekino adopts H1–H7 with these project values.

- **H1 merge method.** kinekino keeps **merge commit**, because records cite topic-branch hashes. Squash is not enabled. The ruleset is unchanged.
- **H2.** Applies only to squash repositories, so it does not apply here. In a merge-commit repository, every commit on a topic branch is a commit on `main`. Every commit subject must therefore be a Conventional Commits subject, `type(scope): summary`, imperative and at most 72 characters.
- **H3 topic branches at merge time.**
  - One work package per pull request, and one logical change per commit.
  - No merge commits in the branch. Catch up with `main` by rebasing, and use "Update branch" only with its rebase option.
  - No `WIP`, `fixup!`, `squash!`, `amend!` or empty commits.
  - Generated churn (lockfile changes) goes in its own `chore(deps)` or `build` commit.
  - No tool or session trailers. Author and committer are the owner's noreply identity (C10.2, C10.3, C10.5).
- **H4 rewriting.**
  - An agent may rewrite **its own unmerged topic branch** (`claude/…` and other provider prefixes, only its own commits) to meet H3. It pushes the result only with `--force-with-lease=<branch>:<expected-sha>`.
  - Plain `--force`, a `+refspec`, rewriting anyone else's branch, and rewriting `main`, tags, or merged or shared history stay forbidden.
  - This narrows common rule C1.3 for kinekino. After a review comment, prefer a new commit. When a branch is rewritten to meet H3, the pull request says so.
- **H5.** A merged change is undone with `git revert` in a new pull request titled `revert: <original subject>`. Merged head branches are deleted by the repository setting. An agent branch that has had no pull request for 14 days is reported in an issue, and the owner deletes it.
- **H6 enforcement, first stage.**
  - CI gains a `History` job. It installs playpilot at a pinned commit and runs `playpilot history-check --method merge --range "$BASE..$HEAD"` on pull requests.
  - The job is **not** a required check yet. Making it required changes the ruleset, which is the owner's step.
  - The `playpilot guard` hook (`.claude/settings.json`) is not added yet. It comes in stage 2, once the cloud environment has the playpilot CLI.
- **H7 auto-merge.**
  - The agent turns auto-merge (merge commit) on right after opening a pull request. GitHub merges only when the required checks pass.
  - To add commits to a pull request that has auto-merge on, the agent turns auto-merge off, pushes, then turns it on again.
  - The `needs-human` measurement rule and its two owner exceptions are unchanged: removing or weakening a required check, and releases and tags.

## Consequences

- `AGENTS.md` and `.agents/rules/git-and-release.md` change their "no force push" lines to the narrower H4 wording. `routine-operations.md` is unchanged.
- Commits merged before this decision are not rewritten. Many of them have subjects that are not Conventional (for example "Record build cache survey and prune decision"). `History` runs only on a pull request's own range, so it does not flag them.
- Until `History` is required, a pull request whose commits break H3 can still merge. Agents keep to H3 by hand.
- Stage 2: install the playpilot CLI in the cloud environment, add the guard hook, then let the owner make `History` required.
