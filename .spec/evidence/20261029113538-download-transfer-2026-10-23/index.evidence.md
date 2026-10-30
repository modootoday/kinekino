---
id: evidence:kinekino/20261029113538
title: Download transfer fixture gate — 2026-10-23
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Download transfer fixture gate — 2026-10-23

Scope: admitted source8 streaming use case and Windows owned-partial adapter. Actual application10 + Windows filesystem4 + existing storage identity guard regressions2 =16 distinct passing tests. This does not implement or verify actual provider transport, concrete media probe, durable jobs integration, HLS, resume/re-resolution/restart or download CLI support.

Baseline root5ef988b697bf5f6a0732180aa72cb043df533a6c/library0b9ea031ce5c08a5bf99ed9852b668cf2bfce038/root+S lockE469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2. Gate `runtime/verification/media-download-r1`; original stage `runtime/development/media-download-r1`. Source8 original43F9151D1F3C03440A2F90C9BD448987A17AE57037DA3940FB639165C49ECE39 remains frozen. Final-source8 A02376A4EB957B8A1472FBB0D63E5A4F8B84A2DC980EBA04654E4F8D22045368 references seven originals and r2 staged_output.rs.

Fresh RAM28.03GiB/VRAM6221MiB/C125.54GiB/writer0. Existing runner conservative2 CPU2/affinity3/BelowNormal/descendant Job, S/T, offline/locked/j1/thread1. Eight target paths absent in root/S; only explicit8 copied to S, changed bytes did not inherit old mtime. Root Cargo/dependencies/registration/index unchanged. Existing SDK6/Media28 preserved. No network/media/model/GUI execution.

## Failure, diagnosis, repair

Workspace fmt01, metadata02 and related root/contracts/jobs/filepublication all-target strict03 passed. First minimum BIN/list05 exposed14 tests. Exact application10 passed. First filesystem test case11 failed at output.create(), terminal101, and remaining filesystem cases were not run on that BIN.

r1 adds only io_kind/raw_os_error observation to existing ApiError details. Rebuilt exact case07 reproduced InvalidInput/raw_os_error=null, terminal101. Independent PM inspection located installed Rust Windows std creation-mode validation before CreateFileW: create_new requires write or append bool. r2 adds `.write(true)` while retaining raw access0xc0010000/share1/create_new/reparse/guard behavior; diagnostics retained. Neither assertion nor shared storage policy weakened. Four exact filesystem cases then passed. Original BIN, both failed receipts/logs and revision preimages remain preserved.

Final fmt09/strict10 passed. Application10 source stayed byte-identical, so its first BIN passes are retained rather than re-counting duplicates on r2. The filesystem4 ran on fixed r2 BIN. Exact guard2 ran on root-lib actual list669: original metadata guard/reopen and same-volume rename versus copied metadata rejection. No old19/190 broad replay. Fixture ExecutionControl verifies the existing reserve-before-Verifying order but does not prove durable CAS or concurrent real job cancellation.

## Binaries and observations

- First download BIN BD2D172E0C0680AF24138D0A21C8967F76461A19B55F297C23CE2DCA2EE4D60D/1589760 bytes: application10 pass, firstfs fail.
- Diagnostic r1 3AB45326432A016D281697346F07686774334D1AB285E645D71CE52A980FEDDD/1597952: exact failure reproduced.
- Final r2 EDDBC0E712A82D955E8E8D2A7C8DE93F10026DD8AF8A531B2FA27C9459FDD3F0/1597952: filesystem4 pass.
- Guard lib A6740421AE535E3E847E4341FE3A3896CC96D22AF55F19303A8CC52DFA3FEC4F/99436032: direct guard2 pass.

All29 runner receipts terminal:27exit0 and two preserved101 failures. Four Windows fixture cases observed abort cleanup/reacquisition, original-handle rename preserving an existing target on collision, reserved filename rejection and unsealed-output cleanup. Application fixtures cover bounded stream, expected length/digest mismatch, cancellation/deadline, verifier rejection, final session expiry, primary+cleanup error preservation and collision. Computed SHA is received-byte identity, not origin authenticity; observed length is exact-response evidence, not a trusted digest. Synthetic verifier is not a media probe.

Protection: root1286/library479/index-docs6 unchanged; S existing3193 files unchanged and eight new file pins observed;525 protected existing BINs unchanged. Gate-only new BINs are separately pinned. All owned execution sessions terminal; final process snapshot found no gate/build processes. Two existing origin tests leave synthetic directories under the gate's temp folder. Recursive removal was rejected by automatic approval (`blocked by policy`); retain them as evidence, do not claim they were deleted. New download filesystem fixture directories cleaned by their Rust owners.

## Skills and remaining work

Read and applied modoo.today `modootoday/software-qa/test-fixture-and-mock-build/SKILL.md`, SHA a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100: actual contract fixtures, isolated mutation, explicit cleanup/reacquisition, no real-service/timing claim from mocks. Systematic-debugging separated reproduction, error observation, root cause and minimal fix; Rust ownership/RAII guided original handle lifetime. Primary Rust OpenOptions contracts and installed Windows bindings are linked in repairs/README.md.

Next required vertical: original provider session/request/headers and expiry; cancellable blocked transport; actual container/tracks/duration probe compatible with original share contract; formal re-resolution/object identity/Range resume; HLS VOD plan and remux; persistent job/CLI registration and actual authorized provider media acceptance. No capability/CLI support flag is enabled by this gate. PM owns source adoption; gate is not overall product completion.
