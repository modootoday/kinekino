# Releasing

`.github/workflows/release.yml` (ADR 0010) follows specfold's release shape. A `v<version>` tag
builds the binaries into a draft release and marks it published once all five archives are on
it. By hand (workflow_dispatch, mode `dry-run`) it
builds and packages every target and deletes its draft.

1. Bump `version` under `[workspace.package]` in the root `Cargo.toml`, run the usual checks
   (`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test`,
   `cargo xtask lint-structure`, `cargo xtask spec-check`) and commit.
2. Optionally run the CI workflow and the Release workflow (`dry-run`) by hand.
3. Push the tag `v<version>`; the workflow fails if it does not match `Cargo.toml`. Pushing the
   tag is the release decision, as in specfold.
4. Confirm the release page lists five archives and their `.sha256` files.

## Archives

`kinekino-<target>.tar.gz` (`.zip` on Windows) with `kinekino`, `player`, `kinekino-vad-worker`,
`LICENSE` (MIT), `README.md`, `THIRD_PARTY.md`, and `<archive>.sha256`. Targets: `x86_64-unknown-linux-gnu`,
`aarch64-unknown-linux-gnu` (glibc 2.35), `x86_64-apple-darwin`, `aarch64-apple-darwin`,
`x86_64-pc-windows-msvc`. Unsigned until signing lands (ADR 0010).

## One-time setup

- GitHub environments: none today. As in specfold, add one per package registry when one is
  added (for example `crates`, `winget`, `homebrew`), named after the registry, with required
  reviewers and the tag policy `v*`, set only on the job that publishes there and only for tag
  pushes. Publish with Trusted Publishing (OIDC) where the registry supports it.
- Pages: Settings → Pages → Deploy from a branch → `main` `/docs` (no workflow, as specfold).
- No secrets today; signing secret names are listed in ADR 0010.
