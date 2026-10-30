# Source provenance

- Meari upstream: https://github.com/modootoday/meari
- Original recorded upstream/local base: `0844f6c24caf224617ca8abbb2dd75695a971b99`.
- Meari is consumed from crates.io (`meari`, `meari-native`, `meari-dom`, `meari-paint`, `meari-shell`,
  `meari-html`, `meari-platform` 0.1.0; MIT OR Apache-2.0). `meari-dom`/`meari-shell` are imported under the
  `blitz-dom`/`blitz-shell` names. No Meari source is vendored in this repository.
- Meari's MIT/Apache license and NOTICE files ship inside its published crates.
- Fonts: Noto Sans, Noto Sans KR and Noto Sans JP, SIL Open Font License 1.1. They are **not embedded**
  in any binary and not stored in this repository. `kinekino fonts fetch` downloads them from
  `google/fonts` at the pinned revision `6157dfaa640c` (`meari::domain::design::fonts::REVISION`),
  verifies length and SHA-256 of every file against the manifest, and caches them under
  `runtime/fonts/noto/6157dfaa640c/` (or `$MEARI_FONT_CACHE`). Each font is downloaded together with its
  OFL notice, which ships beside it and is reproduced in every font-delivery provenance record:

  | File | SHA-256 | OFL notice | Notice SHA-256 |
  |---|---|---|---|
  | `NotoSans[wdth,wght].ttf` | `bfb7bb691513f12e734dc346c03a03f784912432d7e3fa8e56efcf906fe86b3d` | `notosans-OFL.txt` | `cee9892f9f0cc8fe882c9e9537ee6a89621d86ee7ceaf70b02e2b2b1c25c061a` |
  | `NotoSansKR[wght].ttf` | `194018e6b2b293a7964f037b25c0249ce1418bc9ab3c971060a03aa57861e252` | `notosanskr-OFL.txt` | `1c05c68c34f9708415aada51f17e1b0092d2cea709bf4a94cd38114f9e73d7d9` |
  | `NotoSansJP[wght].ttf` | `c2f3b4d463500a2ddcd3849cded1fceeb9fd6d1c32e6cbecd568453ba50fc68f` | `notosansjp-OFL.txt` | `1c05c68c34f9708415aada51f17e1b0092d2cea709bf4a94cd38114f9e73d7d9` |

  Copyright 2022 The Noto Project Authors (Noto Sans); Copyright 2014-2021 Adobe, Reserved Font Name
  'Source' (Noto Sans KR/JP).
  The OFL permits use, bundling and redistribution with software; the fonts may not be sold by
  themselves, and modified versions must not use the Reserved Font Names. The earlier Pretendard
  assets were removed.
- Demo/design assets are carried from the project-selected source/design directories; this is a private handoff,
  not a new claim that all assets are generally redistributable.
- Cargo dependency versions/checksums remain in Cargo.lock. No global Cargo/Rustup configuration or credentials are included.

See `docs/continuation/selected-source-sha256.json` for the selected source file hashes before packaging-only
configuration/documentation transformations. Do not treat a snapshot or manifest as evidence of a successful
build or AI quality approval on the receiving machine.
