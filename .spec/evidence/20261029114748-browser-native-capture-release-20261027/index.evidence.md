---
id: evidence:kinekino/20261029114748
title: terminal과 원 native Capture 해제 관측 분리 — R25/R26
status: active
created: 2026-10-29
updated: 2026-10-29
---
# terminal과 원 native Capture 해제 관측 분리 — R25/R26

## 발견·구현

원 native worker는 browser work를 terminal로 바꾼 뒤 루프 끝에서 active Capture를 drop한다. Handle::release는 terminal만 검사했으므로 이 사이에 원 work/ticket이 제거될 수 있었다. 실제 WebView 실행으로 재현했다고 주장하지 않으며 source 경로 대조로 확인한 수명 공백이다.

private SDK BrowserWork에 native_capture_released를 별도 유지했다. 기본값false이며 원 native worker가 원 Capture를 같은 STA에서 drop한 뒤만 기록한다. active 상태에서 관측 기록은 거절한다. release는 terminal에 더해 이 관측을 요구하고 아직 없으면 ResourceExhausted/Original browser capture release is pending으로 원 work를 보존한다. BrowserDocumentSession의 기존 bounded Drop 루프는 해당 pending을 재조회하도록 맞췄다. catalog TicketLease는 기존부터ResourceExhausted 재조회이므로 변경하지 않았다. 새로운 blocking Drop·sleep budget·공용 registry를 추가하지 않았다.

worker는 원 active 종료/교체·cancel 뒤 해제 관측을 기록한다. queued 취소/이미 terminal인 Start는 새 Capture를 만들지 않는다. Capture 생성이 Err로 반환된 경우 원 work는Failed이고 owned Capture가 없음을 기록한다. run_inner의 정상 반환은 원 active가 해제된 뒤이며 남은 terminal work를 기록한다. 오류/unwind를 Capture 해제 관측으로 승격하지 않는다.

관측은 Rust Capture의 원 소유 해제에 한정한다. COM unregister 성공·모든 callback drain·공유 WebView 프로세스/Job family의 물리 종료 ACK가 아니다. 기존 event guard Drop의 일부 unregister 오류는 무시되는 상태이며 실제 OS 해제 전체는 미검증이다. 실제 native 실행·GUI는 보류로 수행하지 않았다.

## 범위·핀

기존 private5파일: application/browser_provider/{state.rs,document_session.rs}, infrastructure/browser_provider/{mod.rs,worker.rs,tests.rs}. 원 preimage 및 R25 첫 실패 시험은 후보 runtime/verification/preimage-r24-native-release에 보존했다. 원 donor/root/WIP·소유권 유지.

- R25 inventory0683D9022EDD662D2A4C25A02B364A93A32D56489B9CC9FF4E953ED6A02A5672,2292파일. 1163 첫 시험 실패 입력.
- R26 SOURCE.md SHA9F7E4E558C05C9D94F5D0F0DDB9A563ED05360AE1F5401C096E08FCEC69056FA;inventoryA0F8D8E0ABDD66AB9E6F6FAA9A4997636717F157DA1E702FD38BD0DDE2327ADA;2292파일. R25 대비 시험의 cancel 요청1줄만 추가.
- root lockB2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D/mirror lock955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5/AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 유지.

## 실제 검증·첫 실패

원 로그는 후보 runtime/verification의1162~1168에 보존했다.

- 1162 R25 support all-targets strict 통과.
- 1163 R25 native Handle pure6pass/1fail. 새 fixture가Queued에서 cancel 요청 없이 Cancelled 이벤트를 넣어 terminal이 되지 않았다. release는 올바르게RevisionConflict를 반환했으며 시험의ResourceExhausted 기대가 틀렸다. 원 실패를 보존하고 실제 cancellation→Cancelled 순서만 수정했다. release 조건을 완화하지 않았다.
- 1164 R26 native Handle pure7pass/936filtered. active release 관측 기록 거절→cancel 요청/완료→native 관측 전 release pending/원 ticket 보존→명시 관측 후 release/재조회NotFound를 검증했다. native 관측은 시험이 직접 넣은 합성이며 실제 Capture Drop을 실행하지 않는다. 기존 lock poison 시험의 의도된 panic stderr는 전체 성공과 구별한다.
- 1165 application/browser 회귀79pass/864filtered.
- 1166 support all-targets strict,1167 기본 all-targets strict 통과. 1168 fmt는 receipt 재감사 결과exit1이며 당시 통과 보고 오류다. 원 replay.check_request 서명 줄바꿈 불일치를1182에서 수정·통과했고 원 실패를 보존했다. 이를 R26 재실행으로 계상하지 않는다.
- custody verify-current session89270 terminal exit0,2292 content/identity unchanged.

heavy1·원 handle terminal 후 직렬·stable MSVC·C-only native/source-sdk-integration-r1/tmp·conservative2·locked/offline/-j1 유지. 설치/브라우저/HTTP/추론/다운로드/수동GUI/WARP/push0이다. R24 actualCLI12회·DFB0 190/정상bins를 새 입력 실행으로 계상하지 않는다. AI0/9·전체CLI/GUI는 남는다.

## 다음 수명 연결의 공백

현 DownloadCandidate::publish는 publication 직전에 source.check_origin을 호출하므로 browser grant가 그때까지 current여야 한다. 반면 JobsCore의 기존 OwnedAttemptSession poll_close는 publication 전 process settlement이며, session owner는 settlement 때 drop된다. 실제 BrowserDocumentSession을 generic G에 넣기만 하면 candidate drop의 bounded Drop에 의존하고 native release pending을 원 core가 보유하지 못한다. 따라서 이 gate 수정만으로 document owner 이전/종료 인수를 완료로 표시하지 않는다.

후속은 [원 문서 수명 통합 계획](../../../plans/player/production/source-document-owner-integration-20261027.md)을 따른다. 기존 원 CAS·출력·실패를 유지하면서 발행 전 current와 발행 후 원 owner 정리를 기존 retaining worker의 단계로 연결해야 한다. 살아 있는 document를 닫고 grant를 임의 frozen authority로 바꾸거나 closed=true로 성공을 만들지 않는다.

기독 modootoday/software-qa/test-fixture-and-mock-build 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 원 소유·단계별 관측·합성/실제OS 구분 및 첫 실패 보존을 적용했다.
