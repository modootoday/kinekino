---
id: evidence:kinekino/20261029114825
title: 제품 CLI review producer/host 시작 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 CLI review producer/host 시작 검증

2026-10-27 중앙 PM. [명시 시작 계약](../../player/application/review-startup.md). 공통 `prepare_timeline`을 제품 CLI `--serve-review`에 연결했다. pinned startup JSON은 source/case/media·도구/한도 선택이며 authority DTO가 아니다. 실제 video/PCM/replay/캡처→bundle→새 package token→live resolver→동일 SDK source/playback→원 stdio shutdown을 사용한다. source375 private overlay·원 archive/과거 호스트의 성공과 구분한다.

## 첫 입력과 후속 보완

첫 입력 SOURCE `F64CA3E256DBE43F83775B6E442E349C11AA8D72785CC8E777E1535A9BA75472`, inventory `74C44D834F260D35D1B1FB1906A7F178E06922E7153B63C6473A0D62EBB51192`,2226파일을 source375-review-startup-input-r1에 보존했다. 546strict/548no-run/549시작 옵션2/550실제1(39.23초)/551all-targets strict/552~553Product145가 통과했다. 실제 원본 `runtime/verification/review-startup-actual-datalab-r1`의 두 제품 host PID24928/22452는 각각20666/18131ms·exit0·비강제·원 child wait/streams join을 기록했다. 원 미디어30376/36100도exit0·Job empty·IPC exited·비강제다.

두 host에서 fresh producer·새 빈 user 생성/Verified2/replay·실제 headless 재생 중 EOF 종료·재시작 후 이전 session 거절·기존 association2 보존을 확인했다. 새 case9E660931…/AB17A2A1…를 자동 migration하지 않았다. actual producer는 정상 CLI SHA7DFEDAC68A297E5E3088BEEB9FAE449CC06DBAA3D8629491928BCB444523748D다.

첫 strict542는 Infallible match unreachable,543은 private module 접근/비Clone ApiError,544는 collapsible_if로 실패했다. 로그와 실패 전 소스를 source375-preparation-r1-preimage에 보존했다. 최초 fmt의 모듈 경로 해석 실패도 수정했으며 성공으로 계상하지 않는다.

종료 재검토에서 incomplete 분기가 원 shutdown 첫 오류 대신 fallback Timeout을 선택할 수 있음을 발견했다. 원 pending 관측의 첫 오류를 우선하고, startup transport 실패의 cleanup 오류가 원 startup 실패를 교체하지 않도록 보완했다. package 한도도 producer 생성 전 거절한다. 첫 입력/실행을 지우거나 이 보완의 검증으로 승계하지 않는다.554/555all-targets strict 통과 후 r2를 새 고정 입력으로 검증한다. r2에는 실제 stdout reader를 닫은 ready_write 실패/exit8 및 미디어 child 미생성 시험도 추가했다.

## 수정 입력 r2 실제 검증

SOURCE `A2FF8328AE03CF14DE19453D79148E849A9989BD0D25B33929894F33D855685C`, inventory `1B54D8DB74D6D06CA6F0C68E75438D2ED9E2A22B097408A9E38C1BFEA93DEA25`,2226파일을 source375-review-startup-input-r2에 새 고정했다.555all-targets strict/556snapshot/557no-run/558actual1(47.19초)이 통과했다. 원본은 `runtime/verification/review-startup-actual-datalab-r2`다. 정상 host PID21716/33352 두 개와 별도 broken-ready host 한 개를 실행했다. actual producer 정상 CLI SHA `DF846071C2C34965A48139D24B001F23F60A7DB1D72E62CE5A577A79EA0CBD98`.

정상 새 case `07D32AB26DF327DFE89E716F874D63E5BB32E174BF04BD71586C520BF010507E`/`BAB49C78859ADA7E3ACE51B827B66497B96D9D83E94CDE5581682DB03198CDD7`, manifest `AA8CB3BA8978642B9531A323514752909C6937326F3C6C8D1F0F72DC96793AEB`/`75FE4D196F1619B70D03C2F964822AF20A285C71A266B8DA9A280EDB87B0DCF6`를 보존했다. fresh generation→실제 CLI create/Verified2/replay→같은 연결 재생→live 상태 EOF 정상 종료→새 host의 이전 session 거절·기존 저장 association2 보존을 다시 실행했다.

broken-ready는 실제 stdout reader를 준비 전에 닫아 원 ready_write BrokenPipe/os232를 유발했다. 제품은 engine_failed·exit8을 보존하고 추가 미디어 child를 만들지 않았다. stderr/raw·원 child PROCESS와 wait/join을 보존했다. 이 host를 정상 성공으로 계상하지 않는다. 정상 두 host의 미디어 PROCESS와 별개다. 원 pending의 초기 오류 보존 코드 보완이 모든 fault 조합을 입증하지는 않는다.

같은 r2 입력의 후속 통합은 아래와 같이 완료했다. r1이나49C 입력의 성공을 승계하지 않았다.

| 로그 | 결과 |
|---|---|
|559/560|Product no-run·exact145/145|
|561/562|독립 Meari no-run·exactCore18/18|
|563/564/565|Native no-run·exact26/26·자동OS1/1|
|566|정상 player/player-cli build, 두 artifact profile.test=false/fresh=true|
|567|기존 저장/CAS/replay·combined·readonly CLI3/3,25.25초|
|568|현재 입력의 시작 옵션/필수 설정2/2|
|569|fmt --check exit0|
|570|전후2226파일 content/identity 동일·stage 없음|

네 group-return.txt는 source375-review-startup-{product,core,native,os}-exact-r2 아래 동일 SOURCEA2FF…/execution_result=Ok/producer_integrity=Ok를 보유한다. frozen190 계획 SHA082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464·등록/ignored·원 EXE/PDB를 대조했다.

566 정상 player SHA `B3D39F432B2EB5A0AB1E7BB49BC811061FD5A132E1931989874760951EAC0769`, CLI SHA `DF846071C2C34965A48139D24B001F23F60A7DB1D72E62CE5A577A79EA0CBD98`. root HEADd13cbfca140e41863b4d0bd27930be3f57a1123f/Meari0844f6c24caf224617ca8abbb2dd75695a971b99·두 lock·Cargo/rustc/custody SHA가 기존 핀과 일치한다. C native/meari-native/tmp·locked/offline/jobs1·CPU2/BelowNormal·producer1 유지. no dependency install·전역 toolchain 변경 없음.

## 남은 범위와 적용 판단

준비 중 stdin 취소, producer 오류의 원 owner 보존/cleanup 미완료/강제 crash 및 추가 IO fault 행렬은 미검증이다. 정상 두 host 성공으로 이 범위를 닫지 않는다. 실제 Sprite 기술 표본이며 actual9·AI 사용자 품질 승인·GUI 시각 인수가 아니다. WARP/수동GUI/추론/다운로드/push 보류, 기존 소유권/WIP/원본·첫 실패 보존. 전역 toolchain 변경·배포 없음.

기존에 읽은 modoo.today software-qa/code-change-review-verification·test-fixture-and-mock-build 및 ai-quality-plus/agent-run-postcondition-and-replay-check(body SHA61843099B7D54C3D7AC226218A5EE1B6CCDBE36EF84268762DBC33A68FBB58B5)의 원 process/후조건·실제/fixture·claim/근거 분리를 적용했다. private config SHA와 원 EXE·각 child receipt를 보존하고 PID/nonce/시각을 고정 성공값으로 만들지 않았다. Codex runtime 인증을 주장하지 않는다.
