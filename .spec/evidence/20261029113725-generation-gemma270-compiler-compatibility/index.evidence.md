---
id: evidence:kinekino/20261029113725
title: Gemma cap 엔진의 Rust 1.99 Windows 호환 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma cap 엔진의 Rust 1.99 Windows 호환 검증

후속 [CPU CLI](generation-gemma270-cli-build.md)·[실제3자막요청](generation-gemma270-structured-smoke.md)·[실제경계 API](generation-gemma270-context-boundary-actual.md)를 수행했다. 아래 진행 중 표시는 이전 단계 이력이며 새 경계 수정·재검증은 후속 정본을 따른다.

2026-10-19. [runtime cap 구현](generation-gemma270-runtime-cap-implementation.md)의 고정 소스에서 **원래 strict clippy·최종 core check·전체1371tests가 통과했다.** 새 CLI build·Gemma270 실제 생성·2048 경계·품질은 별도 게이트다.

## 변경 책임

| 위치 | 원인과 수정 |
|---|---|
| 별도 async-trait0.1.89 fork | 생성된 무메시지 must_use가 Rust1.99 double_must_use를 유발. `futures must be awaited or polled` 이유만 명시하고 Future 본문·타입·수명 변환 유지 |
| mistralrs-code-exec/session.rs | Unix 전용 SIGINT_GRACE_WAIT에 cfg(unix)를 적용하고 Windows block의 false를 반환 표현식으로 변경 |
| core/varbuilder_utils.rs | derive(new)의 redundant_field_names를 동일 필드 초기화의 private XLora::new로 교체 |

lint allow나 no-deps로 원래 gate를 대체하지 않았다. 앞선 no-deps는 실패 원인을 분리한 진단이다. tensor·RoPE·native32768/runtime2048 정책은 유지했다. [ADR](../../../adrs/player/adr-engine-compiler-compatibility-overlay.md)에 유지·복구 기준을 둔다.

## 고정 소스와 보존 범위

mistral.rs base는 `4400935451da5e2dc7379a3f92fbbada66557f6c`다. [최종 프로젝트 manifest](../../../evidence/ai/engine-context-cap/gemma270-runtime-cap-compat-r1/manifest.json) SHA256은 `daceffb396ea5a2a74f80dddb033228b8478f05e5cee895540aa2a4fe13cb09a`, patch SHA256은 `242e9cc1dd3dc445687525571fe277577a2a0bd35e3f69e1e1fb43b7d1eee080`이다. cap3파일·Cargo manifest/lock·Windows session·private 생성자, 총7쌍의 원본/수정과 설정·라이선스4개를 보존했다.

[매크로 manifest](../../../evidence/ai/engine-context-cap/async-trait-mustuse-r2/manifest.json) SHA256은 `a8a7b29c618330fb6106cc44ae1d94bc5e1a65f171d4f9894cff42f008504c39`다. 고정 crate archive SHA256 `9035ad2d096bed7955a320ee7e2230574d28fd3c3a0f186cbea1ff3c7eed5dbb`를 검증하고 Rust 도구로53파일을 추출했다. expand.rs 한 attribute만 수정했다. manifest의53개 원본 해시 목록과 evidence에 실제 복사한 expand 원본/수정·Cargo.toml·dual license5개를 구분한다. 별도 fork는 runtime에 남아 있으며 시스템 Cargo cache는 수정하지 않았다.

프로젝트 path patch와 lock에 fork를 명시했다. async-trait registry source/checksum만 path dependency로 바뀌었고 다른249개 dependency는 유지했다. 수정 lock SHA256은 `fd58fe6183d993dfe5add321ccbeffc535fc9e5e08dc96f511b93b4d1471134d`다. manifest의 compile=false·overlay=false는 준비 당시 상태이며 후속 성공으로 덮어쓰지 않는다.

## 실제 명령 결과

추가 [전체 macro 보존 manifest](../../../evidence/ai/engine-context-cap/async-trait-full-source-r1/manifest.json) SHA256 `a48e778d0934e76dc05793b7957ce5a29f17eb932bc65bf46d09096f0b466afe`에 실제 사용 fork53파일과 원본 crate archive를 보존했다. Rust snapshot 도구는 원본 inventory·리뷰한 expand 해시·정확한 전체 파일 집합을 검사하고 복사 후 해시를 재검사했다. 다른52파일은 원본과 일치했다. 준비 manifest를 덮어쓰지 않고 새 증거를 추가했으며, 이 보존도 전체 엔진 컴파일 입력 attestation은 아니다.

rustc1.99.0/b940084d7, x86_64-pc-windows-msvc, target-cpu=native, CPU no-default-features, j2 조건이다. 다른 호스트·GPU·기본 features 통과로 확대하지 않는다.

| 명령 | 결과 |
|---|---|
| cargo clippy --locked -p mistralrs-core --all-targets --no-default-features -j 2 -- -D warnings | r4 exit0,47.21초 |
| cargo check --locked --offline -p mistralrs-core --all-targets --no-default-features -j 2 | exit0,47.04초 |
| cargo fmt --manifest-path runtime/ai-spike/source/mistral-gemma-cap-source-r1/Cargo.toml --all --check | 최종 호환 소스 exit0. 원문 stdout/stderr는 같은 검증 폴더에 보존 |
| cargo test --locked --offline -p mistralrs-core --lib --no-default-features -j 2 -- --test-threads=1 | 최종 호환 소스1371passed/0failed/5ignored. build12분00초·시험10.31초 |
| cargo build --locked --offline -p mistralrs-cli --no-default-features -j 2 | axum0.8.8이 캐시에 없어 다운로드 필요 오류. 세션75417은 전체 명령 exit1. CLI EXE 생성 미완료 |

