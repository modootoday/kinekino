---
id: evidence:kinekino/20261029114117
title: PROJ 파일 목록 키보드·참조 구분 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# PROJ 파일 목록 키보드·참조 구분 검토

2026-10-18. 대상 패키지 파일 선택과 원본 참조. 기존 시안 확대 근거는 [PROJ 요소 계약](../../player/presentation/studio/projection-visual.md)에 있다. 이번 변경은 전체 scene/패키지 시각 인수가 아니다.

## 수정 전 실제 관측

r71(PID103004,1440×900, `runtime/review-runs/ui-projection-files-20261018-r71/player.exe`)에서 깊이 탭 scene/XYZ/grid와 공통 미상 scalar가 표시됐다. 패키지 탭에서 mask.png를 클릭한 뒤 Down을 눌러도 선택이 mask.png에 머물렀다. 파일 목록 아래 이미지는 파일 선택과 관계없이 같은 원본을 표시했다. 소유 창 닫기 후 player/mpv 조회는 비어 있었다.

## 수정

기존 다섯 샘플 파일을 stable ID로 유지하고 최신 Taffy response를 index별로 보존해 공용 keyboard_selection을 연결했다. 클릭은 해당 항목에 focus를 주며 Up/Down/Home/End가 같은 목록을 이동한다. 선택 파일명·파일 미리보기 연결 필요를 명시하고 이미지에는 원본 프레임(참고)을 붙였다. 소스가 없는 상태에서는 이전 선택 파일명을 표시하지 않는다. 실제 패키지 파일/preview/type/revision 계약은 후속이다.

## 수정본 실행

r72(PID4588, `runtime/review-runs/ui-projection-files-20261018-r72/player.exe`)에서 확인했다.

- 1440×900: mask.png 클릭→Down은 depth.png의 선택/focus와 선택 파일명 안내로 이동했다. End는 마지막 scene.glb를 선택했다. 원본 프레임(참고) 이미지와 파일 미리보기 연결 필요가 구분됐다.
- 760×480: scene 아래로 적층된 목록까지 스크롤해 항목 클릭→Home으로 첫 texture.png를 선택했다. 설정 열기→Escape 후 같은 파일 선택과 목록 위치를 보존했다. 공통 미상 상대 깊이 track과 고정 검증/저장 버튼도 표시됐다.
- 760 Empty: 파일 목록·이전 선택 파일명·원본 이미지가 사라지고 패키지 입력 연결 필요/소스 이미지 연결 필요가 표시됐다. 원본 참조 heading과 scene/grid 골격은 남는다. 파일이 없어졌다고 가짜 선택/검증 완료를 만들지 않았다.
- 최종 fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드 통과. 마지막 Empty 표시 guard를 추가한 뒤 fmt와 전체 test를 다시 확인했다. headless 전체 fixture 통과는 모든 화면의 수동 시각 인수와 별개다.

소유 r72 창 닫기로 종료한 뒤 player.exe/mpv.exe 조회 결과는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다.

## 남은 범위

Up·LongLabels/NeedsInput·980/1180·닫힌 scene subtree의 focus 복구는 이번 실제 표본 밖이다. 실제 파일별 thumbnail/manifest/depth/mask 데이터·파일 검사·scene orbit·태그/아이콘·전체 패키지 스키마는 후속이다. 같은 원본 참조를 실제 파일별 preview로 표시하지 않는다. 안내 추가로 길어진 내용의 상세 여백과 검증 표 전체 접근은 별도 인수 대상이다.
