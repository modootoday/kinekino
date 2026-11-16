Agent-Provider: <!-- Claude Code, Codex, ... or none for a person's own change -->

## Summary

<!-- What changes and why. Link the spec documents (adr:, plan:, sot:) it follows. -->

## Verification

<!-- Commands run and their results. Say what was not run. -->
- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] `cargo xtask lint-structure`
- [ ] `cargo xtask spec-check`
- [ ] `cargo test --workspace`

## Not checked

<!-- What still needs a person: GUI quality approval, other OSes, real hardware. -->

## Human-gated

<!-- Add the `needs-human` label if this touches .github/workflows, .github/rulesets,
     releases or tags, dependency updates past a pinned (=) or major version,
     security-sensitive code, or anything else that waits for the owner. -->
