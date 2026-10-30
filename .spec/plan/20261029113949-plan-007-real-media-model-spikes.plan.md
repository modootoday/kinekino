---
id: plan:kinekino/20261029113949
title: 승인된 UI·공통 SDK와 실제 미디어 모델 스파이크
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 승인된 UI·공통 SDK와 실제 미디어 모델 스파이크

사용자는 UI/UX 제안을 승인했다. ImageGen 이미지는 참고로만 두고 세부 품질은 실제 Rust UI로 판단한다. Pages 갱신은 하지 않는다. 공통 SDK 우선 규칙을 유지하며 모델/도구 스파이크는 제품 통합 전 독립 Rust 도구로 수행한다.

1. 승인 기록과 공통 UI/SDK 계획을 갱신한다. GUI 디테일 검토는 실제 Rust 렌더 단계다.
2. 모든 기능을 모델 필요/불필요로 분해하고 HF의 최신 후보·원본/변환 계보·라이선스·고정 revision·파일 해시·정밀도·Windows/Rust 엔진 지원을 조사한다.
3. 기존 ChatGPT/Terminal을 보존한 resource preflight, CPU/스레드·워커1개·오프라인 추론·종료 후 unload 정책을 적용한다. VM/DPI·장시간 재생 시험 보류는 유지한다.
4. 실제 배포된 영상/오디오/자막을 내려받아 출처·권리·해시·트랙/PTS·언어를 기록한다. 원본은 보존하고 짧은 PCM/프레임/자막·구간 fixture를 파생한다. 번역 자막은 원음 ASR의 정답으로 사용하지 않는다.
5. ASR/VAD/화자·정렬/음악·반복구간/OCR/텍스트 교정·요약·추천/피사체·추적/깊이·투영 별로 load→실제 입력→출력 파일→실행/품질/자원 측정을 한다. 준비/실패/blocked를 성공과 구별한다. 무작위 tensor 추론을 기능 성공으로 계산하지 않는다.
6. 양자화 비교는 같은 입력·전처리·엔진·샘플링·컨텍스트로 한 요인씩 바꾸고 반복한다. 시작한 실패/timeout/빈 출력을 분모에 포함한다. 한 호스트 결과를 다른 하드웨어 보장으로 일반화하지 않는다.
7. GUI/CLI 공통 facade에 검증된 adapter를 연결한다. 수동·저장·변환·편집·투영 패키징은 모델이 필요 없는 경로부터 기능별로 검증한다.
8. fmt/test/clippy, manifest·파일 열림·출력 의미 검토, 제품 적용 여부와 남은 게이트를 책임별 .spec 문서에 기록한다.

범위 전체를 한 모델로 해결하거나 모든 정밀도/GPU/언어가 검증됐다고 표시하지 않는다. 화자 수·음악 점수는 OP/ED 정답이 아니다. 완전3D 생성은 별도 연구 트랙이며2.5D 결과로 완료 처리하지 않는다.

## 현재 검증 상태

이 계획은 남은 작업과 실행 순서를 담당한다. 모델별 실행·품질·미실행 분류는 [상태 정본](../../sots/verification/ai/model-status.md), 관측 사실·원시 출력·한계는 [검증 색인](../../sots/verification/ai/README.md)에 둔다. 준비 당시의 0회·중간 실행 횟수를 현재 상태처럼 누적하지 않는다.

