---
id: evidence:kinekino/20261029113721
title: 분절 단일 요인 A/B 고정 입력과 대조 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 분절 단일 요인 A/B 고정 입력과 대조 계획

2026-10-18. [원본 tokenizer 결과](forced-segmentation-actual.md)에서 실제 정렬 A/B용 외부 단위 파일6개를 Rust로 생성했다. **새 모델 로드·tokenizer 실행·정렬 추론은0회다.** SDK 세션의 새 isolated adapter와 frozen build 인계가 완료되면 root가 실제 대조를 실행한다.

## 입력 보존

생성기는 원본 `segmentation-functions-r1/segmentation-result.json` SHA256 `b4b09dde514f23903698ec9341b1aefcc6c21e1c779095e95ab447356fbd6089`, 고정 transcript3개와 오디오3개 SHA를 확인했다. 원본에 보존한 기존 Rust 단위 목록과 원본 tokenizer의 정리 후 목록을 각각 그대로 옮겼다. 다시 분절하거나 글자를 추가/삭제하지 않았다. 단위 수1..512, 단위 길이1..256B 및 control/marker 문자를 검사했다. SDK adapter는 별도로 Unicode 정리·전체 concat·transcript/language/provenance 계약을 검증해야 한다.

| 입력 | baseline-control | original-tokenizer |
|---|---:|---:|
| KO1883 | 19단위 | 19단위 |
| JA1516 | 28단위 | 28단위 |
| JA1634 | 31단위 | 32단위 |

각 JSON은 `schema_version=1`, `transcript_sha256`, `language`, `source{name,evidence_sha256,case_id}`, `words`를 갖는다. 원문·오디오 경로/해시와 각 파일의 SHA는 [cohort manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-cohort-r1/result.json)에 둔다. manifest SHA256은 `0b83adb94cac29abca3c6b1fbbca71b919c57c16664bc42811572626e3052e18`이다. 같은 디렉터리의 `{ko-1883,ja-1516,ja-1634}-{baseline-control,original-tokenizer}.json`6개가 실행 입력이다. 이 데이터는 모델의 정답 timestamp가 아니다.

## 사전 대조와 실제 실행 순서

후속 [실제5회 A/B](forced-segmentation-ab-actual.md)를 완료했다. 아래 순서의 default 과거 head 동등성·JA1516 same-words control이 실제 통과했고 원본 분절 실험군에도 기존zero5개가 남았다. 이 문서의 사전 계획과 후속 관측을 구별한다.

1. SDK가 새 source fork/patch·컴파일 입력·EXE·정확한 CLI·검사 결과를 인계하면 root가 직접 검토한다. 기존 frozen fork/EXE/결과를 수정하지 않는다.
2. 같은 새 EXE의 비추론 processor 경로로6개 파일을 검사한다. baseline-control은 기본 경로와 단위·prompt·BPE IDs가 같아야 한다. 원본 일본어 단위의 prompt/BPE 차이는 전체 보존한다. 아직 생성 파일을 SDK에서 검증 완료했다고 표시하지 않는다.
3. JA1516의 새 기본 경로를 기존 frozen head 관측과 먼저 비교하고, 같은 단위 목록을 명시한 baseline-control을 실제 실행해 class·raw/fixed/final·full logits 동일성을 확인한다. 새 기본 경로 또는 주입 경로 자체의 수치 변화가 있으면 원본 분절 효과를 해석하지 않고 원인을 조사한다.
4. 같은 EXE·모델 revision·원본 PCM/텍스트·CPU F32·threads4·batch1에서 JA1516·JA1634 각각 기본/원본 tokenizer 단위 목록을 순차 실행한다. 먼저 완료한 기본 JA1516은 그 쌍의 기준으로 재사용할 수 있다. 계획상 최소5회이며 실제 실행/자원 대기는 결과별로 따로 계수한다.
5. 기존 frozen head 관측과 새 기본 경로를 비교하고, A/B 결과는 분절 수·원문 보존·길이0/역전과 raw/repair 차이를 구별한다. 단위 목록이 달라지므로 단순 index zip이나 같은 단위 수만으로 시간/단어 대응을 판단하지 않는다. 필요한 경우 공통 원문 문자 경계에서 비교하고 사람이 검토한 시간 정답과 구별한다.

root의 검토된 EXE/manifest/input admission과 단일 worker 자원 조건을 유지한다. 전후 단위 수를 맞추기 위한 임의 split/merge, zero clamp, runner-up 강제 선택은 하지 않는다. 관측 목적이며 속도 우열·모델 품질 승인과 구별한다. 이 계획의 실행은 아직 미완료다.

## 생성기 근거

실행 명령은 `player-model-spike prepare-segmentation-cohort segmentation-cohort-r1`이다. 실행 EXE `4c0321b0947014a42d0bb9fe6538c0b3b322465e39042e98cab38fcacd03eb7b`의 컴파일 입력171개·EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/4c0321b0947014a42d0bb9fe6538c0b3b322465e39042e98cab38fcacd03eb7b/build-manifest.json)했다. 담당 도구49tests·fmt check·clippy all-targets -D warnings·locked offline build를 통과했고 생성 명령은 exit0이었다. 생성·파일 hash 확인을 실제 정렬 또는 SDK 제품 API 지원으로 표시하지 않는다. [품질 게이트](../../../plans/player/ai-quality-gates.md)를 따른다.
