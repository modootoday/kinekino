---
id: sot:kinekino/sot-012-media-production
title: 자막·다운로드·스냅샷·숏폼·카드뉴스 설계
status: draft
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 자막·다운로드·스냅샷·숏폼·카드뉴스 설계

2026-10-26 통합 경계와 작업별 인수는 [task 통합 계약](task-integration-contract.md)에서 연결한다. 기존 기능별 정본·부분 구현/실측을 유지하며 전체 제작 완료로 승격하지 않는다.


본문은 2026-10-18 승인된 책임별 재배치에서 아래 파일로 손실 없이 분리했다. 이 문서는 통합 범위의 진입점이며 기능 정책을 중복 정의하지 않는다. 기존 미구현·후보·관측 한계를 각 문서에 유지한다.

- [범위](scope.md)
- [근거와 플랫폼 프리셋](platform-presets.md)
- [자막 저장과 변환](subtitles.md)
- [영상 다운로드](downloads.md)
- [스냅샷과 썸네일](snapshots.md)
- [숏폼: 수동·추천·일괄 모두](shorts.md)
- [YouTube 커뮤니티·Instagram 카드뉴스](cards.md)
- [작업 큐와 책임 경계](job-boundaries.md)
- [검증 게이트](acceptance.md)

화자 분리는 후속 사용자 확정 요구다. 화자 정보의 자막/sidecar 저장·변환 손실은 [화자 도메인](../../domain/subtitles/speaker-contract.md)을 추가 적용하고 [공통 SDK](../api/sdk-contract.md)를 통해 GUI/CLI가 같은 작업을 사용한다.
