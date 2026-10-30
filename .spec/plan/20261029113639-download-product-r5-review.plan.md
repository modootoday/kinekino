---
id: plan:kinekino/20261029113639
title: 제품 연결 r5 기존 대상 상태 기대 정정
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 연결 r5 기존 대상 상태 기대 정정

PM이 source14 SHA8FC8ADD240E8EFF0AD578F52F65E27EB9A23371AC042DEA9225C899F41A49A4D/raw14 및 r4와의 차이를 확인했다. tests/support/download_cli_host.rs의 Case::Existing 상태 기대 한 줄만 Failed→NeedsInput이다. leaf5ECC1BEDDF23E7B3E37FA393AD8D0039B7BCA6BCD3F3DCC40BF4E99B09CF20BF, 나머지13 불변이다.

기존 media-jobs owned persistence는 RevisionConflict를 needs_input으로 분류한다. actual20의 원 revision_conflict/기존 bytes 보존/publication_started=true/outputNone은 이 정책과 일치하므로 제품 변경 없이 신규 fixture 기대를 정정한다. 같은 오류·비덮어쓰기·원출력·정리 단언은 유지한다. 원20/원BIN/r4/현장과 초기 설계 오류는 보존한다. 확정 P1/P2 발견0이며 수정 후 실제 인수는 아직 미검증이다.

같은 슬롯에서 delta1만 S에 반영해 fmt/strict·새 test BIN/list와 existing exact 양성을 검증한다. 제품 bytes가 같은 배포 CLI 재사용은 producer/hash/FileID·적용 범위를 명시한다. 정상/손상/취소/미입장 기존 결과를 새 BIN 결과로 바꾸지 않는다. 관련 기본·core/doc 회귀와 보호/원 owner 반환을 완료한다. 기존 modoo code-change-review-verification 기준의 정책 대조·원실패·미검증 분리를 적용했다.
