---
id: evidence:kinekino/20261029113615
title: 구조화 디코딩 비교 상태
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 구조화 디코딩 비교 상태

2026-10-18. 같은2B Q4/설치 mistral.rs0.9.4 CPU에서 기존 실제3개 프롬프트의 제약 없음/JSON Schema를3회 교대하는18개 요청을 완료했다. context2048,max_tokens256,prefix cache0, 요청 temperature0/seed42를 동일하게 제공했다. [기존2B 결과](text.md)는 온도0.8/seed미고정이어서 이 실험의 대조군으로 섞지 않는다.

domain text_schema는 필수 필드·문자열 원소·알려진 cue ID·연속 범위를 정의하고 text_output은 참조·범위 중복·교정의 단어 순서/내용을 후검사한다. 카드 사실관계는 별도 의미 검토다. [공식 구조화 문서](https://docs.mistralrs.dev/guides/serve/structured-output/)를 참고하되 최신 문서와 설치 버전의 동등성을 가정하지 않고 실제 HTTP/응답을 보존한다.

## 실제 실패와 수정

[첫 측정](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-balanced.json)은 약7.15초 후 exit1이었다. [서버 로그](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-15104/server.stderr)는 `cannot seed the CPU rng with set_seed`를 반환했다. 모델 비교 실패가 아닌 전역 --seed 설정 단계 실패다. 요청 시작0개/서버 시작 실패1개로 구분한다.

전역 --seed를 제거하고 요청 seed42/temperature0만 제공하도록 수정했다. API의 seed 적용과 thinking 설정 처리는 요청 값만으로 확정하지 않는다. 다음 실행의 응답과 고정 구현을 확인한다.

## 대기 이력과 자원 회귀

새 binary player.exe를 예전 preflight가 놓치는 회귀를 발견했다. canonical 이름과 대소문자 감지를 추가하고 회귀 시험을 통과했다. 이전 ready 보고만으로 새 GUI 부재를 입증하지 않는다.

[v3 사전 점검](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-balanced-v3.preflight.json)은 실제 player.exe PID100392 때문에 ready=false였다. CPU 약15.31%/가용RAM 약36.99GB여도 GUI 실기가 실행 중이면 worker를 시작하지 않는다. Get-Process로 해당 PID 생존을 확인했다. GUI는 닫지 않았고 관측 timeout을 종료로 해석하지 않았다.

Rust 어댑터는 임시127.0.0.1 포트만 사용하고 web UI/MCP config 주입·tool 요청·외부 추론은 사용하지 않는다. 프록시/리다이렉트를 끈 localhost 요청이다. 생산용 상주 서비스나 인증 검증이 아니다. 전용 Windows Job kill-on-close와 오류/정상 종료 child wait를 구성했다. worker 강제 종료 시나리오 자체는 아직 실측하지 않았다. 초기 실패 후 mistralrs 프로세스가 없음을 확인했다.

UI 담당이 자신이 실행한 검토 창을 정상 종료한 후 새 label로 재시도했다. 측정 label별 preflight를 보존하며 기존 JSON/stdout/stderr/preflight가 있는 label은 덮어쓰지 않는다. 사용자 ChatGPT/Terminal은 종료하지 않았다.

## 실제18개 비교 결과

[v4 측정](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-balanced-v4.json), [전체 요청/응답](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-63404/manifest.json), [독립 집계](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-63404/evaluation.json)를 보존했다. 18/18 HTTP200이며 시작 실패1회와 미시작 preflight 대기는 별도다.

| 작업/각3회 | 제약 없음: 원문 JSON/구조·참조 | 펜스만 제거 후 구조·참조 | 제약 있음: 원문 JSON/구조·참조 |
| --- | --- | --- | --- |
| 교정 | 0/3 · 0/3 | 0/3 | 3/3 · 3/3 |
| 카드 | 0/3 · 0/3 | 3/3 | 3/3 · 3/3 |
| 하이라이트 | 3/3 · 0/3 | 0/3 | 3/3 · 3/3 |

무제약 교정은 JSON 끝에 여분 따옴표, 카드는 markdown fence, 하이라이트는 문자열 대신 숫자 cue ID를 냈다. 카드는 간단한 fence 제거만으로도 구조 통과하므로 이를 모두 제약만의 장점으로 부르지 않는다. 같은 arm의3회 결과 문자열은 모두1종이었다. 이 입력·설치 환경에서의 관측 안정성이며 다른 하드웨어 재현성/언어 전체 품질을 증명하지 않는다.

제약 교정도 대사 대부분을 누락해 단어/순서 보존0/3이었다. 제약 카드는 칼/땅의 설명을 잘못 엮고 원문에 없는 목적·사실을 덧붙였다. 하이라이트는 모든7cue를 하나로 선택해 유용성 승인이 남았다. 구조·근거 ID 통과9/9를 의미 품질 통과로 바꾸지 않는다. 사용자 자동 적용/게시 승인은 없다.

v4 전체 로드/서버/18요청 wall260.43초, 100ms 표본 최대 private2,993,164,288 bytes/working set3,919,183,872 bytes였다. 개발 빌드와 병행했으므로 idle 성능 순위로 쓰지 않는다. 서버 PID57224가 배치 뒤 inventory에서 사라졌음을 확인했다. 작업 완료 후 UI에 추론 종료를 알렸다. worker 강제 종료 kill-on-close 자체의 실기 검증은 여전히 남았다. 도구 unit12개·fmt·Clippy와 validation unit6개·fmt·Clippy도 통과했다.
