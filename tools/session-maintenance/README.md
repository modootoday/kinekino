# Session build maintenance

2026-10-29 user explicitly requested Python automatic cleanup. This exception is
limited to this maintenance wrapper and its isolated fixtures; product policy and
product tests remain Rust. Uses existing Python 3.12, standard library only.

`session_build.py --log-stem 3183-example --program cargo -- fmt --all --check`
launches the existing resource-runner from the private SDK mirror. Its original
stdout/stderr/receipt remain unchanged. Each invocation reserves the heavy slot,
checks live compiler/runtime processes and 20 GiB free space, fixes native target,
jobs=1 and incremental=0, and supplies a unique managed TMP/TEMP under build-cache.
The maintenance receipt records executable SHA, args and paths. After original
runner termination, only a successful invocation with no live compiler/runtime
may write its success marker and remove its own scratch. Failed/incomplete scratch
is retained. `--inspect` is strictly read-only.

Automatic deletion is confined to matching owner+success markers in this new
session-managed namespace. Legacy tmp, incremental/deps, worktrees, source/WIP,
fixed binaries/PDBs, verification, media/models and global/independent caches are
never passed to this cleaner. It is not an alternative route for the earlier
policy-rejected legacy recursive cleanup. The user performed that legacy cleanup
manually. No timer, background daemon, system policy, download or global setting
is installed. Native Windows is the production scope; fixture results do not
certify other operating systems. Concurrent wrapper invocations fail closed.

After a successful idle invocation, newly appeared loose `.rcgu.o/.tmp/.partial`
files directly in this native target's deps may also be removed individually.
The pre-run inventory excludes existing/first-failure leftovers. No recursive
deps deletion occurs. Incremental is disabled by project config and wrapper env;
this prevents new incremental cache accumulation without deleting historical data.

Windows filesystem calls use extended-length paths. A partially cleaned managed
run whose owner marker is gone may be recovered only from its existing durable
maintenance receipt with the exact managed path, exit0 and idle observation.
Failure/unknown receipts are excluded. `.cleanup.json` records actual completed
cleanup separately from the original runner result. The first long-path failure
and its passing Rust result remain in the maintenance verification directory.

Applied modoo.today `modootoday/build-performance/optimization-portability-review`:
local layout is explicit, original runner limits are retained, jobs overrides are
rejected, independent fixture names and irrelevant evidence preservation are
tested, and failures are distinguished from empty selections. Full build/SDK
validation remains a separate Rust verification step.
