---
id: adr:kinekino/20261109051610
title: OS sandbox adapter for plugin processes
status: active
created: 2026-11-09T05:16:10Z
updated: 2026-11-16T09:10:00Z
rel:
  governs: [crates/kinekino-core/src/platform/process.rs]
  depends_on: [adr:kinekino/20261030200000, adr:kinekino/20261030030000]
---

# OS sandbox adapter for plugin processes

## Context

ADR 0011 (20261030200000) runs media source plugins as separate processes and, in its
decision 6, limits stage-1 isolation to signature checks and the user's install and
permission approval. It lists an OS sandbox adapter (Linux seccomp/landlock, macOS
sandbox-exec, Windows AppContainer) as a follow-up and states that a separate process is not a
sandbox. The roadmap step M5-7 (PG-9) asks for the adapter contract, the security boundary
and what stays unsupported, as a decision record only; no sandbox code belongs to this step.

Facts read on the Linux dev host on 2026-11-09 at main `b37c9593`:

- Plugins start in two ways. The long-running `source/1` worker
  (`kinekino-source-host` `plugin_host/worker.rs`) goes through the process ownership port of
  ADR 0006 (20261030030000): `OsPolicy::process().spawn(SpawnRequest)` with the plugin
  directory as working directory and the parent environment inherited (`environment: None`).
  The one-shot `document/1` call (`plugin_host/document.rs`) uses `std::process::Command`
  directly, also with the inherited environment, outside the ownership port.
- The process port already reports what an OS really guarantees through
  `ProcessCapabilities`, one `Enforcement` value (`Enforced`, `Advisory`, `Unsupported`) per
  guarantee. Windows uses a kill-on-close Job Object; POSIX uses a process group. Neither
  limits files, network or system calls.
- The manifest schema (`kinekino-plugin-sdk::manifest`) carries `signature.key_id`,
  `entry_sha256` and `media_hosts`, but the host does not yet check the signature or the
  hash when it starts a plugin. `media_hosts` only limits which stream hosts the host admits
  from `Resolve`; nothing stops a plugin process from opening other connections or reading
  the user's files.
- `document/1` plugins receive the raw document from the host and need no network;
  `source/1` workers may fetch catalog pages themselves.
- The dev host kernel is 6.12 with landlock in its active LSM list. No Windows or macOS
  machine is available to this step.

## Decision

1. **A sandbox is a property of the spawn, behind the existing process port.** The port in
   `kinekino_core::platform::process` gains a platform-neutral `SandboxPolicy` value that a
   caller puts on `SpawnRequest`. It names only what the child may do, never how an OS does
   it: read-only paths (the plugin directory), at most one private writable scratch
   directory owned by the host, a network rule (`None` or `Any`), and an explicit
   environment. The adapter adds the read-only system paths an executable needs to load (and,
   with network `Any`, to resolve names) and lists them in its report. OS adapters in `kinekino-platform-unix` and `kinekino-platform-windows` apply
   it; no other crate names landlock, seccomp, AppContainer or tokens, and callers add no
   platform `cfg`.
2. **Capabilities are reported, not assumed.** `ProcessCapabilities` gains one
   `Enforcement` entry per sandbox guarantee (file access, network, system calls) using the
   existing three values. A guarantee counts as sandboxed only when the adapter reports
   `Enforced` on the running machine (for example, landlock missing from the kernel makes
   file access `Unsupported` there). Tests and the `plugins list` output compare against the
   reported value instead of the OS name.
3. **Strict by default, never a silent downgrade.** A plugin spawn asks for the sandbox. If a
   requested guarantee is not `Enforced`, the spawn fails with an `Unavailable` error naming
   the missing guarantee, unless the user has granted that plugin an explicit unsandboxed run
   in the plugin settings. That grant is per plugin, shown in `plugins list`, and never
   inferred from the machine. There is no automatic fallback to an unsandboxed spawn.
4. **Network follows the protocol.** `document/1` calls run with network `None`. `source/1`
   workers run with network `Any` in the first stage, because per-host network rules cannot
   be enforced the same way on every OS; narrowing the worker's network to manifest-declared
   hosts is a later decision. `media_hosts` admission of stream URLs in the host is unchanged.
5. **Both launch paths use the port.** The `document/1` call moves onto the process port
   before any sandbox applies to it, so no plugin process is started outside
   `OsPolicy::process()`. The inherited environment is replaced by an explicit, minimal one
   for sandboxed spawns.
