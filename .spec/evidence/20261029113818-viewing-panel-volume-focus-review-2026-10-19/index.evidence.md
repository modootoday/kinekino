---
id: evidence:kinekino/20261029113818
title: 시청 패널·볼륨 팝업 포커스 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 시청 패널·볼륨 팝업 포커스 검토

2026-10-19. VIEW-07a/08a 후속. 기준은 `v2/viewing/native-player-playback-v2.png`의 (973,910,210,48)4배 확대이며 파생물은 `runtime/review-runs/viewing-volume-r127-4x.png`다. 가는 track·thumb·음량 SVG와 패널/팝업의 입력 계층을 비교한다.

## 발견과 수정

일반 r126 PID73300,1180×760 무소스 시청에서 P 회차 패널 열기→볼륨 팝업 클릭→Escape를 실행했다. 첫 Escape는 팝업만 닫고 회차 패널을 유지했으며 두 번째 Escape는 회차 패널을 닫았다. 다시 볼륨 팝업을 열고 Home을 눌렀으나80%가 유지됐다. 열릴 때 rail에 포커스가 없었다.

공용 VolumeControl은 열기 클릭이 발생하고 기존 팝업이 닫혀 있을 때 내부 rail response에 한 번 request_focus한다. 매 프레임 포커스를 가져오지 않는다. Rail의 기존 Home/End/좌우 처리·focus 외곽선·수치/zero glyph 계약을 유지한다. r126은 caption 닫기로 종료했다.

## 수정 후 실제 실행

r127 PID98728,1180×760 무소스 일반 시청에서 볼륨 팝업을 클릭으로 열면 rail에 amber focus 외곽선이 표시됐다. rail을 따로 클릭하지 않고 Home으로0%,End로100%,왼쪽 방향키로99%가 됐다.0에서는 하단 zero-volume glyph와 비어 있는 track,100/99에서는 채움과 일반 glyph를 확인했다. 실제 음향 출력이나 mute 정책 검증은 아니다.

Tab으로 팝업 rail에서 하단 inline rail로 focus가 이동했다. 팝업이 열린 채 내부 focus를 다시 강제로 가져오지 않았다. Escape로 팝업이 닫혔다. r127은 caption 닫기로 종료했다. 기존 가상 모니터만 사용했으며 VM·DPI·모니터 구성은 변경하지 않았다.

cargo fmt --check와 cargo test, cargo build --bin player 통과. 라이브러리74통과/1ignored와 통합 테스트 통과를 확인했다. 첫 clippy는 동시 SDK 세션의 tests/playback_control.rs field_reassign_with_default로 실패해 담당 세션에 전달했다. 해당 파일은 UI 세션에서 수정하지 않았다. 경고 수정 전달 후 전체 clippy를 재실행한다. 외부 미디어 ignored 검사는 실행하지 않았다.

## 미검증·남은 차이

이번 baseline은 무소스 일반 시청이다. 실제 영상/음량 명령 적용·전체화면·좁은 창·브라우저와의 입력 조합은 별도다. 시안의 amber thumb와 현재 밝은 중성 thumb 색, HWND 합성 때문에 확보하는 popup 여백은 여전히 시각 차이다. 포커스 수정으로 전체 VIEW 시각 일치를 주장하지 않는다.

키보드로 팝업 열기,Escape 후 trigger focus 복귀와 회차가 채워진 상태의 조합은 이번 최종 실행에서 확인하지 않았다.

재검사 결과: cargo fmt --check 및 cargo clippy --all-targets -j 1 -- -D warnings가 통과했다. SDK 세션의 테스트 경고가 해소된 현재 작업 트리를 확인했다.

후속: [r128 토큰 검토](media-rail-thumb-review-2026-10-19.md)에서 활성 thumb 색 차이를 공용 토큰으로 수정하고 실제 inline/popup·0/100을 확인했다. 실제 영상/좁은 창 범위는 여전히 별도다.
