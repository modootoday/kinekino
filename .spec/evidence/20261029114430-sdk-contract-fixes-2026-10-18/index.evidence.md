---
id: evidence:kinekino/20261029114430
title: SDK 감사 후속 수정·검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK 감사 후속 수정·검증

2026-10-18. [읽기 전용 감사](sdk-integration-audit-2026-10-18.md)의 A-01/A-02/A-03/A-07과 문서 상태 A-04 후속이다. 소유 범위는 application/api, player-cli runner와 관련 tests, SDK/CLI 두 정본이다. presentation/native legacy/모델 도구/기존 frozen 증거/부모 Git index는 변경하지 않는다. GUI capture 및 legacy playback 통합 간극(A-05/A-06)은 이번 수정에 포함하지 않는다.

## 변경 계약

정본 API/CLI/ADR에서128UTF8bytes가 필수 정책이라는 근거는 발견하지 못했다. Rust request_id/idempotency_key의 최대 길이는 JSON Schema maxLength와 같은 **Unicode code point128**로 정렬했다. Rust chars는 Unicode scalar를 센다; NFC 정규화나 grapheme 단위로 바꾸지 않는다. 예전 byte 검사에서 거절하던129bytes이상/128codepoints이하 입력은 이제 허용한다. 빈 값/129codepoints는 거절한다. 기존 ASCII 경계는 같다.

request_id는 공개 correlation 값이므로 C0/C1 제어문자를 추가 거절하고 같은 JSON Schema pattern을 내보낸다. 이전에 통과할 수 있던 control ID는 이제 invalid이며 응답에는 cli를 사용한다. 이는 JSON Schema maxLength를 byte로 재정의한 것이 아니다. idempotency_key는 응답에 반사하지 않고 기존 명령 지원 제한을 유지한다. expected_revision은 기존 decimal u64 validation을 유지하며 schema 설명에 semantic range 검사를 명시한다.

정상 Deserialize된 full Request의 validation 오류에는 유효한 request_id를 common ApiError에 보존한다. CLI는 이를 Response::failure에 전달하고 SDK/CLI 공통 Response failure가 ID를 재검사해 invalid/empty/oversize/control 값은 cli로 대체한다. invalid raw JSON이나 unknown field에서 request_id를 부분 복구하지 않는다. 원래 raw JSON/비밀 payload/idempotency 내용을 오류에 복사하지 않는다.

ErrorCode::ALL은 errors module의 enum 중앙 목록이며 schema가 같은 Serde snake_case 표현으로12개 code를 생성한다. source_expired/cancelled/internal 누락이 해소된다. CLI 도움말은 positional discovery/command와 full Request --input 단독 두 형식을 구분하고 중복 command/context flag 금지, 조건부 frame 지원, host/playback/AI unsupported 표시를 유지한다.

## 검증 상태

소유 source만 rustfmt하고 전체 cargo fmt --check가 통과했다. 독립 runtime/parallel-build/sdk-j2에서 `cargo test -j2 --target-dir runtime/parallel-build/sdk-j2 -- --include-ignored --test-threads=1`과 `cargo clippy -j2 --target-dir runtime/parallel-build/sdk-j2 --all-targets -- -D warnings`가 통과했다. 최종 code/control-pattern 변경 후 재검사한 결과다. 총76개(54library+2CLIunit+3CLIcontract+1CLIrequestidentity+6frame+10subtitle), ignored0이다. GUI window/모델을 실행한 것이 아니라 기존 offscreen unit/actualCLI/실제 local media 파일 시험을 포함한 것이다. 이후 UI source 변경에 대한 자동 인수는 아니다.

추가 dependency나 Cargo/Cargo.lock 수정 없이 related Rust test가 실제 CLI stdin/stdout/stderr/exit와 SDK 직접 실행의 Response 동등성을 검사했다. hash는 새 PowerShell 검증 로직을 작성한 것이 아니라 기존 native Get-FileHash를 별도로 호출해 관측했다. 새 test의 optional SDK_REQUEST_ID_EVIDENCE_DIR은 해당 process에서만 설정했고 새 evidence directory를 create_new로 저장한다. source 원문·user secret을 자동 수집하지 않으며 아래 secret marker는 고정된 가짜 시험 문자열이다.

## 실제 CLI 관측

최종 test binary를 다시 실행해 `.spec/evidence/player/sdk-contract-fixes-2026-10-18-r1/`에9개 요청의 원시 request/stdout/stderr 및 index manifest를 보존했다. 재실행은1test 통과14.75초였다. 모든 stdout은 UTF8 envelope1개이고 stderr는 비어 있다. 이 실행은 media 변환/추론/GUI 실행이 없는 read-only 요청이다. 요청 문법 오류·unknown field처럼 Request가 정상 구성되지 않은 경우는 cli를 유지하며 raw JSON에서 ID를 부분 복구하지 않는다.

