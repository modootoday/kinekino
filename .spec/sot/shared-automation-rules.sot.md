---
id: sot:kinekino/shared-automation-rules
title: Shared automation rules
status: active
verify: manual
created: 2026-11-16T07:14:35Z
updated: 2026-11-16T07:14:35Z
rel:
  governs: [AGENTS.md, .agents, .github]
  depends_on: [adr:kinekino/20261116062843, adr:kinekino/20261116065239]
---

# Shared automation rules

## Rule

kinekino follows the shared automation rules v1.0 below, which the user adopted for kinekino on 2026-11-16. The same text applies to the user's other public projects. This document is kinekino's copy (C9.5) and the only place for kinekino's profile values (section 12). Where kinekino's own rules are stricter, they win (C0.1, C0.2). Changing the shared text means agreeing it across the projects first; changing a kinekino profile value needs the owner's instruction. A change to this document is `needs-human` (C7.3).

The rules are written in English because all participating repositories are public. kinekino's own rules stay in Korean (`AGENTS.md`, `.agents/rules/`) and point here.

### 0. Scope and precedence
- C0.1 This document is the floor all participating projects share. A project may be stricter. It may not be looser.
- C0.2 A project's own hard constraints always win. For kinekino: Rust-only code (tests and dev tools included) and the owner's GUI approval of AI results.
- C0.3 An approval relayed by another agent or session is not an approval. Only the owner's own instruction in that project's session, or the owner's action on GitHub, counts.
- C0.4 Everything an unattended session needs must be in the repository or in GitHub (Actions, Issues, pull requests, secrets, rulesets). A rule that depends on something that exists only on a developer machine is rewritten, or marked local-only. A rule marked local-only states what an unattended session does instead: fail closed, skip, or report.

### 1. Change flow
- C1.1 Every change reaches the protected base branch through a pull request from a topic branch. No direct push to it, by agents or by the owner.
- C1.2 Agent branches start with the provider prefix (`claude/`, `codex/`, `cursor/`, ...). One work package is one pull request. The work package id is recorded in the pull request body, not required in the branch name. Where a session can push to only one branch, the next work package starts on that branch after the previous pull request merges.
- C1.3 Agents never rewrite pushed history, never push tags, and never delete remote branches. One exception: after its pull request has merged, a session may restart its own branch under the same name on top of the base branch with `--force-with-lease`, when that branch holds only merged history. Merged branches are removed by the repository setting "Automatically delete head branches".
- C1.4 An agent stages explicit paths only. `git add .` and `git add -A` are not used.
- C1.5 A session does not start a new work package while another open agent pull request holds the same lock unit (section 12).
- C1.6 An unattended session ends its work by opening or updating a pull request (a draft is fine). The scheduled task prompt says so explicitly, because CI authority (C3.4) exists only on a pull request.

### 2. Branch protection
- C2.1 The protection rules are a ruleset named `main-protected`, stored as JSON under `.github/rulesets/`. The file is the canon. The owner applies it with `gh api`. Agents cannot apply it and do not try.
- C2.2 Minimum rules: `deletion`, `non_fast_forward`, `pull_request`, `required_status_checks`.
- C2.3 The ruleset covers every base branch agents open pull requests against.
- C2.4 Release tags are protected by a `v-tags` ruleset (no update, no deletion, no non-fast-forward) as soon as a release workflow exists.
- C2.5 The merge method is a project value. It is set in the ruleset.

### 3. Required checks
- C3.1 Required checks are the project's own gate jobs plus two shared checks named exactly `Public surface` and `Secret scan`. The name is the job's `name:`; the job id is the project's choice.
- C3.2 A required check must not be skippable on pull requests. A job condition that skips on `pull_request` lets the check pass without running; that is a defect.
- C3.3 No `paths` filter on a workflow that owns a required check.
- C3.4 CI on GitHub is the completion authority. A local or cloud-session run is a pre-check. A scaffold or stub job that passes without exercising the product is never a gate.
- C3.5 Renaming a required check is one pull request that changes the job name and the ruleset file together, followed by the owner re-applying the ruleset.