| 작업 축 | 관측한 현재 범위 | 다음 단계 |
|---|---|---|
| 후보·입력 확보 | HF 고정 revision/라이선스/정밀도와 실제 Sintel·KO/JA FLEURS 파생 입력을 보존했다. 모든 후보를 취득·실행한 것은 아니다 | 새 corpus의 원본·사람 주석·시간 축·권리·파일 hash 확인 |
| ASR | Whisper FP32/Q8 및 Qwen ASR 실제 KO/JA/EN 실행, 패딩 반복 decode·프로토콜 접두어 문제 수정 | 오인식·숫자·다양한 실제 발화 평가, 검토 가능한 자막 경로 |
| 강제 정렬 | 실제10회 중 구조3통과/7실패. 숫자 A/B·PCM 이동과 일본어2표본 raw/repair 귀속을 확인했다 | 사람 단어 경계 기준·다른 입력/분절 개선. 구조 통과와 시간 정확도를 구별 |
| 화자·음악 | AMI0~60초 개선실험 및60~120초시간분리추론·정답평가, 원본DER총6쌍대조·JSON정밀도회귀수정 완료. 소수발화자누락·JER약52%로품질미승인 | JER독립대조·독립meeting/언어·시간경계/화자pipeline 개선, 여러 실제 회차 OP/ED 누락/오탐 |
| OCR·피사체·깊이 | 렌더한국어자동crop연결·negative0후보, 원본820초자연크레딧30자동crop→한session순차영어인식. 한프레임탐지/불완전마스크·상대깊이/2.5D GLB | 인간gold/polygon·자연burn-in/다언어/corpus·후처리parity·전체마스크·추적·외부편집기왕복. 완전3D별도 |
| 제작 텍스트 | Qwen3/3.5 GGUF와2B 구조화18요청. JSON 형식 개선과 의미 오류를 보존했다 | 누락/사실 환각/근거 ID의 의미 평가와 수동 검토 경로 |
| 의미 검색 | MiniLM ONNX FP32/AVX2 및 원본 Candle2회 수치 대조, E5 FP321회 완료 | 사람 relevance·held-out·no-match 보정, 질의별 악화 분석 |
| 모델 불필요 경로 | 자막 왕복·실제 프레임·수동 카드4개/12초 세로 영상·GLB 재읽기/렌더 | 프로젝트 저장/재열기·비파괴 편집·자동/일괄 job·GUI/CLI 통합 |
| 공통 SDK·GUI·CLI | discovery·자막 변환·조건부 local frame API/CLI 실행 및 현재 제품 회귀 검사 완료 | GUI 호출과 공통 capability의 차이 감사, 필요한 adapter 연결 |
| 재현·자원 | 새 실행 전 source/EXE 동결, root 단일 워커와 관측 자원 gate 적용. 원본 R2는60% 미만 admission을 실제 통과 | 새 label마다 identity·자원·실패·미검증을 보존. 과거 source 누락을 소급 해소하지 않음 |

시각 구현의 완료 판정은 새 최상위 AGENTS와 프로젝트 UI 규칙을 따른다. 화면 요소 확대·1:1 대응·실행 후 재검토 없이 시각 일치를 승인하지 않는다. 이 AI 실행 계획은 UI 인수 검사를 대신하지 않는다.

## 우선순위와 진행 중인 독립 업무

1. 완료된 SDK 감사·오류 수정과 실제76개 회귀/CLI9case의 범위를 대조한다. supported·조건부·unsupported·DesignSample과 legacy 재생·GUI frame 미연결을 구별한다.
2. [원본 DER 실제4쌍 대조](../../sots/verification/ai/diarization-der-reference-actual.md)는 완료했다. components차이0us이며 JER는 미검증이다. 다른 입력·경계 사례와 원본 JER 검증 범위를 확장한다. 외부 고정 scorer의 Rust adapter·원문 불변·제품 의존성과 분리 정책을 유지한다.
3. PCM배율 전처리는 수정·실제실험했지만 이 입력의지표개선은없었다. [원시segmentation진단](../../sots/verification/ai/diarization-segmentation-diagnostic.md)에서 확인한NO_SPEAKER·overlap누락에 대해 source/reference alignment·원음·문맥/집계·원본전처리대조를 우선한다. 창간군집화도별도개선하며 reference화자수를oracle로주입하지않는다.
4. 나머지 역할의 다음 증거는 [품질 게이트](ai-quality-gates.md)를 따른다. 이미 통과한 검사를 이유 없이 반복하지 않고, 새 변경·실패·해결되지 않은 우려가 있는 범위에 맞춰 검사한다.
5. 모델 결과를 제품 유스케이스에 연결할 때 공통 application 계약을 먼저 사용한다. standalone spike 출력·수동 파일 제작·GUI demo를 제품 기능 성공으로 반환하지 않는다.

SDK/CLI·실제 UI·독립 연구·Rust 변환 도구의 소유권은 각 [병렬 계획](plan-009-parallel-delivery.md)과 인계 기록을 따른다. root는 AI 취득/추론을 한 워커만 수행하며 사용자 ChatGPT/Terminal을 보존한다. Pages·VM·DPI·새 장시간 재생 시험은 재개하지 않는다.

## 판정과 이력 보존

실행/출력 구조·의미/시간/화자 품질·자원·제품 통합·배포 조건은 별도 판정한다. 사람 정답·인증·권리·지원 runtime이 부족한 경우 정확한 외부 조건을 남긴다. 모든 정밀도·GPU·언어·하드웨어의 통과나 품질 실패 모델의 자동 적용을 선언하지 않는다.

이 계획을 갱신하기 전 중간 단계 기록은 [변경 전 원문](../../evidence/player/plan-refresh-2026-10-18-r1/plan-007-before.txt)에 byte 그대로 보존했다. 원본과 복사본 SHA256은 `ab057c11c473d1f19ae298e3f642526c686bc78fb4e22df754818f25f540d557`로 같았다. 이 이력의 “raw 미관측”·4/8/9회·낮은 snapshot 수는 당시 상태이며, 현재 결과는 위 검증 정본을 따른다.
