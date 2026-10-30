---
id: evidence:kinekino/20261029113737
title: 제안 3기능 UI 연결 누락 감사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제안 3기능 UI 연결 누락 감사

2026-10-19 r160 작업 중 현행 Rust 소스를 읽어 확인한 결과다. 실제 생성·수정·채택 동작을 검증한 기록이 아니다. 제품 방향은 원문 비교→사람 수정→명시적 채택이며 모델 품질/자동적용 승인과 구분한다. root가 공통 계약 경계를 SDK 감사와 통합한다.

| 기능 | 현재 UI 근거 | 누락 |
|---|---|---|
| 자막 교정 | screens/correction_inspector.rs는 선택 row_id의 original과 DesignSample 한정 correction을 공용 text_comparison에 표시한다. screens/media.rs의 수락/기각은 unavailable_actions다 | 실제 제안 조회, 편집 draft, revision을 검증하는 명시적 채택/기각·receipt |
| 카드 문구 | screens/card_inspector.rs 제목/본문 readonly, 요약 초안 생성 disabled. 카드 artboard와 선택 rail은 존재한다 | 원문/제안 비교, 사용자 수정, 문구별 채택 및 카드 초안 연결 |
| 하이라이트 | screens/clips.rs AI 탭은 DesignSample clip_candidate 목록이다. clip_candidate.rs 동작은 unavailable_grouped다 | 실제 후보/근거 source range, 사용자 범위·문구 수정, 명시적 후보 채택과 편집 초안 연결 |

`application/workspace_read/mod.rs::WorkspaceSnapshot`은 generic table/canvas 등을 제공하며 typed proposal identity/base revision/edit draft/채택 receipt를 제공하지 않는다. `application/api/capabilities.rs::COMMANDS`에 위3기능의 proposal 생성/수정/채택 명령은 없다. subtitles.edit_cues 및 production.export_cards/render_short의 이름만으로 제안 채택 지원을 주장할 수 없다. 기능별 실제 지원 여부와 신규 계약은 SDK 소유 감사로 확정한다.

reviews.*는 sample_only·immutable 품질 판정이며 제품 문서의 편집/채택 저장이 아니다. 이 경계를 재사용하거나 UI에 application 정책·가짜 저장을 만들지 않는다. UI는 정식 capability·source/revision·제안 identity·수정 검증·명시적 채택 결과에 맞춰 연결해야 한다. 현재는 소스 감사이며 신규 동작과 실제 GUI 수정·채택 검증은 미완료다.
