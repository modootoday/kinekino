---
id: change:kinekino/20261116061230
title: Site and README show the MCP server and settings CLI as shipped
status: active
created: 2026-11-16T06:12:30Z
updated: 2026-11-16T06:12:30Z
rel:
  depends_on: [guide:kinekino/site-github-pages]
---

# Site and README show the MCP server and settings CLI as shipped

## Change

README and the site pages (English and Korean: agents, cli, features, index, use-cases) listed `kinekino mcp serve` and `kinekino settings` as planned. Both are in the code and tested (`kinekino-app/src/mcp.rs`, `kinekino-cli/tests/mcp_stdio.rs`). Checked by hand on 2026-11-16: after `kinekino settings set mcp.enabled true`, the server lists `version`, `capabilities` and `schema` by default, and with `allowed_tools all` also the five host-free write commands, each requiring `confirm: true`. The pages now say partial for MCP (off by default, host-free commands only) and available for `kinekino settings`. The agents page no longer says calls go through the local host; they run in process on the `kinekino --input -` path. The README `llms.txt` link points at `docs/llms.txt`, where the file is.

## Migration

None. Documentation only; no command, setting or schema changed.
