---
id: adr:kinekino/20261029113548
title: 정렬 classifier 관측은 예측 경로와 분리
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 정렬 classifier 관측은 예측 경로와 분리

상태: 로컬 진단에 적용. 제품 정렬·자동 적용 승인은 별도다.

## 결정

길이0 정렬의 점수 민감도를 조사할 때 기존 forward·argmax·시간 보정·최종 결과 구성은 유지하고 결과 구성 후 관측 분기에서 timestamp 위치의 logits만 복사한다. 실제 선택과 최고 다른 class의 점수 차이는 확률로 표시하지 않는다. 대체 class 선택·분절 변경·최소 길이 clamp와 묶지 않는다.

별도 Rust domain 계산과 infrastructure tensor 복사, application DTO 및 root 독립 row replay를 구분한다. EXE·패치·컴파일 입력을 실행 전에 동결하고 과거 보고서와 동일 입력/최종 bit를 확인한 뒤 해석한다. 기존 동결 파일은 소급 교체하지 않는다.

## 결과와 비용

[실제118row 대조](../../sots/verification/ai/forced-head-logits.md)에서 최종 결과 동등성과 전체 row 해시/순위 재계산을 통과했다. 실패 입력2개는 여전히 구조 실패다. 전체 row 보존은 보고서 크기와 메모리·직렬화 시간을 추가하므로 성능 측정과 분리한다. 자원 admission은 [기존 관측 정책](adr-011-correctness-observation-resource-gate.md)을 따르며 실행 중 사용자 앱은 보존한다.
