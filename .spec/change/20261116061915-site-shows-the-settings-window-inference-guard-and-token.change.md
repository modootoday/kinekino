---
id: change:kinekino/20261116061915
title: "Site shows the settings window, inference guard and token storage as they are"
status: active
created: 2026-11-16T06:19:15Z
updated: 2026-11-16T06:19:15Z
rel:
  depends_on: [guide:kinekino/site-github-pages]
---

# Site shows the settings window, inference guard and token storage as they are

## Change

The site pages (English and Korean: settings, features, features/inference, index) still described the state before P6 and P1b were merged. Checked against the code on 2026-11-16:

- The settings window uses the pinned Noto Sans faces (`kinekino-settings-ui/src/styles.rs`), not Pretendard, which was removed. It was checked on Linux X11 (M2-V5b); Windows and macOS were not checked.
- The inference port, consent and cost guard exist (`kinekino-inference`: `consent.rs`, `budget.rs`, mock server) and are tested against the mock only. No real model has been called (P7 held).
- The app wires `SessionCredentialStore` (`kinekino-app/src/lib.rs`, `settings/mod.rs`), so a token lasts for the session only. The pages said tokens go to the OS credential store; the Linux Secret Service adapter exists but is not wired.

The settings page, settings area, inference area, inference consent and cost guard, and Meari UI now show partial, with these limits in the notes.

## Migration

None. Documentation only; no command, setting or schema changed.
