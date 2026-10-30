---
title: "AI-native: CLI and MCP"
description: "People use the app; scripts and AI agents use the same commands through the CLI today and an MCP server later."
ref: agents
nav_order: 4
hero: { c: split, visual: player-mock, scene: agent }
sections:
  - { c: features, cols: 2 }
  - { c: split }
  - { c: prose }
---

kinekino is built so that a person in the GUI and an AI agent in a terminal do the same work through the same contract. The CLI exists for that reason; MCP is a second surface on the same commands, not a separate feature.

## Surfaces

- **CLI**: `version`, `capabilities`, `schema`, `GROUP ACTION`. `status:available`
- **Request JSON**: `--input FILE` or `-` with `--json`. `status:available`
- **Local host endpoint**: `--endpoint` over OS IPC (Windows named pipe, Unix socket). `status:partial`
- **MCP server**: `kinekino mcp serve` over stdio (roadmap P9). `status:planned`
- **Markdown twins and llms.txt**: every page of this site as Markdown. `status:available`

## Safe to call

Every request carries a request ID, an expected revision and an idempotency key. Errors come back with a code and a `retryable` flag, so an agent knows when to try again.

```sh
kinekino --input request.json --json --request-id r-42 --idempotency-key k-42
```

## MCP, as planned

The MCP server will list the same tools and schemas as `kinekino schema` and send calls through the same local host. It is off by default.

| Setting | Default |
|---|---|
| `settings.mcp.enabled` | off |
| `settings.mcp.allowed_tools` | read only |
| `settings.mcp.confirm_destructive` | on: deleting or paid calls ask you first |

> [!CAUTION]
> Until P9 ships there is no MCP server. Agents can use the CLI today.

[CLI reference](cli.md)
