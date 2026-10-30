---
id: plan:kinekino/20261029113523
title: tools 기본 거부 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# tools 기본 거부 경계

## 완료

PM은 root actual의05-final-audit를인수한뒤직렬index소유권으로4줄을적용했다. .gitignore만stage/차이검토후 `f2365796824de011edcc52ac20ef653969eada45`에커밋했다. git diff --check와실제git check-ignore에서공개3dir의src허용·가상미승인tool/직접bootstrap/validation/runtime/.spec거부를확인했다. 기존tracked71파일은유지,파일이동삭제0·무관WIPstage0·push0·indexempty다. 코드변경이아니므로추가Cargo재실행은하지않았다. afterignore SHA BB14A0B3701D48C515250D6BAE312409961638587951BE5C24DB06BDC84D74C7, index309406F87D98854655511489FEF6D4A1C17A96DB2EDDF83347DC43AF69C9FAE0를SDK후속입장에명시인계했다. 아래는적용전계획이력이다.

2026-10-24 읽기 감사에서 현재 `/tools/*/*`만으로는 새 도구 부모와 직접 `tools/*.rs`를 거부하지 못함을 확인했다. 추적된 도구는 model-review-bundle20/player-ui-sdk-bridge31/resource-runner20의71파일이며 PM도 git ls-files로 대조했다. 미추적24디렉터리/직접bootstrap3은 내용·권원·제품필수성 미검토 상태다. 이름만으로 비밀이나 배포 필수성을 단정하지 않는다.

SDK 실제 시험의 최종 보호 감사/슬롯 반환 후 PM 직렬 작업으로 기존 tools 블록의 `!/tools/` 뒤에 `/tools/*`를 두고 위 세 책임 디렉터리만 허용한다. 기존 Cargo/README/src/tests 하위 규칙과 runtime/scripts/secrets/target 제외는 유지한다. 경로별 파일 예외를 늘리거나 tools 전체를 공개하지 않는다. 현재 .gitignore before SHA9164E39C7C38735CC7AE58F560F1A4F86654670CF154E535CB567337C96B0134.

적용 뒤 git check-ignore의 실제 비공개 파일/가상 새 도구·공개 세 디렉터리 source/test·runtime 경계를 검증하고 .gitignore 한 파일만 명시 stage/독립 커밋한다. 기존 미추적 파일을 삭제/이동/stage하지 않는다. ignore는 추적 파일을 비공개로 돌리거나 이력을 지우는 기능이 아니다. source/compiler/runtime 검증 성공으로 계상하지 않는다. Card 공개 패키지 허용은 별도 기능 채택 diff로 검토한다.

이후 bootstrap3의 책임별 private 경로 이관은 소비 경로 검증과 담당 배정이 선행하며 여기서는 이동0이다. 신규 공개 도구는 제품 필수성·재배포 권한·민감정보/fixture 검토 뒤 디렉터리 단위로 허용한다. 현재 계획은 적용 전이며 SDK 보호 핀/기존 작업을 변경하지 않았다.
