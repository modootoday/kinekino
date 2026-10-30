---
id: evidence:kinekino/20261029114834
title: F1 기존 package media custody와 실제 저장 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# F1 기존 package media custody와 실제 저장 연결

2026-10-26. ReviewPackageMedia<S>를 추가했다. 이는 기존 ReviewStore가 검증·보호하는 package asset의 MediaResolver 어댑터다. 원 package/item/source request membership은 ReviewCases와 같은 retain_case를 사용한다. 별도 registry나 요청 내부 raw path 접근을 만들지 않는다. 다른 보관소의 자산 지원과 실제9의 원미디어 등록/연결은 여전히 전체 요구사항에 남으며 이 어댑터로 대체하지 않는다.

asset_id가 해당 package에 실제 포함되고 SHA가 일치해야 한다. 기존 ReviewStore는 bytes/SHA와 전체 asset 파일 guard를 검증·유지한다. 선언 media_type의 명시 audio/video 목록만 association 입력으로 받는다. JSON/text/알 수 없는 형식을 영상으로 추정하지 않는다. MIME admission은 decode 성공이 아니며 synthetic bytes도 이 검증 범위에서 content identity만 증명한다. verified_duration_us=None, timeline Unverified/playback_ready=false를 유지한다. 실제 PTS/seek/capture/export·모델 품질 승인으로 승격하지 않는다.

## 실제 결과

runtime/verification/source375-datalab-b0-r1-logs/112-media-custody-tests: 17pass/0fail/0ignored/1412filtered, exit0/64316ms. 앞선109의 시험 개수를 늘린 것이 아니라 실제 custody 시험1의 assertion을 확장했다. 기존 합성 package에 고정 bytes/해시/선언 video asset을 추가했다. 정확한 identity/unknown duration, 보호 중 write/delete 거절, SHA 불일치 RevisionConflict, request JSON의 미디어 오인 Unsupported, foreign asset NotFound, lease 해제 후 쓰기 핸들 재획득·재retain을 확인했다.

같은 실제 ReviewFiles case/media 어댑터와 별도 UserReviewFiles를 application mutate에 연결했다. control checkpoint에서 원 video의 쓰기 거절을 관측하고 publication 후 receipt/저장 association 일치, inspect 성공, 동일 key 재조회에서 같은 receipt와 control 재호출0, 작업 종료 후 파일 handle 해제를 확인했다. 이는 같은 프로세스의 실제 파일 SDK usecase 시험이며 CLI/IPC·별도 프로세스·실제 미디어 decode 시험이 아니다. fixture는 C tmp에 보존한다. 원 실제9/package/raw/decision은 미사용·불변이다.

113-media-custody-clippy: -p kinekino-player --all-targets --features native-proposal-window -- -D warnings exit0/18303ms. 기존 paint deprecated warning은 원로그에 보존했다.114-media-custody-fmt exit0/2373ms. locked/offline/jobs1/CPU2/BelowNormal/C native/tmp, root index0, Root/Meari lock 및 원 seal SHA 불변.

## 코드 pin

| 파일 | SHA256 |
|---|---|
| src/application/review_source/review_cases.rs | 4A754E80C60F45BC5F315D50EBD68F099689CE0AEC829026C34062D654DC1DBB |
| src/application/review_source/review_media.rs | 09776E8A2E492689AE3EEF0B6F758FDB6DD93F5CAC39423E16F170640B7A54E6 |
| src/infrastructure/review_source/case_tests.rs | D8307AA7599AB194283D97B7FD11B3319F91FA2EF681C8A3974C29F302F9B4EC |

스킬은 앞선 [case 검증](review-source-case-custody-20261026.md)의 modootoday/software-qa/test-fixture-and-mock-build 본문/해시·실자원 release/reacquire 원칙을 이어 적용했다. 기존 JobFiles SourceBlobStore는 bounded UTF8 snapshot이며 영상 registry로 재사용하지 않았다. VAD GuardedAsset은 고정 profile/audio 입력에 속한 별도 pin/수명 계약이므로 범용 미디어 등록 완료로 표시하지 않는다.

다음은 공통 DTO/schema/capability/SDK/CLI5명령 연결과 실제 process 경계 검증이다. 별도 media custody 등록·Verified timeline 전이·실제 AI/GUI 인수도 미완료다. 원190/정상bins 기준본 통과를 현재 추가 소스의 전체 회귀 성공으로 승계하지 않는다. WIP/소유권·원증거·추론/다운로드/수동GUI/WARP/push 보류 유지.
