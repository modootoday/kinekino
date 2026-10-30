---
id: adr:kinekino/20261029113546
title: "ADR-011: 실행 중 UI를 보존하는 관측 전용 정렬 진단"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR-011: 실행 중 UI를 보존하는 관측 전용 정렬 진단

상태: accepted, 2026-10-18. 적용 범위는 독립 model-spike 도구이며 제품 실행 정책이나 일반 성능 측정의 승인이 아니다.

## 배경

사용자는 UI 데모를 직접 종료하기 전까지 유지하도록 지시했고 root는 앱을 종료하지 않는다. 기존 일반 측정 preflight는 player.exe가 있으면 대기한다. 실제 raw class→ms→repair 원인을 확인하는 CPU 정렬 진단에는 idle 성능 비교가 필요하지 않아 별도 목적을 명시한다.

## 결정

`observe-alignment`은 검토·동결된 진단 EXE SHA719dc70921c568afd283413966665508e2e7cfb0d80c47143bd98548182c8b89와 align/threads4 요청만 허용한다. 여유 RAM≥12GiB, 유한 CPU 사용률0≤CPU<60%를 모두 요구한다. root 단일 워커·CPU F32·BelowNormal·offline·timeout/프로세스 종료 후 unload를 유지한다. 플레이어를 포함한 사용자 프로세스와 디스플레이·VM·전원 설정을 변경하지 않는다.

일반 `measure`의 ready 게이트는 유지한다. 관측 실행에서는 원래 preflight.ready=false 및 실행 중 앱 목록도 덮어쓰지 않고 보존하며 `measurement_purpose=raw_alignment_correctness_not_performance`를 기록한다. 시간/peak 표본은 실행 이력으로 남기지만 성능 순위·양자화 속도 비교에 사용하지 않는다. 새 도구 버전은 허용 SHA·소스/패치·검사를 다시 검토해야 한다.

## 결과와 한계

최초 JA1634 실제 관측은 CPU58.1055%·여유35,372,134,400bytes에서 실행했고 열린 player를 보존했다. 모델 추론은 완료했으나 정렬 구조 실패 exit3이며 품질 미승인이다. 일반 preflight가 ready였다고 소급 표시하지 않는다. 별도 raw auditor가 동일 입력 baseline의 최종 시간 bit와 실제 head/ms/repair 매핑을 확인하기 전에는 원인을 확정하지 않는다. 이 정책은 다른 모델 다운로드나 일반 benchmark의 앱 감지 제한을 우회하는 기능이 아니다.
