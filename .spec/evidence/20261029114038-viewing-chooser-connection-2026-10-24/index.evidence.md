---
id: evidence:kinekino/20261029114038
title: Viewing chooser 연결 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Viewing chooser 연결 검증

2026-10-24. PM 단독 실행 배정으로 private UI14와 SDK 시험 헬퍼1을 합성 검증했다. 제품 canonical 채택·stage·commit은 PM 소유이며 이 기록은 채택을 의미하지 않는다.

## 입력과 실행

- root `c351b5d6e87120d6455944e10ff74bc134bd3171`, Meari `b532fe0eed06d8526b755b11bc36416070244ef7`, lock `8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67`.
- UI14 최종 manifest B6CCF9CF42FC64AE3BE7600CD29E362960D181C3D071C3CB65F4D48F28CCE95C. SDK `viewer-fault-adapter-jobs-r1/leaf.json` E7F05CC7B96DE598B4DA3E347DFEA90FC2142D0F9F494D3A18DB167C6894D344. cfg(test) helper의 원 onOpenJobs local binding/key 재등록 두 줄만 추가했다.
- 합성15 `runtime/verification/ui-viewing-chooser-r1/71-composite15-r5.json` SHA256 `68BD2A13545D3AA27A9C48E7D09EF2CEB9633BC1680A5DA9CB2B1440BD3070DD`.
- 최종 고정 BIN `bin/ui-viewing-chooser-r5-tests.exe` SHA256 `7D05E62F7F0B40B9806C74A838524324E34363040A6DDEF0CBF3BCFC9773226C`, 176966144 bytes, Cargo fresh=false. features native-observer-test-support/native-proposal-window. 생산 JSON은 75-lib-no-run-r5.stdout, 사본/생산자 FileID는 76-fixed-bin-r5.json.
- CPU2/affinity3/BelowNormal/descendant Job, offline/buildjobs1/testthreads1. 각 실행 직전 RAM16GiB/GPU free3072MiB/C free12GiB/writer0 입장 확인. resource-runner 원 stdout/stderr/terminal receipt를 보존했다.
- 72 fmt --check, 73 default clippy all-targets, 74 native feature clippy all-targets -D warnings, 75 native lib no-run JSON 모두 exit0.
- 실제 list/ignored 대조 및 추가3 중복0 확인. 79 C3 선행1 통과는 중복이며 고유수에서 제외한다. 80~113 정확34와 114~116 helper 직접 영향3은 각각 actual 1 passed/0 failed/0 ignored, terminal0. 고유37 통과. 결과 원해시는 122-final-results37-r5.json.

## 입증 범위

초기 chooser 슬롯·공통 컴포넌트·typed props 예산 오류, 원 SDK 선택 키와 명시 document/item 선택, 같은 실제 NativeRuntime/source/export/Host admission/zero-batch 경계, 원 Open 검사 연결 및 영향 경로를 확인했다. C2는 닫힌 export가 renderer에서 먼저 거절되는 단계와 보존된 실제 pending의 Host stale 거절을 별도로 검증했다. C3는 명시 adapter fault 모델이며 실제 SDK 오류 발생이라고 표시하지 않는다. C1은 실제 SDK store/host 연결의 양성 근거다. Runtime route 관측은 Reported 모델이며 실제 retained frame 관측이 아니다. scoped fixture2와 실제 같은 runtime3을 구별한다. C1은 Inspected까지이며 전체 native 진입/복귀 화면 인수가 아니다.

기능·시각 검증은 구분한다. 새 GUI 기능 인수, OS 창/수동 스크린샷/디자인 일치, 기본 launcher 전환, CLI 전체/AI/실제 미디어·D 여정·전체 회귀는 이 실행의 범위가 아니다. 시험을 기능 지원 전체로 확대하지 않는다.

## 원 실패 보존

05 타입 clone 오류, 09 시험 모듈 경로 오류, 13 preparation import 오류를 각각 private 최소 후계로 수정했다. 41 C2는 원 export가 이미 닫힌 단계에서 실패했고 정상 cleanup 완료였다. C2 시험은 원 renderer 거절과 Host stale를 명시하는 후계로 수정했다. 70 r4는 setup `callback reference is not exported`, cleanup Complete/worker_stopped=true/unacknowledged0/retained owner 없음이었다. 정상 export와 달리 cfg(test) fault helper의 onOpenJobs 등록이 빠졌음을 SDK가 수정했다. 70-failure-checkpoint-r4.json에 r4의 고유19pass/실패1/나머지14 미실행을 보존했다. 원 BIN F323/334F 및 원자료를 삭제하거나 최종 성공수에 합산하지 않았다.

## 복원과 보호 감사

117-restored15-r5.json: 기존10 bytes/UTC mtime 및 신규5 부재 복원 완료. SDK 채택 F300 sync5는 UI 이전 기준으로 유지했다. root1377/library493/index·manifest·lock6/protected6/carry122/S기존5856은 해시·mtime(보호6 FileID 포함) 변경0. known2 실사본도 현재 기준10B560 해시 일치.

S 신규51은 전용 ui-chooser-same-runtime 시험 저장자료이며 119-S-new-runtime-r5.json에 열거했다. 삭제하지 않았다. 최종 S5907 manifest는 after-S5907.json. T 기존581 중3 변경 및 신규5는 모두 75 Cargo JSON 실제 artifact에 대응한다(120/121 producer 기록). 신규5 실행0, 최종 T586은 after-T586.json. T 전체 불변을 주장하지 않는다. 기존 알려진 build-script 2개는 과거 원2D2094→현재10B560 생산자 미귀속·원backup 부재 예외를 유지하며, 이번 입장 이후 변경0이다. all_cache_protected=true로 표시하지 않는다.

기존 fixed122에 이번 r3/r4/r5 고정 사본3을 더한 carry-fixed125.json SHA256 `0F04D82566CF9C8381817E9CDED9460288F2C83BDD1A459DCA4CE5BE19DA2060`. 최종 pin/process 확인은 124-final-process-pins-r5.json이며 앞선 123의 잘못된 tools/meari 경로 조회를 보완한다.

## 적용한 스킬

modoo.today `modootoday-free/design-delivery-plus/frontend-first-journey` 본문을 읽고 공통 컴포넌트·typed 경계·진입 가능성 기준을 적용했다. `modootoday/software-qa/test-fixture-and-mock-build` 본문을 읽고 실제 owner/실패 주입 모델/원자료 및 실제 결과 구분에 적용했다. `%USERPROFILE%/.codex/skills/rust-pro/SKILL.md`의 Rust DTO/Result/소유권 기준을 적용했다. 새 타 언어 정책·시험 도구는 만들지 않았다.

PM 검토 후 채택 여부와 다음 기능·시각 인수를 별도 배정한다. 이 검증의 shared S/T heavy 슬롯은 반환 가능 상태다.

PM 인수: 정확15 경로를 a7a9ca4cc3a72ffc76a9a9f55422a1428cb356d4로 채택했다. 기본 launcher·전체 D 여정·OS 시각 인수는 후속이다.
