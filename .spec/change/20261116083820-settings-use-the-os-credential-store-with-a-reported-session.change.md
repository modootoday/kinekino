---
id: change:kinekino/20261116083820
title: Settings use the OS credential store with a reported session fallback
status: active
created: 2026-11-16T08:38:20Z
updated: 2026-11-16T08:38:20Z
rel:
  depends_on: [adr:kinekino/20261106022549, sot:kinekino/project-status, plan:kinekino/20261031153240, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261109121745, guide:kinekino/ignored-tests, guide:kinekino/site-github-pages, plan:kinekino/20261030013900, sot:kinekino/conventions]
---

# Settings use the OS credential store with a reported session fallback

## Change

`kinekino settings` (CLI and GUI) now selects its credential store in `crates/kinekino-app/src/settings/credentials.rs`. It opens the OS store first, which on Linux is the Secret Service through `kinekino_platform::credential::open_os_store`. It falls back to the in-process session store only when that returns `Unavailable`, and keeps the reason. Other errors are returned, not hidden. `kinekino settings credentials` prints `{"backing","persistent","reason"?}`. `KINEKINO_CREDENTIALS=os` refuses the fallback and `=session` never opens the OS store. The unused session store on the bootstrap `App` was removed, so `platform …` commands do not touch D-Bus. Docs `docs/settings.md` and `docs/features/inference.md` describe the new behaviour.

## Migration

None for users. On Linux with an unlocked keyring, a token saved from settings now persists across runs. Tests that drive `settings::run` set `KINEKINO_CREDENTIALS=session` so they never use a developer's keyring.