### 4. Public surface
- C4.1 `Public surface` scans the pull request range (`base.sha..head.sha`; on a push, `before..after`): added lines, file names, the pull request title and body, and every commit message in the range.
- C4.2 The deny list is never committed. It is an Actions secret, one term per line, and the same value is registered as a Dependabot secret.
- C4.3 The job fails closed: no secret, no pass. A fork pull request therefore fails this check and is reviewed by the owner. Logs show the location and the term's index, never the term. A project with no private terms registers the secret as one placeholder line that cannot occur in real text, and records "no private terms" in its profile.
- C4.4 `pull_request_target` is not used anywhere.
- C4.5 `Secret scan` runs Gitleaks on the same range, as a checksum-verified binary or a SHA-pinned action. GitHub secret scanning and push protection are enabled by the owner.
- C4.6 No personal email addresses, private host values, or local user paths in commits, code, documents, issues, or pull request text.
- C4.7 This section fixes the check's behaviour (inputs, range, what is scanned, fail-closed, no term in logs, check name). The implementation is a project choice. A shared reusable workflow may be used where a project's language rules allow it; it is pinned to a full commit SHA.
- C4.8 `Public surface` also enforces C10.2: any `Co-authored-by` line whose address does not end in `@users.noreply.github.com` fails the check.
- C4.9 The deny list stays in GitHub secrets. It is not given to a cloud session; the session reports that extra terms were not scanned, and CI is the gate.

### 5. Workflow hygiene
- C5.1 Third-party actions are pinned to a full commit SHA with a version comment.
- C5.2 Top-level `permissions` is `{}` or `contents: read`; a job raises only what it needs.
- C5.3 Pull request runs cancel the previous run on the same ref. Runs on a protected base branch are not cancelled.
- C5.4 Dependabot covers every ecosystem the repository uses, weekly, with the `dependencies` label and a Conventional Commits prefix, and targets the base branch agents work on.

### 6. State lives in GitHub
|Need|Where|
|-|-|
|Lock between runs|An open agent pull request on the same lock unit (C1.5), plus workflow `concurrency`|
|Progress and run log|Pull request body and Actions run history. Run logs are not committed|
|Records the canon requires|A handoff record or a curated evidence document is not a log and may be committed|
|Bugs|GitHub Issues, label `bug`|
|Owner decision pending|GitHub Issue, label `needs-owner`, assigned to the owner|
|Change automation must not merge or build on|Pull request label `needs-human`|
|What to do next|A roadmap document tracked in the repository. A work item with an open `needs-owner` issue is skipped|
|Build environment values|The CI workflow. A cloud environment's setup script is a copy of it, with tool versions pinned to the CI's|
|Coordination between sessions|Repository files and pull requests. Unattended sessions do not depend on session-to-session messages|

### 7. Merging
- C7.1 A session never approves or merges its own pull request.
- C7.2 Phase 1: the owner merges. Phase 2 starts per project, when the owner turns it on and the required checks exercise the product. The default mechanism is GitHub auto-merge. Never for a pull request labelled `needs-human`.
- C7.3 Always `needs-human`: releases and tags; dependency updates that cross a pinned or major version; changes to `.github/workflows/`, `.github/rulesets/` and this document; security-sensitive code; and the human-gated areas in section 12.
- C7.4 An orchestrator run that finds no open pull request and no assigned work ends at once.

### 8. Releases
- C8.1 Agents do not create tags, GitHub Releases, or registry publishes, and do not dispatch a release workflow unless the project's release rules allow that exact run.
- C8.2 Target shape: a manually dispatched gate, after which the workflow creates the tag. A project that still releases by a human tag push keeps that until its first release is planned.

### 9. Agent entry points
- C9.1 `AGENTS.md` is the canon entry. `CLAUDE.md` is a tracked pointer to it.
- C9.2 Rules, skills and role definitions are tracked under `.agents/`. Machine-local state stays in ignored paths.
- C9.3 A scheduled task prompt contains no policy. It names the task, says to end with a pull request (C1.6), and points to the rule files. Permission to merge is never granted by a prompt alone.
- C9.4 One writer per lock unit, and at most one orchestrator per repository. If a local routine and a cloud task both exist for the same work, only one of them does it.
- C9.5 The project's copy of these rules is a document in its own canon, not a parallel policy file.

