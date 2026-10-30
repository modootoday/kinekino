---
title: "CLI reference"
description: "The kinekino command line, for people, scripts and AI agents."
ref: cli
nav_order: 20
---

Only commands that exist in the current build are listed. Output is JSON by default.

```sh
kinekino version              # product and API versions
kinekino capabilities         # what this build and host can do
kinekino schema COMMAND       # request schema for one command
kinekino GROUP ACTION         # e.g. subtitles convert (see capabilities)
kinekino --input request.json # full Request JSON (context, command, payload)
kinekino platform capabilities
kinekino platform config
```

Common flags: `--json|--human`, `--request-id ID`, `--timeout-ms MS`, `--expected-revision N`, `--idempotency-key KEY`, `--endpoint ENDPOINT`.

| Surface | Status |
|---|---|
| Commands above | `status:available` |
| `--endpoint` local host over OS IPC | `status:partial` |
| `kinekino settings` | `status:planned` |
| `kinekino mcp serve` | `status:planned` |
