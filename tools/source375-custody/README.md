# source375 custody and exact gate (private)

Rust-only, dependency-free Windows verification support. `json` decodes the bounded frozen manifest subset with duplicate-key/control/escape/depth checks; `paths` separates public destination responsibilities from pinned private payload locations; `infrastructure` adapts Windows certutil SHA256 and GetFileInformationByHandle; application/stage/exact own independent use cases. It adds no application policy or global configuration.

Usage:

```text
source375-custody ROOT SOURCE_MANIFEST EXACT_PLAN NEW_RECEIPT_DIR
source375-custody stage ROOT B0_RECEIPT_DIR NEW_STAGE_DIR
source375-custody exact PLAN CARGO_JSON GROUP NEW_LOG_DIR CACHE_ROOT SOURCE_RECEIPT
source375-custody adoption ROOT STAGE B0 DELTA_TSV NEW_OUT
source375-custody apply-reviewed ROOT STAGE REVIEW OWNER NEW_LOG
source375-custody canonical-audit ROOT B0 DELTA_TSV NEW_OUT
```

The inventory mode accepts the exact source375/190 pins, verifies all selected hashes and unique test identities, writes selected copies and existing canonical preimages using create_new, and records canonical absence/bytes/hash/FileID/creation/mtime. It never writes canonical paths. The source stage copies only explicit source responsibilities, selecting frozen bytes before copying so no private-stage intermediate overwrite is needed. Existing .git/.spec/.agents/runtime/target/node_modules/.cache trees are excluded. Root .cargo is omitted; a package's existing .cargo config may be copied as baseline and is recorded in the closure. Launcher env and explicit target-dir remain authoritative.

The exact gate reads original Cargo JSON test producers, requires one producer per expected target with EXE/PDB inside the approved cache, captures producer hash/identity/features/fresh, verifies normal and ignored registration, and runs each frozen exact case sequentially. Exit0 with zero/ignored tests is not a pass. It records each stdout/stderr/receipt and checks producer metadata before/after each invocation and content after the group, including on first failure. It does not substitute expected registrations or weaken assertions. The original Cargo command/feature/target/source stage must be pinned by the caller; fresh is recorded, never fabricated. Concurrent Cargo/writers are prohibited by the PM lease.

The Windows Job CPU/priority budget is supplied by the validated resource-runner. The gate has no implicit retry, timeout kill or unrelated-process cleanup. Actual tests' own ownership/deadlines remain in force. A group error preserves the first failure and partial evidence. Output directories must be new; there is no reset/clean/delete or canonical apply mode. This is not the product SDK's own test suite, GUI acceptance or AI quality approval.

Successor support: the caller supplies an explicit source receipt file. The gate preserves its exact bytes and SHA instead of hardcoding the original source375 pin. This is provenance evidence, not proof that the caller's claimed source closure is correct; PM must freeze and verify the closure separately. Per invocation it now serializes original EXE/PDB identity/size/times before and after, plus launcher/child PID and immutable spawn/terminal receipts. Original earlier gate results retain their documented missing per-case PID/metadata limitation.

Adoption comparison is read-only for canonical sources and Git. It pins original source375, original2103-file stage ledger and r9 receipt, validates seven before/after deltas, hashes every known stage file, and compares selected375 plus successor paths to current canonical bytes and B0 expectations. Different existing files get exact preimage copies in the new private report directory. It distinguishes independent Meari ownership, new candidates, equal/no-write and differences requiring owner review. It does not approve adoption or overwrite WIP, and does not discover additional source files outside the original ledger; that limitation is explicit in its RETURN. Current original/delta/status evidence must be reviewed before any mutation.

apply-reviewed requires the PM's documented owner/source review. It pins the375-row candidate table, checks owner boundaries, dirty/untracked conflicts and empty index, freezes selected bytes/preimages before writes, and rechecks hash/absence per path. Existing files use same-volume rename; new files use create-only hard links. Temp files live in build-cache/tmp. A synced prepare/applied journal preserves partial progress; failures never trigger automatic rollback/retry. Root appmod is skipped for a separate WIP-preserving patch. No Cargo/test producer overlaps source application. It never stages or commits.

canonical-audit is the read-only post-adoption known2103-file comparison with input root required to equal canonical. It records source_is_canonical=true and the extra-source discovery limitation. This is an input snapshot, not successful canonical compilation/testing.
