---
title: "Component styleguide"
description: "Internal catalog of kinekino site components. Not linked, not in the sitemap."
ref: styleguide
dev: true
twin: false
noindex: true
sitemap: false
hero: { c: hero, visual: player-mock, scene: playback }
sections:
  - { c: features, cols: 3 }
  - { c: split, visual: player-mock, scene: timeline }
  - { c: split, visual: player-mock, scene: review }
  - { c: split, visual: player-mock, scene: settings }
  - { c: split, visual: player-mock, scene: agent }
  - { c: split }
  - { c: prose }
  - { c: platform-list }
  - { c: timeline }
  - { c: cta, mascot: true }
  - { c: prose }
---

This page shows every registered component with plain Markdown. Each block below is chosen by one `sections:` item in the front matter. The contract is in `_data/components.yml`.

[Primary action](#feature-grid) [Secondary action](#prose)

- Trust item one
- Trust item two

## feature-grid

Section list becomes cards. Bold text is the title; the code span becomes a badge.

- **Available**: Shipped and verified. `status:available`
- **Partial**: Works, with known gaps. `status:partial`
- **Planned**: Designed, not built. `status:planned`

## split: timeline scene

Copy on the left, the `player-mock` visual on the right with `scene: timeline`.

## split: review scene

Proposals waiting for review.

## split: settings scene

The inference settings with real defaults.

## split: agent scene

The CLI and agent view.

## split with terminal

Without a visual, the section's first `sh` code block becomes a terminal.

```sh
kinekino capabilities --json
```

## prose and callouts

Plain reading column. A table:

| Item | Status |
|---|---|
| Example | `status:partial` |

> [!NOTE]
> Note callout.

> [!TIP]
> Tip callout.

> [!IMPORTANT]
> Important callout.

> [!WARNING]
> Warning callout.

> [!CAUTION]
> Caution callout.

## platform-list

- **Windows**: Verified. `status:partial`
- **Linux**: Next. `status:planned`

## timeline

1. **P0**: First step.
2. **P1**: Second step.
3. **P2**: Third step.

## download-cta

No public build yet.

[Primary](#) [Secondary](#)

## Lists

A list with status badges becomes rows with an icon and a right-aligned badge:

- Range drafts and short renders `status:partial`
- Convert SRT and WebVTT `status:available`
- Speech to text `status:planned`

An ordered list becomes a stepper:

1. Mark a range.
2. Render the short.
3. Review it.

A plain list gets a custom marker; nested items indent one step:

- Plain item
- Item with children
  - Child one
  - Child two

A list of only links becomes chips:

- [Features](#)
- [CLI](#)
- [Roadmap](#)

A link followed by text stays a plain list:

- [Features](#) for the full table
- [Roadmap](#)
