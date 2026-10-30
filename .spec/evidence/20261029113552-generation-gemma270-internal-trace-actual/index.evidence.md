---
id: evidence:kinekino/20261029113552
title: Gemma270 첫 실제 내부 추적과 완결성 실패
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 첫 실제 내부 추적과 완결성 실패

2026-10-19. 고정 진단 engine `d56e22092f0f0b1a5a9c914f8b3d1d6e3bcde3062fe8329618a58dd5cb3f80b0`와 새 worker `d6d8e5d5216f7a7d02295b98ffeae1c24b1d23d4254437ab09ce4313ed9e7405`로 r3의 같은3개 경계 입력을 실제 실행했다. worker는204개 입력+EXE1개 선보존 후 사용했고 fmt·64tests·strict Clippy·locked offline build를 통과했다.

## 관측

| 입력 | HTTP/내부 관측 |
| --- | --- |
| 2047 | 수락·sequence1 연결·forward 진입/반환 각1회·18개 레이어 실제 위치·토큰1개 추가 후 raw 길이2048·최종 ModelLength(2048) |
| 2048 | admission no_generation_capacity·sequence=null·거부 종료 |
| 2049 | admission prompt_too_long·sequence=null·거부 종료 |

내부 request ID는2/4/6이며 sequence ID1과 구별했다. 정규화 message hash가 실제3건 모두 매칭됐고 렌더링3건과 admission3건, 종료3건이 기록됐다. 그러나 `invalid_events=2`, `dropped_events=0`, `complete=false`이므로 recorder 완결성은 실패다. 오류 카운터를0으로 보정하거나 일부 이벤트 존재를 full gate 통과로 승격하지 않는다. 두 invalid의 발생 지점은 현재 journal에 상세 사유가 없어 source 대조로 조사 중이다.

표의 forward 횟수는 저장된 성공 관측 이벤트 수다. 미등록 row에서 sensor key가 생성되지 않으면 실제 model forward가 journal에 누락될 수 있으므로 전체 실제 호출 횟수나 거부 case의 완전한 forward0 증명으로 확대하지 않는다. [원인 분석](../../player/infrastructure/inference/research-followups/gemma270-trace-invalid-r1-analysis.md)에서 미등록 단일 row의 중복 카운트와 sticky 저장 실패를 구분했다. 다음 진단 소스는 이유·row origin·준비 phase·최초 저장 실패·mutation revision을 기록하도록 보강 중이며, 원래 r1 증거는 불변으로 보존한다.

worker exit0/timeout=false,115.1223759초,100ms 표본 peak private1,332,187,136B/working1,564,626,944B였다. 실행 전 관측 CPU23.5294%/free32,416,329,728B로 correctness 관측 gate를 통과했다. 성능 비교 적격은false다. 소유 서버는 Job/프로세스 수명으로 해제했으며 PID 독립 재조회 관측으로 확대하지 않는다.

## 보존

- [실제 run manifest](../../../evidence/ai/model-spikes-2026-10-18/results/gemma270-boundary-internal-trace-r1/manifest.json) SHA256 `d8f872a984d9d7243f57a3150844742209ae40789d78383710e9f087275fce52`.
- [raw journal](../../../evidence/ai/model-spikes-2026-10-18/results/gemma270-boundary-internal-trace-r1/internal-trace/journal.json) SHA256 `574b4a87ab54aeeddfe94b3d4dc43d9dec2f162bca6952efd151ce6a297e266c`.
- [실제 측정](../../../evidence/ai/model-spikes-2026-10-18/results/gemma270-boundary-internal-trace-r1.json) SHA256 `940fed365145a139d6db6530715906bb71b83bc5964e4ec28604faede250d6a3`.

다음은 invalid 원인의 최소 수정·회귀·새 source/EXE 고정 후 재실행, raw journal의 [공통 DTO](generation-boundary-observation-contract.md) 변환, terminal witness/off-on 대조다. 품질max256 결과와 원래 역할/모델23묶음 수를 변경하지 않는다. [구현·컴파일 근거](generation-gemma270-internal-trace-implementation.md)와 이 실제 실패를 구분한다.