[strict clippy r4 원문](../../../evidence/verification/gemma-cap-compiler-compatibility-2026-10-19-r1/mistral-gemma-cap-clippy-r4.stderr) SHA256은 `59e37018b191bd3a9f879dc05fbc38629f48e52524154a5262e08bb3414b2152`, [최종 check 원문](../../../evidence/verification/gemma-cap-compiler-compatibility-2026-10-19-r1/mistral-gemma-cap-check-compat-r1.stderr)은 `466f9bf773532095720dbf9a4ec93faddf9635ef7c28e1dfb4184e1482e94edc`다. 같은 폴더에 r1·진단·r2·r3 실패를 보존했다. 이전 cap-only Gemma52/core1371 통과를 최종 호환 소스 결과로 재귀속하지 않는다.

## 다음 게이트

최종 core 전체 stdout SHA256 `c9fe484083178e3c3f78e96fb3e2392993a15ad72bd2025bd3283f7c2e9cef24`, stderr `cd5d827ad57bbdcba192f6d033a42f386eb480c11c2b76133f801aec2979c02b`를 위 검증 폴더에 보존했다.5ignored의 외부자료/benchmark/문서생성 범위는 이전 core 시험과 같으며 자동 통과로 합산하지 않는다. 새 CLI 의존성은 자원 admission 뒤 lock을 유지하는 명시적 온라인 build로 취득하며 오프라인 실패 로그를 보존한다. 첫 자원 관측 CPU92.9657794676806%/free30943924224B는 ready=false여서 즉시 새 build를 시작하지 않았다. 사용자 앱·서비스·cache는 변경하지 않았다.

Rust 중앙 증거 검사기에 최종 프로젝트7쌍/설정4개·macro 전체53파일/원본crate를 연결했다. 변조 거부 회귀를 포함한9tests·fmt check·strict clippy·release build가 통과했고 실제 중앙 check는366문서/정본 broken0/보존된 상위 참조3개였다. engine-lab8빌드의 컴파일 입력76개·EXE8개도 검사했다. [중앙 검사 보고서](../../../evidence/verification/gemma-cap-compatibility-integrity-2026-10-19-r1/build-evidence.json) SHA256은 `9b323231b0e4e1c0dcf9f7423c4abd8ebd0f9639604c44041177627320eaecfb`, 검사 EXE는 `05711143580b0f1deca1d636176f4c6f015dab62384ce5d42165eae3b5e52e95`다. 같은 감사 폴더의 checker-source-at-audit는 감사 당시 소스 복사이며 compiler attestation으로 취급하지 않는다.

CPU CLI 생성·실행 파일/컴파일 입력 보존, 실제 rendered UTF8/IDs·2047/2048/2049 경계·stop 동작, 동일3요청 smoke와 의미·근거 평가가 남는다. SDK AI capability·자동 적용·모델 품질 승인 수는 바꾸지 않는다.

후속 CLI build에서 mistralrs-server-core/src/lora_adapters.rs:377의 Windows 미사용 path 변수 경고1개를 발견했다. 현재 build 소스·증거 identity를 유지하기 위해 컴파일 도중 수정하지 않았다. 앞선 core strict clippy 통과를 CLI/server 전체 strict gate 통과로 확대하지 않는다. CLI/server 경고 원인·수정 및 해당 gate는 별도 후속 항목이다.

후속 Rust `engine-lab build-cli gemma-cap-cli-online-r1`을 시작했다. 실행 자체의 preflight는 CPU29.719264278799617%/free36468088832B로 admission을 통과했다. 고정 base·리뷰한 프로젝트 파일·macro53파일의 현재 해시를 검사하고 compiler·입력·lock·도구 build를 runtime/ai-spike/engine-builds/gemma-cap-cli-online-r1에 기록했다. Cargo는 lock을 유지하며 필요한 dependency만 취득했고 컴파일 중이다. BelowNormal·j2·HF_HUB_OFFLINE=1이며 모델 실행은0회다. Cargo의 정상 dependency cache 취득과 기존 cache 소스를 수정하지 않은 macro fork를 구분한다. 사용자 앱·서비스·OS 메모리 cache는 조작하지 않았다.

build-cli 도구 선보존 뒤 [최신 중앙 r3](../../../evidence/verification/gemma-cap-compatibility-integrity-2026-10-19-r1/build-evidence-r3.json)에서도367문서/정본broken0/보존상위3개·engine-lab9빌드/89컴파일입력/9EXE 검사를 통과했다. 보고서 SHA256은 `5bd91197bbcbd6b512652c71b1bacf6c6b0986915e78c08453be15532800f006`이다. 앞선 r2 보고서를 덮어쓰지 않고 별도 파일로 보존했다. CLI build 성공 여부는 이 무결성 검사와 별개다.
