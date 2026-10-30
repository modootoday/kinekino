---
title: "Studio: timeline, clips, review, snapshots"
description: "Mark ranges, render clips and snapshots as durable jobs, and review every change."
ref: studio
nav_order: 11
hero: { c: hero, visual: player-mock, scene: timeline }
sections:
  - { c: features, cols: 2 }
  - { c: split, visual: player-mock, scene: review }
  - { c: prose }
---

A project holds your media, ranges and outputs. Each change has a revision, so two tools editing the same project cannot overwrite each other.

## Make things

- **Range drafts**: mark a start and end while watching. `status:partial`
- **Shorts and clips**: render a range as a short. `status:partial`
- **Cards and assets**: export cards and build asset packs. `status:partial`
- **Frame capture**: one frame to PNG with FFmpeg. `status:partial`
- **Snapshots**: crop and export; Windows today, Linux in P4. `status:partial`
- **Subject segmentation and projection packages**: designed, not built. `status:planned`

## Review before it changes

AI and scripts never edit a project directly. They write proposals, and you accept or reject each one in review. Accepted changes keep their provenance.

- Review media and playback `status:partial`
- Accept or reject proposals `status:partial`
- Provenance of edits `status:partial`

## Durable jobs

Renders run as jobs: submit, watch, cancel and retry. A retried request with the same idempotency key does not run twice.

```sh
kinekino jobs submit --input render-short.json
kinekino jobs watch JOB_ID
```
