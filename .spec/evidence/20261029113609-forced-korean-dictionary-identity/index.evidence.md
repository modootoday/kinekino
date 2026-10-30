---
id: evidence:kinekino/20261029113609
title: 공식 한국어 분절 사전 단어열·점수 맵 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 한국어 분절 사전 단어열·점수 맵 대조

2026-10-18. [원본 tokenizer 실제 대조](forced-segmentation-actual.md)에 사용했던 Rust 포트 사전을 공식 Qwen 고정 Git revision의 원문과 대조했다. **파일 바이트는 다르지만21009개 단어 순서와17968개 고유 단어 집합이 모두 같다.** 공식 processor가 단어별1.0을 부여하는 scores 맵의 사전 입력 동일성은 확인했다. tokenizer 알고리즘 전체·다른 문장·시간 경계·품질 승인은 별개다.

## 실제 입력과 비교

공식 입력은 Qwen3-ASR commit `7c6daf77a2421100f5fb066495372c00129d39ff`의 [korean_dict_jieba.dict](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/inference/assets/korean_dict_jieba.dict)다. Rust HTTP adapter가 고정 URL에서1MiB 상한으로 취득해 원문을 보존했다. 포트는 기존 lumos commit7984967 checkout의 사전이며 읽은 실제 bytes의 고정 SHA를 확인한 뒤 비교했다.

| 항목 | 공식 Git 원문 | 실제 Rust checkout |
|---|---:|---:|
| bytes | 275834 | 296843 |
| CRLF 쌍 | 0 | 21009 |
| 첫 whitespace field 단어 수 | 21009 | 21009 |
| 고유 단어 수 | 17968 | 17968 |
| 상대편에 없는 단어 수 | 0 | 0 |

공식 SHA256은 `cc157a9ab78f92bf11ed98a69b285768d6627cf5cd57a581e6c06693effce37d`, checkout은 `9a3d6881b22b976da41359d74dbe4deaae445521997aa05d6287c540f2b4af65`다. [공식 raw bytes](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/korean-dictionary-identity-r1/official-korean-dict.dict)와 [checkout raw bytes](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/korean-dictionary-identity-r1/port-korean-dict.dict)를 각각 보존했다. byte equality를 주장하거나 어느 쪽을 덮어쓰지 않았다.

Rust domain 비교는 빈 줄을 건너뛰고 각 줄의 첫 whitespace field를 단어로 취해 전체 순서와 set을 각각 비교한다. 빈 사전을 거부하고 차이 수와 최대20개 예시를 보존한다. 원본 Qwen processor의 dictionary scores는 각 단어1.0이며 후속 중복도 같은 값으로 덮어쓰므로 이 실제 set 일치는 그 scores 맵 동일성을 뜻한다. 후속 metadata/frequency field의 동등성을 주장하는 비교가 아니다.

## 실행과 보존

`player-model-spike compare-korean-dictionary korean-dictionary-identity-r1`은 preflight CPU25.09765625%·free RAM34527338496B에서 exit0이었다. 모델·tokenizer를 다시 로드하거나 신경망을 실행하지 않았다. [결과 manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/korean-dictionary-identity-r1/result.json) SHA256은 `e391f7a6cefcbaebc276056131e91237a71e430a718d08063ea2396c9c8c2963`이다. 사전2개는 별도 원문 복사 후 실제 SHA를 대조했고 JSON은 공통 report snapshot에 편입했다.

실행 EXE SHA256 `58858551cb985c75a859c7bf53f2058ccb7501b8de91ff39e699f63b50722997`의 컴파일 입력170개·EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/58858551cb985c75a859c7bf53f2058ccb7501b8de91ff39e699f63b50722997/build-manifest.json)했다. domain의 단어열/점수 맵 판단, infrastructure의 취득·고정 입력 해시·원문 보존, application의 자원 admission·시도 기록을 분리했다. Rust49tests·fmt check·clippy all-targets -D warnings·locked offline build가 통과했다. 신규 회귀는 줄바꿈·중복/순서 차이·실제 다른 단어·빈 입력을 구별한다.

## 중앙 보존 검사와 다음 게이트

Rust spec-maintenance에 고정 결과 manifest와 raw 사전2개의 크기·SHA를 검사하는 별도 evidence 모듈을 추가했다. 새 검사는 결과 JSON 자체의 고정 SHA를 먼저 확인한 뒤 두 정해진 raw 파일을 공통 안전 경로·streaming hash 어댑터로 검증한다. 목록 검사가 단어열 계산을 새로 실행하거나 source→binary 재현 빌드·공식 publisher 서명을 증명하는 것은 아니다. 같은 길이의 raw 변조와 고정 JSON 변경을 거부하는 회귀를 포함해7tests·fmt check·clippy·release build를 통과했다.

[중앙 검사 보고서](../../../evidence/verification/korean-dictionary-integrity-2026-10-18-r1/build-evidence.json)에 `korean_dictionary.source_files_verified=2`를 확인했다. 검사기 [EXE](../../../evidence/verification/korean-dictionary-integrity-2026-10-18-r1/player-spec-maintenance.exe) SHA256은 `a1a3d9875144f4a232e5b48de6a30622ceb46ba4b160d02156a22fcf1a1566e4`이며 [검사 시점 소스](../../../evidence/verification/korean-dictionary-integrity-2026-10-18-r1/source/src/main.rs)·Cargo.toml/lock도 보존했다. 이 소스 복사는 검사기에 내장한 컴파일 attestation이 아니다. 전체 검사는 문서310개·정본 링크 오류0·보존 upstream 오류2, 기존 immutable105개 및 참조 실행기6개/소스 목록292항목/EXE6개의 무결성을 확인했다.

앞선 KO1883에서 실제 원본 LTokenizer19단위와 Rust 결과가 같았다는 근거에 이번 사전 입력 동일성이 더해졌다. 한국어 알고리즘 전체 parity와 품질 승인으로 확대하지 않는다. 일본어의 실제 분절 불일치는 그대로이며 SDK 담당이 frozen source와 구별한 새 Rust 진단 어댑터의 baseline/외부 words A/B 경로를 구현 중이다. 실제 모델 실행과 결과 평가는 root 단일 worker로 진행한다. [잔여 품질 게이트](../../../plans/player/ai-quality-gates.md)를 따른다.
