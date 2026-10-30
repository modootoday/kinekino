---
id: adr:kinekino/20261029113531
title: "ADR-012: 모델 취득과 추론·성능 측정의 자원 게이트 분리"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR-012: 모델 취득과 추론·성능 측정의 자원 게이트 분리

상태: accepted, 2026-10-18. 적용 범위는 root의 고정 HF asset 취득 작업이다.

## 배경과 결정

일반 measure의 preflight는 열린 player 때문에 ready=false다. UI를 유지하라는 사용자 지시를 보존하며 모델 취득을 성능 측정이라고 표현하지 않는다. weights 취득은 모델 로드·추론·GPU 실행과 다른 작업이므로 별도 CPU/RAM 확인 후 고정 catalog·파일별 크기/해시 예산을 적용하는 Rust fetch를 한 프로세스에서 순차 스트리밍한다. 일반 measure·추론의 ready 정책은 이 결정으로 변경하지 않는다.

실제 embedding-acquisition-preflight-r2는 CPU14.174%·여유36,143,280,128bytes·열린player3개를 기록했다. ready=false는 그대로 보존했다. 취득 전 조건은 여유≥12GiB·CPU<60%이며 사용자 앱/OS 설정/메모리 캐시는 변경하지 않는다. root는 동시에 모델 다운로드나 신경망 worker를 추가하지 않는다. 개별 파일은 승인된 budget과 LFS SHA를 통과해야 하며 기존 캐시 파일이 있으면 hash를 확인한다.

## 한계

후속 구현: `fetch-record LABEL KEY FILE...`는 별도 application 모듈이 label 디렉토리를 원자적으로 새로 생성하고 request·실제 preflight·파일별 record·최종 result 또는 failure를 보존한다. 기존 download-KEY.json을 수정하지 않는다. 자원 조건을 실제 검사하며 원래 ready=false도 유지한다. report는 acquisitions의 작은 JSON 기록을 정본 증거 폴더로 복사하며 모델 가중치는 복사하지 않는다. 과거 fetch 기록을 새 방식으로 소급 재작성하지 않는다.

직접 fetch는 원래 metadata/download manifest로 취득 결과를 남긴다. 이 경로에는 measured worker의 latency/peak 기록이 없으므로 다운로드 성능이나 AI 실행 메모리 수치를 만들지 않는다. 취득 완료·graph load·실추론·정렬/검색 품질을 별개로 기록한다. 이 정책은 실행 가능한 파일/스크립트를 Hub에서 받아 임의 실행하거나 미검토 모델·라이선스 게이트를 생략하는 허가가 아니다.
