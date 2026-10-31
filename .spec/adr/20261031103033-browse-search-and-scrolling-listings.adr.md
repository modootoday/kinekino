---
id: adr:kinekino/20261031103033
title: browse search and scrolling listings
status: active
created: 2026-10-31T10:30:33Z
updated: 2026-10-31T10:30:33Z
rel:
  governs: [crates/kinekino-player-ui/src/controller/browse.rs]
---

# Browse search and scrolling listings

## Context

The 작품 탐색 view could only walk a plugin's browse tree, and it showed one page of at
most 12 rows: "더 보기" replaced the page, so earlier rows disappeared and anything past
row 12 was never on screen at once. Plugins already answer `search` (SDK and
`SourceProvider::search`), but the host never asked. The Meari window mounts a fixed
set of row slots; changing their number needs a remount, which the browse view avoids.

## Decision

- **Search lives inside a source.** When a source is open (the trail is not empty) the
  panel shows a text field and a 검색 button. Searching asks that source's provider
  through the new `Catalog::search(provider, query, page)`; the default answer is
  `Unsupported`, so a catalog or plugin without search shows the usual refusal line
  instead of failing. Results are generic rows (container or playable) and push one
  level on the trail like any opened item, so 뒤로 returns to where the search started.
  The host does not interpret the query or the results; the plugin does.
- **A trail level remembers how it was listed** (`Level { item, page, query }`), so
  "더 보기" on a search result asks the next search page, not a browse page.
- **Listings append and scroll.** "더 보기" appends the next page to the shown rows. The
  row list is a fixed set of `BROWSE_ROWS` = 60 slots inside the existing
  `overflow: auto` list, so Meari's wheel and scrollbar scroll it; no remount. Once 60
  rows are shown "더 보기" hides; going deeper or searching is the way on. 60 is a
  bound on mounted hidden buttons, not a protocol limit: plugins still page as they like.
- **Typing is not a shortcut.** While the focused element is a text input the window
  passes keys to it (Space, F, Ctrl+O are not player shortcuts then) and Enter runs the
  search. Esc still goes back.
- **The browse view stays laid out.** Meari creates a text field's editor at layout
  time and refuses a controlled input that has none, so `#browseView` is hidden with
  `visibility: hidden` and absolute position (class `kk-off`) instead of `display: none`;
  hidden boxes are not hit-tested. The search row is always present and disabled
  outside a source for the same reason.
- Going back re-lists the parent level from page 1 (appended pages are not cached).
- Every newly shown listing (opening an item, searching, going back) scrolls the row list back to its top; "더 보기" keeps the position. Without this the scroll offset of a long list carried over and a short listing opened scrolled past its rows, so it looked empty (found on the desktop). The view carries a `browse_listing` counter and the window resets the scroll of `#browseLists`/`#browseRows` when it changes.

## Consequences

- Plugins gain a visible use for `search` with no protocol change; golden files are untouched.
- A source with more than 60 entries in one listing needs search or sub-containers to reach the rest.
- Korean IME composition depends on Meari/winit text input support; it is not verified here.
