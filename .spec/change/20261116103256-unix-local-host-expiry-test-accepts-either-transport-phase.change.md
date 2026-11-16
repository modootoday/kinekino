---
id: change:kinekino/20261116103256
title: Unix local host expiry test accepts either transport phase
status: active
created: 2026-11-16T10:32:56Z
updated: 2026-11-16T10:32:56Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/src/infrastructure/local_host/client.rs]
---

# Unix local host expiry test accepts either transport phase

## Change

The `local_host` client test `expired_connected_pipe_keeps_identity_query_cause_and_never_dispatches` failed once on the Linux arm64 CI runner: the failure was reported at `read_response` instead of `write_request`.

Root cause: on connection-bound transports (Unix sockets), the expired host closes its socket, but the client's request write can still land in the send buffer. The failure then appears at the response read as an EOF. Both phases already report `delivery: execution_unknown`, and neither dispatches.

The test now accepts either phase on Unix. It requires an OS error code only for the write phase, and still requires no `dispatch` event. The Windows expectation (`not_sent` at `GetNamedPipeServerProcessId`) is unchanged. The product code is unchanged.

## Migration

None.
