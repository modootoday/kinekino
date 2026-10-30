---
id: evidence:kinekino/20261029114113
title: 제안 편집 배치·패널 실제 감사 — r165 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제안 편집 배치·패널 실제 감사 — r165 관측

## 후속 focus 수정 checkpoint — 실제 재검증 전

SDK 후속 gate는 workspace156 passed·9 ignored, fmt --all --check·strict all-targets Clippy·bins/examples build가 모두 exit0이다. 새 settings_escape_returns_focus_to_trigger_for_keyboard_reentry 회귀가 통과했다. 고정 focus EXE SHA `cc4b1d3208ab73820dd75435b0a446101622028c3e512f28e1c6c18dab952071`, CLI `357a6c399af2aab709b6ca14deba2fa1ce4cf0c4c700fd2a3678b829b6923456`은 runtime/quality-review/focus-sdk-r1/final-bin에 별도 보존됐다. 실제 GUI 재검증은 아직이며 순수 분리75c8 실행 근거와 구별한다.

SDK가 순수 package 추출의 bin/source를 고정한 뒤 studio/mod.rs 단독 lease를 반환했다. UI가 설정 trigger에 안정 push ID를 부여하고 Escape·drawer 닫기에서 1회 focus 반환을 요청하도록 수정했다. 실제 버튼 response가 그려지는 시점에 request_focus하며 버튼이 없는 넓은 창/검사기 없는 화면에서는 반환 요청을 폐기한다. 다른 application/domain/Cargo나 panel 배치는 바꾸지 않았다.

새 회귀는 stable trigger ID·Escape 뒤 focus·Enter 활성화 response를 검사한다. 단독 rustfmt만 먼저 확인했고 root fmt/test/clippy/build는 SDK 소유 후속 gate로 넘겼다. 아직 새 고정 EXE의 실제 Escape→Enter 재진입이나 초기 drawer 내부 focus 진입을 확인한 것이 아니다. r164·순수 package bin 증거에 이 source 수정을 소급하지 않는다.

## 실행 근거

새 빌드명이 아니라 r164 고정 EXE의 후속 관측 ID다. 실행 SHA·자료는 [문서 원문 검토](document-source-ui-review-2026-10-19.md)와 같다. 메인모니터에서 동일 소유 창54072618을 활성화하여760×760으로 관측했다. 최초 캡처가 다른 전면 앱을 표시했으므로 비교에서 제외하고 player 활성화 후 캡처만 사용했다. VM·DPI 변경은 없다.

## 시안과 현재 구조 차이

기준 `.spec/artifacts/design/v2/studio/cards.png` 전체를 직접 확인했다. 원본 영역(230,110,900,810)의2배 파생물은 `runtime/review-runs/cards-composition-r165-2x.png`다. 원본 수정은 없다.

| 요소 | 기준 이미지 | 현재 고정 GUI·소스 | 판단 |
|---|---|---|---|
| CARD 본문 진입 | 상단 도구 다음 thumbnail rail과 큰 canvas | screens::show가 proposals.show를 cards::show보다 앞에 렌더 | 불일치 |
| 원문·초안 비교 | 우측 검사기의 요약 초안 문맥 | 본문 accordion 기본 열림·원생성 문서·각 입력·cue7개가 수직 누적 | 기능 확인과 별개로 시각 미완료 |
| 저장 접근 | 검사기 하단 선택/전체 저장 | 제안 저장/채택이 긴 cue 목록 뒤 본문 스크롤에 위치 | 760에서 접근 가능하지만 이동 부담 큼 |
| CLIP 후보 작업 | 미리보기·후보와 근거를 중심으로 구성 | 같은 선행 제안 편집기로 preview가 아래로 밀림 | 공통 배치 책임의 문제 |

단순히 접힘을 기본값으로 바꾸는 것은 해결 완료가 아니다. 공통 제안 비교 작업영역을 canvas와 병렬인 검사기/보조 패널 책임으로 옮겨야 한다. 좁은 화면에서는 명시적 비교 패널 진입으로 같은 상태를 유지하고 editor 내부만 스크롤한다. 원문·수정문·저장/채택·오류/receipt가 한 작업영역에 머물며 source와 canvas를 일반 미리보기로 대체하지 않는다. 원생성 raw는 보조 펼침으로 유지한다. 전체 화면군의 공통 shell·tokens·Taffy 경로를 사용한다. 구현 위치와 세부 치수는 후속 설계에서 확정하며 이 기록은 구현 완료가 아니다.

## 실제 패널·키보드

CLIP AI 추천의 저장된 하이라이트 receipt3 상태에서 설정 클릭→전환 완료 캡처를 확인했다. 우측 폭 약300의 설정 패널, 내부 스크롤과 하단 실행 영역이 보이고 기존 본문 위를 가린다. Escape 한 번으로 패널이 닫히며 본문 스크롤 위치와 receipt3이 유지됐다. 이어 Return 한 번은 패널을 다시 열지 않았다. 이것만으로 포커스 반환 결함을 확정하지 않으며, 닫힌 후 trigger focus 여부를 추가 확인해야 한다. hover 상태를 keyboard focus로 간주하지 않는다. 현재 관측은 dirty buffer·최대화·실제 재생·전체 키보드 순환 검증이 아니다.

## 후속 구현 경계

후속 동일 EXE 실제 키보드 관측: 설정 버튼을 다시 클릭하여 연 뒤 Tab 한 번으로 제목줄 브랜드에 amber focus 외곽선이 나타났다. 패널 내부로 자동 진입한 것이 아니다. 그 상태에서 Escape는 설정을 닫고 본문 receipt3을 유지했다. 이어 Tab도 제목줄 브랜드에 외곽선을 표시했다. source의 StudioShell::escape는 settings=false만 수행하며 drawer의 .show에도 trigger 반환/초기 focus 지정이 없다. 비모달 패널 계약에서 초기 진입과 닫기 반환을 명시해야 하는 근거다. 열린 패널 전체에 modal focus trap을 추가해야 한다는 근거로 확대하지 않는다.

공유 packages/SDK source lease 동안 presentation·Cargo·루트 fmt/build를 변경하지 않았다. 배치 개선은 UI 세션이 source lease 조율 후 구현하고 fmt/test/clippy 및 새 고정 EXE 실제 시각·키보드 검증을 수행해야 한다. 기능 연결 성공을 시안 일치로 보고하지 않는다.
