---
id: evidence:kinekino/20261029113608
title: DER 대조 어댑터 JSON 정밀도 회귀 수정
status: active
created: 2026-10-29
updated: 2026-10-29
---
# DER 대조 어댑터 JSON 정밀도 회귀 수정

2026-10-18. 동일 녹음60~120초 FP32 report를 frozen aed71a64 adapter에 입력하면 원본 scorer를 시작하기 전 `original report and same-input Rust re-evaluation differ`로 거부됐다. INT8 report는 같은 경로를 통과했다. 원본 모델 출력·5202 보고서는 변경하지 않았다.

## 원인과 수정

실제 저장 DER `15.044017424975799`를 기본 serde_json1.0.154로 읽으면 binary64 bits가 `4624658646963417932`에서 `4624658646963417933`으로1ULP 이동했다. 저장→읽기→bits 대조 회귀 테스트가 수정 전 실제 실패했다. 정수 시간 components의 불일치가 아니라 JSON 소수 파싱의 왕복 정밀도 문제였다.

새 어댑터 Cargo dependency에 `serde_json`의 `float_roundtrip` feature를 켰다. 기존 strict same-input 동등성, 정수 components·SHA·원본score 허용오차·reference 처리·원문 script·Perl 옵션을 변경하지 않았다. tolerance를 넓히거나 보고서를 반올림하지 않는다. 해당 score와 기존2개의 실제 score를 binary64 bits까지 왕복 검사하는 회귀 테스트가 수정 후 통과했다. 추가 runtime·Python·Perl 코드는 없다.

## 검증과 보존

Rust 어댑터10tests(기존9+정밀도 회귀1)·fmt check·Clippy all-targets-Dwarnings·locked offline build가 통과했다. 새 EXE SHA256 `136a5773fcc0ffacebf39c37a0eb418c471f3a0972cc7209d2fea7168c8ee8fe`, manifest SHA256 `62d556fc5f769d414d810ab6fc57fa0e591dac0f94523fcd121e237ac2841eba`를 실제 재시도 전에 보존했다.41개 source/test/manifest/lock 입력과 EXE·compiled-source를 합친43파일을 verify-build로 확인했다. 이전 aed bundle과5202 evaluator는 불변으로 유지했다.

새 frozen EXE가 동일 FP32 request를 새 output `nist-holdout-fp32-r2`로 실행해 원본scorer exit0·stderr empty·timeoutfalse·capfalse, DER15.04 인쇄/정수4components차이0us로 통과했다. 실패한 r1은 scorer 실행0회·결과 디렉터리 생성 전이었고 request 원본을 남겼다. 첫 shell 묶음에서 FP32 request 복사가 없는 디렉터리 때문에 실패했지만 INT8 scorer는 이미 성공했으며, INT8 request는 이후 별도로 보존했다. 이 오류를 INT8 모델/scorer 실패로 합산하지 않는다.

실제 입력·결과와 해석은 [시간 분리 검증](diarization-temporal-holdout.md), scorer 범위는 [DER 실제 대조](diarization-der-reference-actual.md)를 따른다. 수정은 검증 도구의 수치 보존을 해결했으며 모델 정확도 향상이나 JER parity 증거는 아니다.
