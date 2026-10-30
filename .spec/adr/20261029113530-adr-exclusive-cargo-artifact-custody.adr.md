---
id: adr:kinekino/20261029113530
title: 단독 검증 슬롯에서 Cargo 산출물 직접 사용
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 단독 검증 슬롯에서 Cargo 산출물 직접 사용

- 상태: 2026-10-25 중앙 PM 운영 결정. 다음 제품 통합 검증부터 적용한다.
- 범위: 개발 검증 산출물 소유권과 디스크 예산. 제품 API·기능·품질 기준은 변경하지 않는다.

## 근거

SDK 공간 조사에서 제품 Root/jobs 고정 사본은 약1.842GiB, Meari 7개 target의 고정 사본은 약3.319GiB였다. 전부 새로 복제하면 약5.161GiB이며 target 증가와 정상 bins는 별도다. C 여유16.009GiB에서도 최저12GiB 목표를 지키기 어렵다. 과거 PDB6의 일반 NTFS 압축으로2,441,347,072B를 회수했지만 무조건 복제를 반복하면 같은 문제가 재발한다. 사용자 요건은 정확한 소스·생산자·실행·오류 근거이며 모든 통과 target의 독립 바이너리 사본은 필수 요건이 아니다.

## 결정

새 gate는 Cargo JSON이 직접 가리킨 실제 산출물을 package/target/features/fresh/source pin과 함께 등록한다. 정확 EXE/PDB의 SHA·논리 bytes·파일 ID·mtime을 기록한다. 테스트 그룹의 실제 --list와 exact 실행 중에는 원 target을 독점하며 Cargo·소스 변경·새 writer를 허용하지 않는다. 그룹 전후 content hash와 매 실행 전후 metadata를 확인한다. 전체 시험을 먼저 컴파일하거나, feature 그룹별 컴파일→그 그룹 전체 실행→다음 그룹 순서를 사용한다. 새 Cargo가 기존 산출물을 바꾸면 원 생산자 기록과 새 생산자를 구별한다.

통과 target의 EXE/PDB를 기본적으로 별도 복사하지 않는다. 첫 실패의 원 stdout/stderr/receipt/source를 보존하고, 다음 빌드가 덮어쓸 실패 target의 재현 바이너리는 필요한 정확 파일만 예산 안에서 독립 보관한다. 기존 독립 스냅샷과 보호 목록은 이 결정으로 삭제·이동·하드링크하지 않는다.

그룹 중 산출물 불변을 입증하지 못하면 해당 실행은 유효한 통과 근거가 아니다. hash/fresh 위장, 강제 mtime, fingerprint 조작, 새 target, 직접 rustc로 Cargo를 대체하는 우회는 허용하지 않는다. 시작 여유15GiB·성장 예상량·최저12GiB 예산과 jobs1/CPU2/BelowNormal은 유지한다.

## 한계와 검증

이는 독립 사본을 보유한 것으로 간주하는 결정이 아니다. 통과 바이너리의 장기 보존과 재빌드 동일성은 별개이며 기록에서 구별한다. 이번 정책의 실제 적용 성공은 다음 gate의 원 producer/list/exact/그룹 불변/반환 근거로 검증해야 한다. 이전 SDK88·Meari44 성공을 새 합성본의 성공으로 이전하지 않는다.

연결: [현재 검토 빌드](../../plans/player/current-review-build.md), [자원 예산](../../plans/player/resource-budget.md), [SDK 통합 검토](../../plans/player/root-sdk294-adoption.md#read-only-successor371-regression-gap-review).
