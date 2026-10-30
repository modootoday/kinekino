---
id: evidence:kinekino/20261029114445
title: 실패 실행의 origin 정리 독립 진행 — SDK R38
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실패 실행의 origin 정리 독립 진행 — SDK R38

## 변경과 보존 경계

DownloadCoreSession::poll_close는 candidate를 반환하지 못한 원 실행의 origin poll_release를 source/probe 완료 판정과 close_error 반환보다 먼저 진행한다. source·probe·origin 오류는 기존 retain_close_error로 합쳐 원 첫 실패를 보존한다. source/관측/probe/origin 모두 완료되기 전에는 partial abort·source/lease 해제·완료 ACK를 수행하지 않는다.

ProcessSourceOpener::poll_release는 원 closing이 시작된 뒤 원 admission/document 정리를 허용한다. transport receipt 저장 성공인 closed를 선행 조건으로 쓰지 않는다. receipt 실패의 cleanup/error/facts_attempted와 owner_observation Pending은 유지한다. release가 진행됐다는 이유로 receipt를 재시도하거나 transport 성공으로 승격하지 않는다. 성공 candidate는 기존 발행/검증 뒤 candidate.poll_release 순서를 유지한다. 실패 실행의 정리 진행을 성공 후보의 유효성 검사에 섞지 않는다.

변경 전 후보의 owned_session.rs,process_source.rs와 tests.rs/live/jobs.rs/jobs/{cancel,cli}.rs는 runtime/verification/preimage-r37-independent-origin-release에 보존했다. 원 root/donor/source375·기존 실패는 변경하지 않았다. AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 확인.

## 고정 입력과 producer

R38 SOURCE.md B59139EDC660936145F967E796AB7AE9D050C9E24168411396FDD3C7597BA141,inventory A965011734286CF16C04E8457FE32230DAC02D265CC0E597A0AA75D2FDBD12DA/2296.1248 lib no-run compiler JSON/terminal exit0 뒤 fixed-r38/sdk-source-tests.exe를 보존했다:68A0C8085BB36770C7F7CF3FE7BFF25A4AACB521C6CA5A7671C4AD980DDAD8D1/109523456bytes.1254 legacy target no-run compiler JSON/terminal exit0 뒤 fixed-r38/media-download-owned.exe 보존:C2D4735851B9FB8E892E2122FC9C6C0743DBE9F36DE3D81C8C5BA6E1F42CEA3A/3600384bytes.

operator index r38 SHA4CC152363D69524010DCD38B94F9DFDD87BF2576520E295F1633A4476B86B5EE 및 legacy-r38 index SHA4E265878DB85967E3E5E91D6F2C2D8985218063599FC968914EAC6DF7B389AB2는proof=false다. worker R15/1101=477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960,CLI R14/1092=249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B/36405248을 재사용했다. legacy held=D492E6F9AACA0EB1370B0AC8C21F35B60745646CD2F4A55EC75458372DB551A5/209408,ffprobe=EFB014C8F39091C90F3616154FE20AB8470F5F943F6A48EC4FD30C479C7DD697/82620928도 기존 고정 도구다. 새 설치/다운로드는 없다.

## 실제 검사

|실행|결과와 범위|
|---|---|
|1247 support/1253 default strict|각 all-targets -D warnings 실제receipt terminal exit0|
|1248 lib/1254 legacy no-run|각 실제receipt terminal exit0·위 producer 고정|
|1249 transport_receipt_failure_is_not_retried_or_acknowledged|exact1pass/953filtered;원 sink request_id 유지·receipt1회·Pending 관측·origin adapter release 진행·원 guard 보유;다시 close해도 원 오류|
|1250 pending_probe_does_not_block_original_source_cleanup_or_release_partial_early|exact1pass/953filtered;원 SDK/JobsCore 취소 중 probe가 Pending인 동안 원 document owner release 완료를 관측;Cancelling/partial 보유/guard drop0 유지 후 probe 해제→Cancelled·미발행·정리|
|1251 정상CLI/1252 취소CLI|각 exact1pass/953filtered;정상 발행/Completed 및 취소/미발행/원 partial 정리;fixture5RPC/7RPC|
|1255 원 파일 source 취소|exact1pass/30filtered;Cancelled/미발행|
|1256 원 파일 정리 실패|exact1pass/30filtered;Failed/Original source close failed;취소 ACK 금지|
|1257 legacy held client-drop|exact1pass/30filtered;원 child/lease 유지와 취소 정리|
|1258 owner observation|4pass/950filtered;원 run/병렬 close 오류·비정상 exit/forced/Pending 보존|
|1259 fmt|실제receipt terminal exit0|

선택 actual7은 고정 C2 driver로 각각 독립 실행했다. 각 outer의 selected_one=true/ownership complete/physical_closed=true/first_error=null·process exit0/Empty/nonforced를 확인했다. actual7의 outer는 runtime/verification/source-sdk-independent-origin-{sink-failure,probe-pending,cli-normal,cli-cancel,legacy-cancel,legacy-cleanup,legacy-drop}-os1-r38/outer다. 원 raw/내부 receipt/journal은 각 tag tmp와 outer에 유지한다.

1249는 실제 원 Windows IPC와 합성 admission release counter를 검사하는 adapter 경계다. 원 sink 실패를 production core에서 성공 terminal로 바꾸는 시험이 아니며, 시험의 명시적 guard drop을 production core의 cleanup ACK로 해석하지 않는다.1250의 BoundAdmission은 원 BrowserReplayAdmission/BrowserDocumentSourceOwner가 release true를 반환한 뒤에만 gate를 기록한다. browser/native 관측은 fixture다. 실제 shared WebView/COM 종료나 실제 HTTP는 실행하지 않았다. receipt-failure adapter와 pending-probe core 증거를 하나의 완전한 persistent-receipt-failure core 종단 성공으로 합치지 않는다.

1255/1256은 probe 시작 전 source 단계에서 끝나므로 ffprobe는 guard-only다. 실제 ffprobe 정상/손상 검증의 선택9는 이전 R37 근거로 유지한다.1257 내부 held child의 의도된 forced 종료와 외부 test의 정상 종료를 구별한다. source 원 실패·Pending을 성공으로 정규화하지 않는다.

## 입력 동일성과 다음 작업

선택 SDK4 뒤 verify-current session84287 terminal exit0/2296 content·identity 동일을 확인했다. 기존 파일3 뒤 최종 verify-current session84077도 terminal exit0/2296 동일을 확인했다. C-only cache/tmp·conservative2/CPU2/BelowNormal·locked/offline/-j1·같은handle terminal 확인 뒤 heavy1 직렬을 유지했다. 이번 첫 compile/실행 실패는 없다. 이전 R37 및 역사 실패·자원 배정 오류는 원 기록대로 보존한다.

production provider 정책·최초 원 source factory 소유권/등록·실제 HTTP/probe 통합·지속 receipt 실패/복구 행렬·전체 CLI 인수·GUI/AI 비교/수정/판정/이력과 시각 인수는 남는다. 원 DFB0 지정190/정상bins와 이전 R37 선택9를 R38 재실행으로 표시하지 않는다. source375 private overlay·WIP/소유권·AI 사용자0/9/NeedsRepair·추론/다운로드/수동GUI/WARP/push 보류 유지. 배포 제외.

기독 modootoday/software-qa/test-fixture-and-mock-build SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 독립 teardown·원 소유권/취소/오류·합성과 실제 범위 구분 및 code-change-review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 producer/원 실패·주장/근거 분리를 적용했다.
