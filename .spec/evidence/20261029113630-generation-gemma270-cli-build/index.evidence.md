---
id: evidence:kinekino/20261029113630
title: Gemma270 runtime cap CPU CLI 빌드
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 runtime cap CPU CLI 빌드

## 별도 경계 수정판 빌드

2026-10-19 후속 수정판은 CPU CLI build exit0, Cargo1223.3253407초다. EXE98790400B, SHA256 `9256a093133056cff9a7fbbbe37693a8a11abddd5753fe1598316e883f1d78d0`를 [새 보존 manifest](../../../evidence/ai/engine-cli-builds/gemma270-boundary-cli-r2/manifest.json)에11파일과 함께 저장했다. manifest SHA256은 `657baf1922c7d2afdd80f515f97500ad1b6713030ad629e5be452423f5004ec5`다. engine ID는 gemma270-boundary-fixed-cpu-r2다. `snapshot-boundary-cli` 명령은 해당 고정 build label/EXE hash를 검증하며 원본 snapshot과 기본 엔진 선택을 교체하지 않는다. 보존 도구 fmt/2tests/strict all-targets clippy/locked offline build를 통과했다. 빌드 및 보존은 모델 추론0회이며 실제 경계 재검증과 내부 trace 게이트는 별도다.

## 원본 실행판 이력

2026-10-19. [고정 소스와 호환 수정](generation-gemma270-compiler-compatibility.md)의 CLI를 실제 빌드하고 실행 전에 보존했다. 기존59ee 설치 엔진은 유지하며 Gemma270 profile만 새 고정 엔진을 선택한다. 다른 profile은 기존 엔진을 유지하고 파일 부재/해시 불일치 시 fallback하지 않는다.

`cargo build --locked -p mistralrs-cli --no-default-features -j 2`는31분28초/exit0였다. rustc1.99.0/b940084d7·MSVC·target-cpu=native·BelowNormal 조건이다. lock SHA256 `fd58fe6183d993dfe5add321ccbeffc535fc9e5e08dc96f511b93b4d1471134d`는 전후 같았다. admission CPU29.719264278799617%/free36468088832B 뒤 고정 dependency를 취득했다. 다른 CPU/GPU/배포와 full transitive/native attestation은 미검증이다.

EXE98790400B의 SHA256은 `e862c74e937d4b250b0a8687aac5282ebdfb685fcf07408e7bd5e6b1374be229`, engine ID는 gemma270-runtime-cap-cpu-r1이다. [보존 manifest](../../../evidence/ai/engine-cli-builds/gemma270-cpu-cli-r1/manifest.json) SHA256은 `c00c05aa8db4e1e8b3a553958d5de445cd32947417c03b3d5b5b55138128fdd4`다. EXE·build-inputs/compiler·명령/종료·admission·로그 등11파일을 복사 후 해시 검사했다. Rust snapshot 도구의 fmt/2tests/strict clippy/build와 자체 source/EXE 보존도 완료했다.

Windows 경고는 server-core의 Unix 전용 path 변수와 CLI manage의 Unix 전용 Context/bin_dir, 총3개다. core strict clippy를 전체 CLI/server 통과로 확대하지 않는다. 후속 수정 소스에서 플랫폼 조건을 정리하지만 이 EXE와 원래 로그는 유지한다.

실제 [109count/3경계](generation-gemma270-context-boundary-actual.md)와 [자막3요청](generation-gemma270-structured-smoke.md)은 이 EXE로 수행했다. 실행 워커 b36e46ef09a822aab2a516823cd31e756fb09692166853e796b5d2aa1d55017a는 compiler inputs201개/EXE1개를 선보존했고 변조 엔진의 spawn 전 거부 회귀를 포함한62tests·fmt·strict clippy·locked offline build를 통과했다.
