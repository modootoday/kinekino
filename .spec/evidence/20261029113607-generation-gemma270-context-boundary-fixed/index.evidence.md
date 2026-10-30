---
id: evidence:kinekino/20261029113607
title: Gemma270 경계 수정판 실제 재검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 경계 수정판 실제 재검증

2026-10-19 같은 고정 후보와 생성 payload(max_tokens2, ignore_eos=true, truncate_sequence=false)를 수정 엔진으로 실행했다. count109회에서 기존과 같은2047/2048/2049 입력을 선택했다. 원본에서 관측한 보고 total2049는 이번3요청에서 재현되지 않았다. [원본 결과](generation-gemma270-context-boundary-actual.md)를 덮어쓰지 않는다.

| 입력 토큰 | 수정판 HTTP | completion / total | 관측 |
|---|---:|---|---|
| 2047 | 200 | 1 / 2048 | finish_reason=length |
| 2048 | 400 | 없음 | 생성 여유가 없다는 명시 invalid_request_error |
| 2049 | 400 | 없음 | 입력 한도 초과 invalid_request_error |

2047요청58.8175083초이며 HTTP usage의 prefill은58.811초다. 실행 전체72.9611762초, exit0, timeout=false, 표본 peak private1275723776B/working1517105152B다. admission CPU19.3359375%/free35223486464B, BelowNormal·CPU4·offline·maxseq1·prefixcache0·pagedoff 조건이다. 실행 중 다른 업무/자원 조건이 있으므로 이전 실행과 성능 순위를 비교하지 않는다. 소유 server PID111888은 worker 종료 후 CIM에서 부재를 확인했다. 사용자 앱은 종료하지 않았다.

## 고정 실행 근거

- 수정 engine SHA `9256a093133056cff9a7fbbbe37693a8a11abddd5753fe1598316e883f1d78d0`, [CLI 보존](generation-gemma270-cli-build.md).
- Rust worker 및 orchestrator SHA `89edc38413dc48f5a2097357e23b131d249b50820976c29114aff8b6b0e97ab5`. compiler inputs201개와 EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/89edc38413dc48f5a2097357e23b131d249b50820976c29114aff8b6b0e97ab5/build-manifest.json)했다. fmt/62tests/strict all-targets clippy/locked offline build 통과.
- [실제 manifest](../../../evidence/ai/model-spikes-2026-10-18/results/gemma270-boundary-api-fixed-r2/manifest.json) SHA `ef4cdd0bf0a265bb7f317f89e78e52f509634d4e615a1c87dafefaf7f27e1ef9`와 [측정](../../../evidence/ai/model-spikes-2026-10-18/results/gemma270-boundary-api-fixed-r2.json) SHA `aca9587ef28a1e4d27f8315547a75fd87d62d57326d1c8d8a1c72ff8c100b3d3`를 보존했다. 개별count109응답은 manifest의 SHA 목록과 연결한다.

baseline 기본 선택은 기존e862이며 수정판은 명시 generation-boundary-fixed 명령과 Revision::BoundaryFixed로만 선택한다. 다른 모델 profile에 수정판을 요청하면 거부한다. 파일 부재/해시 불일치 fallback은 없다. 새 검사 도구의 저장 바이트 대조 결과는 별도 기록한다.

후속 중앙 checker actual session90044 exit0: 수정 CLI11파일·count109응답·generation3응답·측정을 검사했다. [검사 보고서](../../../evidence/verification/gemma-fixed-integrity-2026-10-19-r3/build-evidence.json) SHA `29f05b2675ac9e3830c6c2ebb827692cc7a633df40930f987b16e48fa3bae978`, 검사EXE SHA `520074c209d5d51dd39e2f7e3c9bc1c697963e8425570cf47cd5af3ada7aaa79`다. 자체fmt/9tests/all-targets strict clippy/release build를 통과했다. [링크 보고서](../../../evidence/verification/gemma-fixed-integrity-2026-10-19-r3/links.json)는 해당 검사 시점392문서/canonical오류0/보존원본미해결3이다. source-at-audit 사본은 검사 시점 소스이며 전체 compiler 입력 attestation으로 표현하지 않는다.

## 남은 게이트

이 결과는 count/chat API 응답과 보고 usage의 전체길이 계약 확인이다. rendered UTF8와 U32LE token IDs, forward offset/position/metadata, 내부 상세 StopReason을 캡처하지 않았으므로 full_boundary_gate_passed=false를 유지한다. 원본 max256 자막 품질3요청을 수정판에서 다시 실행하거나 품질 승인한 결과가 아니다. Gemma 실제 역할/모델 수는 늘리지 않으며 제품 품질 승인0개를 유지한다.
