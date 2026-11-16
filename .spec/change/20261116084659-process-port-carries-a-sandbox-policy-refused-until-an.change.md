---
id: change:kinekino/20261116084659
title: Process port carries a sandbox policy refused until an adapter enforces it
status: active
created: 2026-11-16T08:46:59Z
updated: 2026-11-16T08:46:59Z
rel:
  depends_on: [adr:kinekino/20261109051610, sot:kinekino/project-status, plan:kinekino/20261031153240, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, adr:kinekino/20261106032051, adr:kinekino/20261106051332, adr:kinekino/20261109121745, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions, sot:kinekino/plugin-sdk-extraction]
---

# Process port carries a sandbox policy refused until an adapter enforces it

## Change

`SpawnRequest` gains `sandbox: Option<SandboxPolicy>`, and `ProcessCapabilities` gains `sandbox_file_access`, `sandbox_network` and `sandbox_system_calls` (`crates/kinekino-core/src/platform/process.rs`). The new `refuse_unenforced_sandbox` gate runs first in the Unix and Windows spawn paths. Both adapters report every sandbox guarantee as `Unsupported`, so a sandboxed request fails with stage `sandbox` and `ErrorKind::Unsupported`, naming the missing guarantees. Existing callers (`kinekino-process`, the source-host plugin worker) pass `sandbox: None` and behave as before.

## Migration

Code that builds a `SpawnRequest` or `ProcessCapabilities` literal adds the new fields. Plugins run as before; the worker asks for the sandbox in a later slice, once an OS adapter enforces it.
