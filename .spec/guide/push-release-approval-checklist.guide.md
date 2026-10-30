---
id: guide:kinekino/push-release-approval-checklist
title: Push and release approval checklist (ADR 0011 rewrite)
status: active
created: 2026-10-30
updated: 2026-10-30
---
# Push / release approval checklist (2026-10-30)

Nothing below has been run. Every step is an owner decision; no remote, push, tag, release,
GitHub environment or secret exists yet. Run each block only after ticking its box.

## 0. Prerequisites (owner)

- [ ] Choose the public repo: the rewritten mirror `/workspace/kinekino-clean` (0 forbidden words in
      every blob and message; build, 1440 tests, lint 0 new, r-src 0, specfold 0 errors), not
      `/workspace/kinekino` (original history kept on purpose).
- [x] Merge state: `main-next` merged into `main` with `--no-ff` (e39dbad); `main-next` deleted; P3/P4 slices
      merged after it (afb815a). The mirror was re-written from `main` afb815a (same `strings.txt`,
      same commit-map step): 155 commits; plugin history clone
      `/workspace/kinekino-source-catalog-history` 109 commits / 994 paths.
- [ ] Backups kept: `/workspace/kinekino-pre-adr0011-20261030.bundle` and
      `/workspace/kinekino-pre-adr0011-20261030-e39dbad.bundle`,
      `/workspace/kinekino-pre-adr0011-20261030-afb815a.bundle` (`git bundle verify`).
- [ ] Old→new hashes: `/workspace/kinekino-clean-commit-map.txt`.

## 1. kinekino (public)

```sh
cd /workspace/kinekino-clean
git remote add origin git@github.com:<owner>/kinekino.git   # approval: remote add
git push -u origin main                                      # approval: push
```

- [ ] Branch protection on `main`; CI (`ci.yml`) green on the pushed commit.
- [ ] Do not push `feat/site-mockup` / `spike/embed` unless wanted.

## 2. kinekino-source-catalog (private)

```sh
cd /workspace/kinekino-source-catalog
git remote add origin git@github.com:<owner>/kinekino-source-catalog.git  # private repo
git push -u origin master
```

- [ ] Optional: removed original history at `/workspace/kinekino-source-catalog-history`
      (filter-repo `--paths-from-file` of the 994 paths that ever held source-site words; branch
      `catalog` = the plugin repo). Push it only to the private repo, e.g. as `history/original`.

## 3. GitHub Pages and DNS

- [ ] Pages source: `main` / `docs` (no Actions), see `guide:kinekino/site-github-pages`.
- [ ] Custom domain: add `docs/CNAME` with the domain, then DNS: apex `A` 185.199.108.153,
      185.199.109.153, 185.199.110.153, 185.199.111.153 (`AAAA` 2606:50c0:8000::153 … 8003::153),
      `www` `CNAME <owner>.github.io`. Verify the domain in account settings; enable HTTPS.

## 4. Release (ADR 0010, `.github/RELEASING.md`)

- [ ] Environment with required reviewers and tag policy `v*`, on the publish job only.
- [ ] Signing keys: none today (artifacts unsigned). Decide Windows Authenticode certificate,
      macOS Developer ID + notarization, and checksum/minisign key; add the secret names listed in
      ADR 0010 only after the keys exist.
- [ ] Bump `Cargo.toml` version, then `git tag v<version> && git push origin v<version>` (the tag push
      is the release decision; the workflow checks it matches `Cargo.toml`).
