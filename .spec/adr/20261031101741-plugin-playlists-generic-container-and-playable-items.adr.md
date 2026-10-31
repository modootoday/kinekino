---
id: adr:kinekino/20261031101741
title: "Plugin playlists: generic container and playable items"
status: active
created: 2026-10-31T10:17:41Z
updated: 2026-10-31T10:17:41Z
rel:
  governs: [crates/kinekino-player-ui/src/catalog.rs]
---

# Plugin playlists: generic container and playable items

## Context

Browsing stopped at listings: a source plugin could not hand the player anything to play. The earlier plan modelled one site's shape (series → episodes) in the host. That puts site knowledge in the player, and the mockup's episode panel was built around that one site.

## Decision

- The host knows only two item kinds: `container` (opens a sub-list through `browse`) and `playable` (goes into the play queue). The wire field `Item.kind` is optional; absent means container, so the golden `source/1` documents are unchanged.
- Telling series from episodes, parsing pages and extracting stream URLs stay inside the plugin. The host has no series or episode concept. The player's `Catalog` has only `browse` and `resolve(playable)`.
- Choosing a playable puts that listing's playables into the queue as given, titled after the container the plugin listed them under. The panel is a generic "재생목록" (playlist). Previous and next move through the queue.
- `resolve` returns a URL and headers. The host passes them to mpv only when the URL host is in the manifest's `media_hosts`. The scheme must be https; http is allowed only for loopback (127.0.0.1, localhost, [::1]). Only `Referer`, `User-Agent` and `Origin` headers pass, and any value with CR, LF or a comma is dropped. A refused or failed resolve shows "재생할 수 없습니다".
- When the queue's row count changes, the window remounts instead of sending props in the old shape.
- libmpv: an `END_FILE` whose reason is STOP or REDIRECT (the replaced file) is not counted as a load failure.

## Consequences

- Any site shape (seasons, playlists, single videos) fits without host changes; a plugin only nests containers.
- The SDK's pre-existing `series`/`episode` requests stay in the wire for compatibility, but the host does not depend on them.
- Tests use only the synthetic plugin `com.example.fake` and a loopback media server.
