---
id: sot:kinekino/potplayer-ui
title: PotPlayer 기반 UI 검토 제안
status: draft
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# PotPlayer 기반 UI 검토 제안

2026-10-18 후속 지시: Pages 추가 갱신을 중단하고 로컬 정본으로 검토를 이어간다. 아래 Page 저장·갱신 흐름은 이전 작업 이력이다. 문서 재배치 승인은 상세 UI 승인과 구별한다.

후속: 사용자 요청으로 병합 이미지를 독립 화면 6개로 교체하고 AI 구간 스킵·제작 작업 공간·기능별 세부 기준을 추가했다. [상세 제안](sot-019-detailed-ui-ai-proposal.md)이 최신 검토 내용이다. D1~D11은 사용자 검토 대기다.

Pages 검토본(비공개 Page)에 시작·재생/회차·탐색·전체화면 시안, 결정 D1~D5, 조작 목표, 접근성·DPI 게이트, Rust 아이콘 후보를 저장했다. 사용자 요청으로 추가 제품 구현을 보류한다. 검토 답변을 Page에 기록한 뒤 정본과 구현을 갱신한다.

modoo.today에서 design-approval을 검색해 읽고 적용했다. HTML 중심 design-specimen-review는 Rust 통일 규칙에 맞지 않아 채택하지 않았다. Pages write-page, frontend-design, rust-pro도 적용했다. 사용자가 이미 네이티브 기본·단일 창·PotPlayer 참고를 지정했고 이번 결정 목록은 그 안의 구체적 배치 제안이다.

[비트맵 시안](../../../../artifacts/design/archive/viewing/native-player-design-v1.png)은 내장 ImageGen으로 생성하고 직접 열어 확인한 배치 참고 이미지다. 주요 프롬프트: Windows Rust native player, compact PotPlayer inspired, four panels START / PLAYBACK + EPISODES / DISCOVER / FULLSCREEN, dark neutral chrome, restrained amber, consistent Phosphor Regular style icons, sample data. 실제 라이브러리 자산과 치수의 검증 결과가 아니다. 이미지의 카드 탐색은 장기 Rust 카탈로그 방향이고 초기 웹 보조 모드의 구현 사실이 아니다. Settings는 이번 범위에서 제외하며 회차 길이·썸네일은 실제 데이터 없으면 표시하지 않는다. 영문 문구는 예시이며 제품 한국어 레이블을 사용한다. 탐색선은 하단 컨트롤 위 전폭 배치를 우선 검토한다.

아이콘은 egui-phosphor와 egui-lucide 공식 문서를 열어 비교했다. Phosphor Regular 우선 검증 제안이며 현재 egui 0.33 호환 버전, 실행 파일 증가량·폰트/텍스처 메모리·DPI는 미측정이다. 경량성 확정으로 표현하지 않는다. 아이콘·접근성 이름은 presentation 전용 모듈 책임이며 재생 정책이나 OS 호출을 넣지 않는다.

검사: 중단 시점 WIP에 cargo fmt 적용 후 fmt --check 통과. cargo test 21개 통과. Clippy -D warnings는 미연결 title_bar/control_icon의 dead_code와 float literal 타입 경고로 실패했다. 이를 숨기거나 검사 통과로 표시하지 않는다. 디자인 승인 후 실제 아이콘 라이브러리로 연결/대체하며 해소할 작업이다. 새 UI 실행·렌더링·접근성·DPI는 미검증이며 기존 UI 결과를 승계하지 않는다. Page 저장과 이미지 접근 확인을 완료했으나 Pages 자체 렌더링은 별도 확인이 필요하다.