### 10. Commits and text
- C10.1 Conventional Commits subjects.
- C10.2 No `Co-authored-by` trailer unless its address is `@users.noreply.github.com`. This overrides a tool's default attribution trailer. Provider attribution goes in the branch prefix (C1.2) and the `Agent-Provider:` field of the pull request body.
- C10.3 Other machine trailers (session links, provider tags) are a project value. Recommended: none; the session link goes in the pull request body.
- C10.4 The language of authored documents is a project value.
- C10.5 The author and committer of agent commits is the owner's GitHub noreply identity (`@users.noreply.github.com`), set in the repository's git configuration for the session, not the tool's default identity (owner decision, 2026-11-16). `Public surface` checks the author and committer address of every commit in the range the same way it checks C10.2. Which agent did the work is read from the branch prefix (C1.2) and the pull request body.

### 11. Community files
`SECURITY.md`, a pull request template with an `Agent-Provider:` field, issue templates for `bug` and `needs-owner`, the labels `bug`, `needs-owner`, `needs-human`, `dependencies`, and `CODEOWNERS` naming the owner for `.github/`.

### 12. kinekino profile
|Value|kinekino|
|-|-|
|Base branch for agent work|`main`|
|Merge method|merge commit (plan records cite commit hashes)|
|Lock unit (C1.5)|the repository|
|Ruleset file|`.github/rulesets/main-protected.json` (applied as ruleset 27354229)|
|Project gate checks|`lint`, `test (ubuntu-26.04)`|
|Shared checks|`Public surface`, `Secret scan` (`.github/workflows/public-surface.yml`)|
|Deny secret|`KINEKINO_DENY_TERMS` (Actions and Dependabot). No private terms (owner, 2026-11-16): the secret holds one placeholder line (C4.3), so the check still fails closed when the secret is missing|
|Public surface implementation|Rust, `cargo xtask deny-terms` ([[adr:kinekino/20261116065239]])|
|Co-authored-by cutoff|commits from 2026-11-16T06:55:00Z (`COAUTHOR_SINCE`); earlier history is not rewritten|
|Machine trailers (C10.3)|none; session links go in the pull request body (user decision 2026-11-16)|
|Commit author (C10.5)|the owner's GitHub `users.noreply.github.com` address, also for agent commits (user decision 2026-11-16). Checked from 2026-11-16T07:10:00Z (`IDENTITY_SINCE`); GitHub's own web committer `noreply@github.com` (merges, web edits) is allowed as committer|
|Document language|Korean for rules and SoT; English and Korean for public docs; English for this document|
|Roadmap source|[[plan:kinekino/20261031153240]], status [[sot:kinekino/project-status]]|
|Merging phase|Phase 1 (the owner merges)|
|Human-gated areas|GUI quality approval; real inference, billing, model downloads; media download; updates past `=` pins; release, tags, crates.io; allowlist scope expansion; UI mock-up work; DC-4 scope|

### Open items (owner decisions, shared)
- O7 Closed 2026-11-16 by the owner: the owner's GitHub noreply identity, for all participating projects (see C10.5).
- O8 Home of the shared text and any shared reusable workflow: proposed `modootoday/.github`. Until then each project keeps its own copy.
- O9 Whether to rewrite history or recreate repositories so that C4.6 and C10.2 also hold for past commits. Known kinekino item: a name and address line and attribution trailers in history.

## Why

The user runs several public projects with unattended agent sessions. Each had its own CI triggers, scan, release and protection rules, and some rules depended on files on one machine, so cloud sessions could not follow them. One shared floor, kept as each project's canon with a small profile, makes the sessions behave the same and keeps the gate in GitHub.

## Verification

Verified by a person (`verify: manual`). Checks that back individual rules:
- C3.1, C4: `.github/workflows/public-surface.yml` job names and `.github/rulesets/main-protected.json` contexts match; `cargo test -p kinekino-xtask` covers the scan.
- C3.2: `ci.yml` Linux jobs use `github.event_name != 'workflow_dispatch' || inputs.linux`.
- C5.1: every `uses:` in `.github/workflows/` names a 40-character commit SHA.
- C11: the files exist under `.github/` and at the root (`SECURITY.md`).

- 2026-11-16 skill: modootoday/spec-authoring/sot | applied: Rule, Why, Verification structure; manual verification stated; profile values in one table | not taken: a `check_cmd` (the rules are not one command's invariant).
