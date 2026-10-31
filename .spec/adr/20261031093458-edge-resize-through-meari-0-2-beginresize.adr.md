---
id: adr:kinekino/20261031093458
title: Edge resize through meari 0.2 BeginResize
status: active
created: 2026-10-31T09:34:58Z
updated: 2026-10-31T09:34:58Z
rel:
  governs: [crates/kinekino-player-ui/src/window]
---

# Edge resize through meari 0.2 BeginResize

## Context

The undecorated player window had edge resize only on X11, done by hand because `_NET_WM_MOVERESIZE` from the player's own X connection lost the pointer grab. meari 0.2.0 adds `WindowAction::BeginResize(ResizeDirection)` over winit `drag_resize_window`.

## Decision

Move to meari 0.2.0. A press in the resize band maps the EWMH edge code to a `ResizeDirection` and asks the OS through `BeginResize` on every target. X11 keeps the manual drag as a fallback when the request is refused, or when `KINEKINO_X11_OWN_RESIZE=1` is set.

## Consequences

- X11 (Xvfb desktop with a WM): verified. The SE corner went 1280x720 to 1480x820, and the top edge moved y with the height held.
- Windows and Wayland: built but not verified on a real session.
- macOS: meari reports Unsupported, so there is no edge resize there.
