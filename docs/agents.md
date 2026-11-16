---
title: "AI-native: CLI and MCP"
description: "People use the app; scripts and AI agents use the same commands through the CLI or a local MCP server."
ref: agents
nav_order: 4
hero: { c: split, visual: player-mock, scene: agent }
sections:
  - { c: features, cols: 2 }
  - { c: split, visual: code, chrome: terminal }
  - { c: prose }
---

kinekino is built so that a person in the GUI and an AI agent in a terminal do the same work through the same contract. The CLI exists for that reason; MCP is a second surface on the same commands, not a separate feature.

## Surfaces

- **CLI**: `version`, `capabilities`, `schema`, `GROUP ACTION`. `status:available`
- **Request JSON**: `--input FILE` or `-` with `--json`. `status:available`
- **Local host endpoint**: `--endpoint` over OS IPC (Windows named pipe, Unix socket). `status:partial`
- **MCP server**: `kinekino mcp serve` over stdio. Off by default; exposes only commands this build supports without a host. `status:partial`
- **Markdown twins and llms.txt**: every page of this site as Markdown. `status:available`

## Safe to call

Every request carries a request ID, an expected revision and an idempotency key. Errors come back with a code and a `retryable` flag, so an agent knows when to try again.

```sh
kinekino --input request.json --json --request-id r-42 --idempotency-key k-42
```

## MCP

The MCP server lists the commands that `capabilities` reports as supported, with each tool's input schema taken from `kinekino schema`, and runs calls through the same validation and execution path as `kinekino --input -`. It is off by default; turn it on with `kinekino settings set mcp.enabled true`.

| Setting | Default |
|---|---|
| `settings.mcp.enabled` | off |
| `settings.mcp.allowed_tools` | read only |
| `settings.mcp.confirm_destructive` | on: a tool that writes files or state requires `confirm: true` |

> [!NOTE]
> With the default read-only setting only `version`, `capabilities` and `schema` are listed. `allowed_tools: all` adds the write tools this build supports, such as subtitle conversion and snapshot export. Commands that need a running host are not exposed yet.

[CLI reference](cli.md)
