---
id: evidence:kinekino/20261029113835
title: "J4 r4 probe observation lifetime — Card exact1 passed, 2026-10-24"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# J4 r4 probe observation lifetime — Card exact1 passed, 2026-10-24

## Input and CPU verification
PM allocated frozen r4 effective4 A4810F30EA75431735FFE8EE0BDFA87B2C847FBEAC41807B360CE0602B685CEA, child4B867E9193C5656F997488389683272548BE96CB9C67A79CAE80F27FBE1314E7/24879B. From r3, the only change is one source_redraw_sent=false assignment and comment in separate Accept dispatch setup; parent probe and test3 unchanged. Production application/domain/render code changed0.

Latest SDK r6 cache11 95A1717404BD37AEB5C8406C75A9E5E358576E47444DE26BC40D7567D1529B46 and carry247 A57A556294A6D077A063E1D31AE753A1945F7FD6E111E0356AE9CF8492ED3D5A admitted with promoted62/protection203/rootpins6 and prior exceptions preserved. Gate runtime/verification/default-chooser-commit-journey-j4-r4-cpu-r1: fmt0; native-proposal-window all-target strict0/14414ms; lib no-run0/56879ms; list1259/ignored107; original expected4 and old I4/D/B ten each unique/ignored. Original Cargo JSON selects lib test/native-proposal-window only.

New independent EXE45ADA943E0FFDE94199BDBA505E60FE2E015C39D4AFF01ABD91158BCD991F64C/178295808B, PDBE5412AABAB2DE114A216000746C55EADEB1F4818F0B91B9BB20CF75E86BBC6CC/1094217728B. Prior r2 native1152/0/107 and canonical default1163/0/76 explicitly reused, not newly run. No full duplicate test run. CPU2/mask3/BelowNormal/descendantJob, fresh RAM16GiB/GPUfree3072MiB/Cfree12GiB/writer0, offline/locked/j1/thread1.

## Source defect boundary
The cfg(test) probe consumes source_redraw_sent at first Save ReturnReady observation and originally resets only after chooser return/reopen. Separate Accept already resets its guard/save-observation flags but inherited the consumed redraw latch. Existing one-shot wake advances the host to ReturnReady; a clean production no-op need not emit another rendered notice. Rearming the same probe ReturnReady observation for the separate mutation permits its typed inspection/feedback checks without altering production scheduling. The change can add exactly one existing redraw event for this Accept within event80; it is not redraw0 and not a polling/tick loop. No deadlines, global budgets, predicates, transcript expectations or original owner paths changed.

## Actual evidence
Gate runtime/verification/default-chooser-commit-journey-j4-r4-actual-r1. Exactly original Card test1 passed, outer0/8641ms (harness8.63s), inner run7604ms. Original child61868 creation134373203767099272 exit0/forcedfalse/familyEmpty/stdinEOFrequested. Original inner-success.json run_ok/scenario_passed/closed=true, probe_fault=None, cleanup_status=WorkersQuiescent(Closed); same_runtime=true, source_return_action=true, typed_host_return_requested=false, media_executed=false.

Original nine files independently copied and hashed in02-evidence-inventory.json: inner-success, family receipt, raw streams, writer.lock and four original proposal snapshots. Original Rust transcript/trace assertions passed. Record retains13 original requests/responses: create/create/inspect; list/inspect/inspect; edit/inspect; accept/inspect; list/inspect/inspect. SaveRequested1, explicit journey-accept-requested1. No request injection, synthetic revision/receipt, extra Close/poll/tick/resubmit or product retry. Save expectedrevision1→stored2, separate Accept expectedrevision2→document/output3. Actual receipt automatic_application_enabled=false, source journey-source-0/rev1/hash a*64, source refs a/b preserved. First card remains first/back1/refa; selected second card retains edited Japanese/Korean front and independent back/refb.

Checkpoint43 dispatch calls/replies8,44 observation-wake calls/replies10 actual guardafterrev3 settled; seq23wake/24settled; new seq25 existing LiveSourceNavigationRedraw; checkpoint48 ready ReturnReady inspected3 feedbacktrue/frame25; seq26 journey-accepted→seq27 source VIEW→return/Viewing→fresh chooser→seq44 journey-reopened draft3/output3→seq45complete→CloseIntent/InnerClose/RunReturned/CleanupReturned. Total49 events and76 checkpoint lines fit original80/96 budgets; snapshot8 and512-byte line caps unchanged. Original13 request/response identity checks, selected index1/source/ref/field identity and feedback assertions remain enabled.

## Return audit and limitations
CPU own existing2 restored SHA/bytes/mtime/FileID, new2 and emptydirs absent; source62/protection203/rootpins6/cache11/carry247 delta0, EOF10/PID10absent/writer0/pending0. CPU slot215E9AE5C406426DC61FD5BB12DCC42EFD441AB24C72EB1A29DBD1AEB0C89A0A; carry24987F5C3398B0B91F437E8641A8C90016AFAF1CECE9825A756A6F35EB92B426D51. Actual source62/protection203/rootpins6/cache11/carry249 delta0, alias/worker writes0, EOF7/PID3absent/writer0/pending0. Prior alias FI exceptions and worker FI-only attribution remain historical, not newly classified. Carry content uses completed hashes+freshstats/FI, no whole rehash.

Actual slot0D11C37380993A3A88FEAB0F1B8C3D6D9F65C3AD70989A2C6A7C6E2D5D785985 returned to PM before analysis. SDK may reuse shared slot; this session has no further allocation. Highlight/Cue/Document actual0, D/B actual0, manual visual acceptance0, public/root source integration0. A successful Card probe-scoping fix is not a production feature fix or whole J4 acceptance. r2/r3 original forced failures and r3 preparation error remain preserved; r4 preparation error0.

## Applied skills and next step
Full modoo.today modootoday/software-qa/defect-triage-evidence and %USERPROFILE%/.codex/skills/systematic-debugging/SKILL.md read/applied: bounded discriminating change, retain original failed evidence, distinguish source hypothesis from actual result and per-case scope. Next requires PM assignment for remaining3 fixed-EXE actual cases; do not assume equivalent execution success or repeat completed checks without cause.
