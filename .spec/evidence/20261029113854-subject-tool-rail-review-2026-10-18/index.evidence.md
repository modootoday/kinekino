---
id: evidence:kinekino/20261029113854
title: OBJ 캔버스 도구 rail 실제 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# OBJ 캔버스 도구 rail 실제 검토

2026-10-18. OBJ-05/06 부분 개선이며 전체 OBJ 시각 인수는 미완료다.

## 참조와 변경 전

기준 `.spec/artifacts/design/v2/studio/subject.png`의 crop(261,158,70,270)을 기존 Rust visual-review로4배 확대했다. 파생물은 `runtime/design-inspection/subject-tool-rail-r32-4x.png`다. pointer/plus/minus/brush/eraser 세로 배열·어두운 바탕·외곽·pointer 실루엣을 확인했다.

변경 전 `runtime/review-runs/ui-obj-review-20261018-r32/player.exe`, PID101216,1440×900에서 영상 밖 가로 아이콘+텍스트 줄과 사람 모양 선택 glyph를 확인했다. 샘플 이미지·청록 사각 경계만 있고 contour/alpha가 없었다. 첫 탭 전환 순간 spinner와 로딩 후 프레임을 구분했다. 캡션 닫기로 종료했다.

## 변경 책임

공용 toolbar::canvas_unavailable가 contain 영상 rect 내부 좌상단에 토큰 기반 배경/테두리/곡률/여백과 Taffy column,32단위 hit target을 조합한다. selection-pointer.svg를 직접 작성해 Icon::SelectionPointer로 등록했다. 실제 command는 미연결이므로5도구 모두 비활성·미선택이며 이름/사유 tooltip을 제공한다. 요소 기록 계약의 낡은 사용자 종료 문구를 소유 PID 정리 규칙으로 맞췄다.

## 수정 후 실제 관측

`runtime/review-runs/ui-obj-tools-20261018-r33/player.exe`, PID36684. 기존 가상 모니터 위치3500,100. VM/DPI/모니터 구성 변경 없음.

| 표본 | 관측 |
|---|---|
|1440×900 Populated OBJ | 영상 내부 좌상단에5개 세로 glyph와 어두운 rail, 이전 가로줄 제거. 청록 bounds·inspector·filmstrip 유지. disabled 스타일은 시안의 활성 amber pointer와 의도적으로 다름 |
|760×480 Populated OBJ |5도구 모두 영상 안에 보임. 본문은 기존 세로 scroll, inspector는 설정 버튼으로 접근 |
|760 설정 열기→Escape | 패널만 닫히고 OBJ 화면/도구/이미지 유지 |
|760 Empty |5도구는 비활성 유지. 소스 이미지 연결 필요 안내, 샘플 이미지·선택 bounds·샘플 작업진행 제거 |

r33은 caption 닫기 입력으로 종료했다. 화면은 도구 반환 이미지로 관측했으며 실제 캡처 파일 저장을 주장하지 않는다.

## 검사와 남은 작업

cargo fmt --check, cargo test -j2(77통과/8ignored), cargo clippy -j2 --all-targets -- -D warnings, cargo build -j2 --bin player 통과. headless 전체 fixture는 시각 인수를 대체하지 않는다.

미완료: contour/반투명 mask, 보기 전환, typed mask snapshot·선택 track, thumbnail 목록/실패 interval, transport 상세와 모든 caption 상태. rail hover/tooltip·1180/980 표본·실제 편집 command·custom GL callback은 미검증. 비례와 미세 정렬의 후속 비교가 필요하며 완전 일치를 선언하지 않는다.