| case | exit | 실제 correlation | 확인 |
| --- | --- | --- | --- |
| unicode-128 | 0 | 원래 가128개(384UTF8bytes) | schema와 같은128codepoints 허용 |
| invalid-version | 2 | 요청-상관 | response/error 모두 원 ID 보존 |
| unsupported | 3 | 요청-미지원 | playback.status는 여전히 unsupported |
| unicode-129 | 2 | cli |129codepoints ID 비반사 |
| empty-id | 2 | cli | 빈 ID 거절 |
| control-id | 2 | cli | 제어문자+가짜 secret ID 비반사 |
| secret-payload | 2 | public-correlation | invalid version에서도 TEST_SECRET_PAYLOAD 비반사 |
| invalid-json | 2 | cli | partial ID/do-not-recover/TEST_SECRET_RAW 비반사 |
| unknown-field | 2 | cli | strict deserialize 실패, raw ID 복구 없음 |

별도 actual `schema version --request-id evidence-schema`와 --help를 같은 evidence directory에 저장했다. schema catalog12개와 maxLength128/not-control pattern, full Request --input 단독 도움말과 조건부 capture/unsupported 표기를 확인했다. 추가 두 stdout은 Rust9case index 외 파일이며 아래 hash로 식별한다.

실제 사용 EXE: runtime/parallel-build/sdk-j2/debug/player-cli.exe SHA`b8eb54bcae105720656f26692f173fffb1f0ce050d54d968c356e6249eabf944`. index manifest SHA`2a513c20af509420397c364656bd7c2b3dca53c06b5295da488d9ddbd71e3b12`.

| stdout | SHA256 |
| --- | --- |
| unicode-128.stdout | b91fcbeb457bcca6216a863aba880a85e2e6a0891be9dcadb7f2deafa00e90a1 |
| invalid-version.stdout | 8d8738b5b0e5e5cc63d1a2eb779669791d57c902973cb3b783e5e5452b0ad056 |
| unsupported.stdout | 98eaf4364da2dda643733ae5cb0cc7104f459a00e8b88db855b9b0c03c72dbfe |
| unicode-129/empty-id/control-id.stdout | 574eea35316b93c724edb12decb8df9bba3149e96f942456c049d05df31972d5 |
| secret-payload.stdout | 3ae53b6f585ed99a326879843a153a9eb78ef01f65a4cebaa952725882251f59 |
| invalid-json/unknown-field.stdout | fa13251041f66e519e8806888f5437f7940762b877ccdb17cd858483b8f1bef8 |
| schema-version.stdout | 5046c176f7fe9c14b7925ad90f7aa8f62ce5b868744bc05f2fbdf7901dda14cb |
| help.stdout | b538701e79f59dfc2bbe15df88d353ffd576855e14cd64e26a105371f43a35cc |

## source 식별·범위

아래는 수정 후 읽은 source의 SHA이며 해당 검사/EXE/관측 시점을 식별하는 보조 근거다. 전체 tree의 hermetic source→EXE 동등성이나 이후 빌드 동일성을 증명한 것은 아니다. debug EXE 경로는 이후 build로 바뀔 수 있으므로 관측과 함께 기록한 hash를 사용한다.

| source | SHA256 |
| --- | --- |
| src/application/api/commands.rs | d2ebbb83bf1d31d60dc05facd92904ba2430426f3e0b736eeaa2d200e8e2fb65 |
| src/application/api/mod.rs | 837068328ab86d5154be5c764eccf622c05e61d0dc8404210af691542f430a23 |
| src/application/api/errors.rs | afd0a6a55cb00a87f17796ae729822bdb89c89f52d32feacee2860538fbfcd5c |
| src/application/api/schema.rs | 007c9690c90da0ae14d77b598f17d5f92c94088a9bc7cef3616d098123b9586a |
| src/application/api/tests.rs | f9b6684f611ae7657c93bc6186ba03989e29f005154425bf63beedd0cdc68ea0 |
| src/bin/player-cli/runner.rs | 61bb8dbe2fe4623c525d8a6a72d2a84280c4e18ccb5901d9fa0dfa5cc87a4d49 |
| tests/cli_contract.rs | 46b484cfc35169f8fe59f555694a359078c0162131de34c558db8279d8328f24 |
| tests/cli_request_identity.rs | cb77e6272b5bfabce7a760f0ae332bc76a3c8bb91457c13d669e6d92adb52e66 |

SDK/CLI 정본은 현재5개 지원 표면과 전체 목표API·durable host/job/event/모델 계획을 구분하도록 정정했다. 이전 읽기 전용 감사·root 회귀·모델/frozen 증거는 바꾸지 않았다. 신규 dependency/제품명·출력명/GUI·legacy controller·모델·네트워크·자원 정책·VM/DPI/Pages/사용자 앱·부모 Git index 변경은 없다. A-05/A-06과 전체 시각/제품/AI 품질은 미완료로 남는다.
