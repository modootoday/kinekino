---
id: evidence:kinekino/20261029114726
title: F1 원 검토 패키지 CaseResolver 연결 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# F1 원 검토 패키지 CaseResolver 연결 검증

2026-10-26. 기존 ReviewStore/ReviewSession의 검증 및 파일 보호를 재사용한다. application/review_source의 ReviewCases<S>는 새 registry나 파일 경로를 받지 않고, 고정 package SHA로 기존 store를 열어 정확한 item.source.sha256과 source_request_sha256의 일치를 확인한다. 후보 자산이나 다른 항목을 원본으로 대체하지 않는다. 성공한 CaseLease가 원 ReviewSession을 소유하므로 기존 mutate의 최종 publication guard까지 패키지/모든 증거 파일과 rooted directory guard가 유지된다. 실제 저장 publication과 결합한 종단 CLI 시험은 아직 아니다.

## 실제 검증

runtime/verification/source375-datalab-b0-r1-logs/109-case-custody-tests: locked/offline/jobs1, native-proposal-window, lib review_source, test-threads1. 17pass/0fail/0ignored/1412filtered, exit0, 70214ms. 기존16에 실제 ReviewFiles를 사용하는 case custody 시험1을 추가했다. 독립 합성 패키지와 증거만 사용하며 실제9/raw/decision을 변경하지 않았다.

새 시험은 정확한 case 조회/재확인, lease 동안 package/source/candidate의 쓰기·삭제 거절, candidate SHA를 source로 제시한 경우 RevisionConflict, 없는 item/package NotFound, path-like item InvalidArgument를 확인했다. 원 request JSON은 존재하지 않는 Z 경로를 포함하지만 해석/접근하지 않는다. lease 해제 후 package 쓰기 handle 재획득과 두 번째 retain 성공을 확인하고, source bytes 변조 후 새 retain이 RevisionConflict로 거절되는 것을 확인했다. 마지막 변조는 합성 fixture에만 적용하며 fixture를 보존한다.

110-case-custody-clippy: -p kinekino-player --all-targets --features native-proposal-window -- -D warnings, exit0/19216ms. 기존 blitz-paint deprecation warning은 원로그에 보존했다. 111-case-custody-fmt: cargo fmt --check exit0/2397ms. CPU2/BelowNormal·C native/tmp·기존 toolchain 유지. root index 비어 있음. Root/Meari lock 및 원 Media source-seal-r2 SHA는 기존 pin과 일치했다.

## 코드와 범위

- src/application/review_source/review_cases.rs SHA256 02EA7F3E42B0E39A5F4E626D6C0868C7CC6E72F9B77E4F6773E4828F9B8DEC9B
- src/infrastructure/review_source/case_tests.rs SHA256 757894C97F6DB66EA4CECF7A4EC011D11C4EFE32701F958D8C3111F6BB830ACF
- application/review_source/mod.rs는 ReviewCases를 등록·export하고 infrastructure/review_source/mod.rs는 test-only case_tests를 등록한다.

스킬: modootoday/software-qa/test-fixture-and-mock-build SKILL.md SHA256 a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100을 재확인했다. 자원 유지·해제 뒤 재획득 및 독립 fixture 원칙을 적용했다. mock만으로 물리 파일 보호를 주장하지 않고 Windows ReviewFiles에 대해 시험했다. 프로세스 경계/전원손실/미디어 decode·PTS 및 사용자 AI 품질은 이 시험 범위가 아니다.

다음은 기존 asset custody의 MediaResolver 연결과 독립 process 복구/경합, SDK/schema/capability/CLI5명령 등록이다. 탐색한 proposals/source_import는 텍스트 snapshot이며 workspace_read/source의 asset path는 custody proof가 아니므로 영상 lease로 승격하지 않는다. download VerificationLease와 VAD GuardedAsset은 별도 수명/고정입력 계약을 갖고 있어 실제 등록 identity 및 범용 재사용 경계를 더 확인해야 한다. 새로운 registry나 raw request 경로 자동 열기를 도입하지 않는다. Verified/실제 AI/GUI 인수 미완료, 기존 보류 유지. 과거 canonical190/정상bins는 현재 추가 소스의 전체 검증이 아니다.
