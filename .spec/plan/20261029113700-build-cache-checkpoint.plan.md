---
id: plan:kinekino/20261029113700
title: 채택된 체크포인트의 지속 빌드 기준
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 채택된 체크포인트의 지속 빌드 기준

## 2026-10-24 현재 배정

Card 정본47은 `371ed79414698e49f6ee7444ec00d5e84a0cd1cc`로 커밋됐고 canonical integration의 full1163/0/76·strict·feature mapper1/parity2 및 원복 감사를 인수했다. 이 조건을 충족했으므로 SDK에 동일 S/T의 source61+lock62만 검증된 정본으로 한 번 승격하도록 배정했다. 이전 S기존22/부재40과 lock 복구본을 독립 보존하고 동일 내용은 쓰지 않는다.

이번 승격은 Cargo0·빌드0·alias4/worker2 변경0·root/index0이다. 현재 alias는 이전 원본 실행 파일인 점을 명시하며 최신 제품 바이너리로 승격하지 않는다. 기존 carry229/cache11과 알려진 예외를 보존한다. 새 source baseline·변경 경로·복구 경로·실제 해시 및 writer 반환을 확인한 뒤 UI J4 r2 own4만 delta로 검증한다. 절감 효과는 아직 측정되지 않았다.

## 선행 연구와 전환 원칙

현재 r3 실행은 변경하지 않는다. 다음 canonical source 채택·검증 완료 뒤에만 PM이 SDK 소유 S/T의 기준 전환을 별도 배정한다. 미채택 후보를 상주시키거나 보호 감사를 생략하는 계획이 아니다.

읽기 연구에서 r2 restored62와 r3 preimage62가 같은 오래된 원본이고 r3 supply62에서 실제 내용까지 왕복함을 확인했다(proposals/mod D2AB→195D, output_review/schema660D→FA57). source 내용·mtime·alias 각각의 재컴파일 기여 시간은 분리 측정되지 않았으며 alias를 단독 원인으로 단정하지 않는다. 현재 r3 worker2는 내용/mtime 불변·FI1 차이만 관측됐으므로 r2의 재컴파일 관측과 혼합하지 않는다.

전환 시 원 source/원부재·lock·alias4·worker2의 SHA/bytes/mtime/FI 및 raw를 독립 보존한다. 기존 고정 EXE/PDB와 Cargo producer 근거도 유지한다. 같은 S/T에서 채택된 source/lock을 한 번 정합화하며 같은 내용의 파일은 쓰지 않는다. 실제 Cargo source/features/profile/lock과 귀속된 정상 BIN/build-script만 새 보호 대상으로 인수하고 이전 버전은 독립 보존본으로 남긴다. 이후 gate는 이 새 기준에 대한 최소 delta만 공급·원복한다.

PM 직렬 writer·소유권·기존 미완료 예외와 실제 시험/EOF/owner/실패 증거는 유지한다. cold target·cache 삭제·mtime 역조작·fingerprint 편집·동시 writer·증거 축소는 하지 않는다. 절감 효과는 다음 동일 범위 gate의 fresh 기록과 elapsed로 측정하며 추정 시간을 완료 실적으로 보고하지 않는다.

근거는 읽기 전용 cpu_budget_research의 r2 `17-S-restored62.json`, r3 `02-S-preimage62.json`/`02-supply62.json` 대조다. 구현·기준 전환·성능 실측은 아직 미수행이다.
