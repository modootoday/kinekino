# Resource runner

Standalone Windows Rust execution-budget helper. The owned launcher and its child use at most the first two permitted logical CPUs, BelowNormal priority, and an inherited Windows Job budget. The runner verifies the child budget, waits for its termination, writes a terminal receipt, and returns its exit code. It adds no timeout or kill-on-close policy.

```text
resource-runner stdout-path stderr-path receipt-path executable [arguments...]
resource-runner --topology
resource-runner --profile conservative2|build4 stdout-path stderr-path receipt-path executable [arguments...]
```

The first form runs an explicitly supplied trusted executable with OS arguments, without a shell. Output and receipt paths must be writable. Receipt fields include launcher/child PID, affinity, logical CPU count, priority, Job inheritance policy, terminal state, exit code, and elapsed time. Job inheritance is distinct from individually observing every descendant.

The second form performs read-only Windows CPU topology queries and prints JSON before entering the execution budget or spawning a child. It compares the existing process-mask `first_two` selection with a diagnostic candidate using different physical cores within the process/system/thread intersection. The candidate is never applied. Query failures and unsupported or incomplete scope are reported explicitly; ambiguous processor groups and 32-bit topology queries are unsupported. This diagnostic does not measure performance or justify changing the default policy.

Application orchestration, pure selection, Windows adapters/record decoding, and JSON presentation are separate modules. There are no external crate dependencies. The package is an independent Cargo workspace.

```text
cargo fmt --manifest-path tools/resource-runner/Cargo.toml --all --check
cargo test --manifest-path tools/resource-runner/Cargo.toml --all-targets --locked --offline -j1 -- --test-threads=1
cargo clippy --manifest-path tools/resource-runner/Cargo.toml --all-targets --locked --offline -j1 -- -D warnings
cargo build --manifest-path tools/resource-runner/Cargo.toml --bin resource-runner --locked --offline -j1
```

The validated checkpoint passed ten tests, strict Clippy, formatting, a build, one child `rustc --version` budget smoke, and one direct topology query on an x64 Windows host. Failure/partial-input behavior is covered by pure tests; other processor-group and 32-bit hosts were not exercised. The new binary has not replaced an existing deployed runner. No performance comparison or candidate-policy rollout is included.

The explicit profile extension is a source-only candidate awaiting a separate gate. Omitting
`--profile` keeps `conservative2`: the original lowest-two process mask and no load sampling.
`--topology` remains query-only. `build4` selects the lowest four bits from the current
process/system/thread permitted intersection, without assumptions about physical cores or SMT.
It never expands a parent Job or changes another process.

Before creating a Job or child, `build4` requires at least four permitted logical CPUs and two
consecutive one-second intervals with CPU usage strictly below 50% and available physical RAM
at least 16 GiB at the interval endpoints. `GetSystemTimes` includes idle time in kernel time;
busy time is `(kernel_delta + user_delta - idle_delta)`. Counter resets, zero intervals, query
failures, unknown profile names, unsupported multiple processor groups, or insufficient masks
reject with exit 125 and a terminal rejection receipt with no `child_pid`. No silent fallback
to a different profile occurs. The policy is admission-only, not continuous resource monitoring.

Receipts retain existing child budget fields and add requested profile/CPU count, effective
CPU count, selected mask, admission/rejection and measured intervals. Job affinity/priority
readback and launcher/child membership in the owned Job are checked. BelowNormal priority,
descendant inheritance, no kill-on-close and owned-child cleanup on verification failure remain.
The canonical deployed runner must not be replaced before this candidate passes its own build,
default-two and explicit-four real-child Job checks, and rejection checks.

The r2 source-only candidate also prepares the new nonterminal receipt and output files before
spawning. After spawn, an owned-child supervisor covers verification, started-receipt, wait,
and terminal-receipt errors. Failure outcomes retain the original PID/error, kill/wait results,
and failure-receipt errors; unresolved cleanup never claims a terminal child. A final guard
cleanup attempt is logged separately if observation still fails. This does not add a Job
kill-on-close policy or terminate unrelated processes. Fault and real-owned-child regressions
are authored but not yet execution-validated.

The r3 source-only candidate finalizes stdout-file, stderr-file and spawn failures as terminal
pre-child launch failures with stage and original error, exit 125 and no child identity.
If the final receipt write also fails, stderr retains both errors. A cleanup result that remains
unobservable after the final guard is explicitly unresolved, not a claim of persistent ownership
or completed descendant cleanup. The short-lived rustc regression proves wait/reap observation,
not forced shutdown of a long-running child or all descendants.
