---
id: adr:kinekino/20261109111833
title: Mascot event port with a no-op default
status: active
created: 2026-11-09T11:18:33Z
updated: 2026-11-09T11:18:33Z
rel:
  governs: [crates/kinekino-core/src/mascot]
  depends_on: [adr:kinekino/20261031111316]
---

# Mascot event port with a no-op default

## Context

The roadmap step M6-4 (DC-3, plan item 39) asks for a mascot event port that does nothing by
default. The mascot's motion presets and renderer live outside this repository; the guide
`mascot-preset-reference` lists the app events its state machine listens for (`app_start`,
`load_start`, `error`, `music_start` and so on). Until this step the code had no mascot
reference at all, so nothing could tell a mascot what the player is doing. This step is the
port, a no-op adapter and contract tests only: no wiring into the player, no new artwork, no
change in any other repository.

## Decision

- `kinekino-core` gets a `mascot` module with `MascotEvent` and the port `MascotEventPort`.
- `MascotEvent` covers only facts the host observes: app start (with `first_run`) and exit,
  load start and end, success, error (with `fatal`), warning, notification, playback start
  and pause, offline and online, and task complete. Each variant has one wire name equal to
  the preset state machine's event key; playback maps to `music_start`/`music_pause`.
- Pointer, hover, drag, stroke, typing, voice, cursor-speed and idle events, and timers such
  as `load_long`, are not host events. The mascot sees its own input and runs its own timers.
- `MascotEventPort::notify` is fire and forget: it takes `&self`, returns nothing, must not
  block, and the trait is `Send + Sync`. A mascot is decoration, so its failure or absence
  never reaches playback or jobs. An adapter that talks to a mascot process queues or drops
  events on its own side.
- `NoMascot` is the default adapter and ignores every event.
- Contract tests check that every variant has a distinct wire name, that the host set matches
  the preset app events exactly, that payload does not change the wire name, that `NoMascot`
  accepts every event, and that the port works as a shared trait object across threads.

## Alternatives rejected

- Returning a `Result` from `notify`: callers would have to decide what a mascot failure
  means for playback; the answer is always "nothing", so the type should not ask.
- Forwarding pointer and timer events through the host: the mascot window owns its input,
  and duplicating timers in the host would let the two disagree.
- Free-form string events: a typo would silently never match a preset; an enum with one
  tested wire name per variant keeps the host and preset table in step.
- Wiring the no-op into the player now: with no real adapter there is nothing to observe, so
  call sites wait until an adapter exists.

## Consequences

- The host can be composed with a mascot later without touching playback code paths beyond
  adding `notify` calls.
- When the preset table adds an app event, the contract test fails until `MascotEvent` gains
  the variant or the event is ruled a mascot-side event here.
- Notification targets and success streaks are not carried yet; streaks are counted by the
  mascot from repeated `success` events.

## Reversal conditions

- A mascot adapter needs back-pressure or delivery confirmation from the host.
- The external preset state machine renames its event keys or moves pointer handling into
  the host window.

## Record

- 2026-11-09: port, `NoMascot` and contract tests added in `kinekino-core`. No call sites,
  no adapter beyond the no-op, no artwork change.