6. **Signature and hash checks come first.** The host checks `entry_sha256` and the signature
   before a plugin is allowed to spawn at all; that check is a separate slice and does not
   wait for the sandbox. A sandbox does not replace it, and a signed plugin still runs
   sandboxed.
7. **Platform scope.** Linux uses landlock for file access (and for TCP where the kernel's
   landlock ABI supports it) and a seccomp filter for system calls, applied in the child
   between fork and exec. Windows uses an AppContainer (or a restricted token where an
   AppContainer cannot be used) inside the existing kill-on-close Job Object. macOS has no
   adapter in this decision and reports every sandbox guarantee as `Unsupported`; the
   deprecated `sandbox-exec` is not adopted. Each OS counts as done only after a run on that
   OS shows a sandboxed plugin denied outside its policy; the Linux dev host is not evidence
   for Windows.
8. **Out of scope.** The VAD worker, mpv, ffmpeg and browser engine children are trusted host
   components and are not sandboxed by this decision. No sandbox bypass, test-only escape
   hatch in product code, or WASM runtime is added here.

## Alternatives rejected

| Alternative | Why not |
|---|---|
| Fall back to an unsandboxed spawn when the OS lacks support | It hides whether a plugin is contained; the user could not tell a sandboxed run from an unsandboxed one. |
| Run plugins as WebAssembly (wasmtime/WIT) | ADR 0011 already rejected it for stage 1: source plugins need HTTP and browser capture, and WASI networking is not mature enough; it stays the option to revisit for low-trust third-party plugins. |
| Sandbox inside the plugin SDK (the plugin restricts itself) | A plugin that misbehaves can skip it; containment must be applied by the host before the plugin's code runs. |
| Per-host network allowlists in the first stage | Landlock only filters TCP ports, not hosts, and Windows has no equal per-process host filter; claiming host limits would overstate the guarantee. |
| Separate sandbox port beside the process port | Sandboxing and family ownership are set at the same spawn; two ports would let a caller start a plugin with one and not the other. |
| macOS `sandbox-exec` profiles | The tool is deprecated, and macOS is outside the player's platform scope today (as in ADR 20261109041206). |

## Consequences

- No behaviour changes until the adapter slices land; plugins keep running with the user's
  rights, as ADR 0011 stage 1 already accepted.
- Machines without the needed OS support cannot run plugins unless the user grants an
  unsandboxed run per plugin; the error must say which guarantee is missing and how to grant.
- `source/1` workers keep full network access in the first stage; the sandbox limits their
  file and system-call reach, not who they talk to.
- Plugin authors must keep all writes inside the scratch directory and must not read user
  files; plugins that do will fail under the sandbox.
- Follow-up slices, each test-first: `SandboxPolicy` and capability entries in the port with
  an `Unsupported` adapter on every OS; `document/1` on the process port; signature and hash
  check before spawn; the Linux landlock and seccomp adapter; the Windows AppContainer
  adapter (needs a Windows machine).

## Reversal conditions

- Reconsider the strict default if most supported machines report a guarantee as
  unsupported and users routinely grant unsandboxed runs.
- Reconsider WASM if third-party plugins from authors the user does not know become a
  supported case.
- Reconsider the network rule if an OS-neutral way to enforce per-host limits becomes
  available, or if a plugin needs no network for `source/1`.
- Reconsider macOS scope if macOS becomes a supported player target.

## Record

- 2026-11-09: M5-7 (PG-9) recorded the plugin sandbox contract, boundary and unsupported
  cases without code changes.
- 2026-11-16: first follow-up slice done. `kinekino_core::platform::process` has `SandboxPolicy` (read-only paths, one scratch directory, `SandboxNetwork::None|Any`, an explicit environment) as `SpawnRequest::sandbox`, and `ProcessCapabilities` reports `sandbox_file_access`, `sandbox_network` and `sandbox_system_calls`. `refuse_unenforced_sandbox` is the spawn-time gate: a policy whose required guarantees (file access and system calls always, network when it is `None`) are not all `Enforced` fails with `Unsupported` naming them, before anything is created. The Unix and Windows adapters report all three as `Unsupported` and run the gate, so a sandboxed spawn is refused on every OS today. Callers still pass `sandbox: None`; the plugin worker asks for the sandbox once an adapter enforces it. Unix test: the refused spawn has stage `sandbox`, no owner and the child never ran. The Windows adapter was type-checked for `x86_64-pc-windows-msvc`, not run.
