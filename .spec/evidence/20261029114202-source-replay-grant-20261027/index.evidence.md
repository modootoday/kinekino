---
id: evidence:kinekino/20261029114202
title: 원 ledger의 typed replay 발급·일회 소비 — R20
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 ledger의 typed replay 발급·일회 소비 — R20

## 구현·소유권

중앙 PM은 최신 private SDK 후보에 Browser 담당 책임의 발급/spent/current 계약을 격리 구현했다. BrowserProviderPort의 issue_media_replay/consume_media_replay/check_media_replay 기본 구현은 Unsupported다. native Handle forwarding은 기존 원 Shared mutex·ticket lookup·Ready/shutdown 검사를 재사용하며 BrowserWork의 원 metadata ledger/current 검사에 도달한다. 별도 launcher/owner registry·RPC capability 복원은 없다.

MediaReplayPolicy는 trusted in-process Rust composition port다. nonblocking/bounded·provider별 명시 정책을 요구하며 RPC/웹 JSON으로 전달하거나 deserialize하지 않는다. 실제 provider 정책 구현·제품 조립은 추가하지 않았다. 성공하는 정책은 test-only FixturePolicy뿐이며 실제 서비스 허가가 아니다.

OwnedMediaReplay는 non-Clone/non-Serialize이고 원 OwnedMediaStream을 소유한다. 원 MediaStreamBinding Arc 내부 AtomicBool에 발급시도/소비 상태를 보존한다. 발급은 원 work/current 확인→원 issuance spent→credential NoneObserved 확인→명시 정책→원 current 재확인 순서다. Unknown/Present는 정책 호출 전 Unsupported다. 실패한 정책도 원 metadata 재발급을 허용하지 않는다. NoneObserved 자체는 허가가 아니며 정책이 별도로 필요하다. 소비는 원 current를 확인하고 compare_exchange로 한 번만 허용한다. BrowserWork의 native command 후보 clone도 같은 binding의 spent를 공유한다. 실제 native port는 현재 retained work만 조회한다.

## 정확 변경·입력

기존 변경은 browser_network/request_ledger.rs, browser_provider/{mod.rs,media_stream_tests.rs}, infrastructure/browser_provider/mod.rs다. 신규 browser_provider/replay.rs와 replay/tests.rs를 분리했다. private runtime/verification/preimage-r19-replay에 원 preimage를 보존했다. capture_state preimage도 보존했으나 이 epoch에서 해당 소스는 변경하지 않았다. 제품 root·원 donor·WIP/소유권·원 증거는 유지한다.

input-r20 SOURCE.md SHA256 `108863DCB9D3F228FF4A943974574C397A61AAC5AA1BEC177F1C58B309C390B8`; inventory `7B03293E96DFE32F50085797A27349B5A228EDABB57185F9E94C36D162986F44`;2289파일. root/mirror lock은 각각 B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D /955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5 유지. AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 유지.

## 실제 검증

로그는 후보 runtime/verification의1130~1133에 보존했다. 모든 gate는 terminal exit0이며 이번 첫 실패는 없다.

- 1130 all-targets strict, locked/offline/-j1, support feature, -D warnings 통과.
- 1131 provider37pass/901filtered. 새4시험: synthetic policy 성공·원 grant 일회소비/clone 중복 거절·원 work 취소 후 무효, 정책 거절 후 metadata 재발급 거절, Unknown/Present는 policy calls0, 같은 URL/fixture 식별의 다른 ledger는 발급 거절.
- 1132 network39pass/899filtered.
- 1133 fmt --all --check 통과.
- custody verify-current session35230 terminal exit0,2289 content/identity unchanged.

기존 resource-runner conservative2·heavy1·stable MSVC·C-only native/source-sdk-integration-r1/tmp를 유지했다. raw HTTP/실제 브라우저는 실행하지 않았다.

## 남은 계약·검증

구체 provider 정책은 아직 없고 생산 source를 등록하지 않았다. adapter의 ReplayAdmission 소비 연결·원 AttemptScope/control binding, 원 BrowserDocumentSession 보유와 resolve→admit→consume→read→verify→publish 수명 연결이 남는다. 현재 document_session Drop은 cancel/release를 수행하므로 저장된 resolution JSON/URL로 grant를 재구성할 수 없다. 기존 clone을 제거하면 native activate/frame 전송 전 정책을 깨므로 원 binding의 spent만 공유했다. clone 자체를 공용 authority로 노출하지 않는다.

Native forwarding의 실제 OS/browser 실행, stale generation/expiry·redirect/실패/정책panic 행렬·실제 HTTP/probe·제품 SDK/CLI 전체 인수는 남는다. 실제 credential 관측·서비스 정책 근거 없이 FixturePolicy를 생산에 넣지 않는다. DFB0 190/정상bins·AI0/9·GUI/기존 보류를 유지하며 이번 합성 시험을 제품 다운로드 성공으로 계상하지 않는다.

기독 modootoday/software-qa/test-fixture-and-mock-build 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 원 포트/수명·합성 정책과 실제 서비스 구분·원 입력 고정을 적용했다.
