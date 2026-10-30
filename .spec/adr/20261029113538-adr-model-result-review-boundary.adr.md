---
id: adr:kinekino/20261029113538
title: 실제 모델 결과 검토의 SDK 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 모델 결과 검토의 SDK 경계

상태: 구현 결정 채택. 모델 품질·사용자 표본 판정·자동 적용 승인을 뜻하지 않는다. 2026-10-19.

## 배경

실행 성공·JSON 구조·수치 대조와 의미 품질을 구분했지만 사용자에게 실제 원본/결과와 구체적 검토 방법을 제공하지 못했다. 검토 자료가 없는 상태를 사용자 미승인 차단으로 표현해서는 안 된다. GUI와 AI용 CLI는 같은 application 유스케이스와 domain 정책을 사용해야 한다.

## 결정

2026-10-19 후속 사용자 승인: 첫 제품 채택 범위는 자막 교정·카드 문구·하이라이트의 수동 검토 제안 기능이다. 원문과 비교한 사람의 수정·명시적 채택을 우선 구현한다. 자동 검사 실패를 사용 가능으로 바꾸거나 원문을 자동 덮어쓰지 않는다. 이 우선순위 승인을 모델 결과·자동 적용 승인으로 해석하지 않는다.

- 실제 source/baseline/candidate·모델/엔진 고정 조건·자원·검사·criteria/how_to_review·known issues를 content-addressed package와 assets로 묶는다. 원문 참조는 독립 인간 gold와 구별한다. 텍스트·정적 자산에 검증된 시간이 없으면 range=null을 사용한다.
- 공통 SDK가 inspect·참조 text-asset 읽기·표본 판정 기록·history를 제공한다. GUI가 파일을 직접 읽거나 CLI가 GUI 자동화로 기능을 대신하지 않는다. 텍스트 읽기는 package 참조/bytes/SHA·strict UTF8·64KiB·허용 media 경계를 검사한다.
- 저장 root는 명시적 로컬 시작 옵션으로 조립한다. 브라우저 데이터·package 내용은 저장 root나 실행 프로그램을 선택할 수 없다. backend 미주입은 unsupported이며 가용성을 가장하지 않는다.
- usable/needs_changes/deferred와 note는 특정 package SHA·item·candidate 조건에 묶인 불변 표본 기록이다. 사용자 판정이 없는 상태는 통과가 아니다. 자동 검사 실패·not_run·자료 누락·deferred를 통과시키지 않는다. 자동 적용과 모델 전체 채택은 별도 정책으로 남긴다.
- 배치 전체 시간·peak 메모리와 개별 요청 시간을 분리한다. 서로 다른 초기 자원·캐시·작품 표본의 숫자로 공정한 모델 순위를 만들지 않는다.

## 대안과 결과

정적 Page/표만으로는 실제 자산 비교와 판정 기록을 제공할 수 없었다. GUI나 임시 웹 화면마다 별도 판정 정책을 두면 검증·저장 규칙이 갈라진다. 따라서 임시 읽기 전용 화면은 관측 자료로 유지하고 제품의 판정은 공통 SDK로 연결한다.

해시는 바이트 무결성을 식별하며 인간 판단의 전자서명이나 동일 사용자 공격에 대한 WORM을 증명하지 않는다. 영상·음성·추적·3D 검토는 별도 미디어 계약과 실제 검증이 필요하다. 텍스트9개 성공으로 전체23역할이나 제품 품질을 승인하지 않는다.

## 근거와 후속

- [상위 검토 정책](../../sots/player/application/model-quality-review.md)
- [공통 package·판정 계약](../../sots/player/application/api/model-review-contract.md)
- [실제9표본·CLI 검증](../../sots/verification/ai/text-review-sdk-actual.md)
- [후속 품질 게이트](../../plans/player/ai-quality-gates.md)
