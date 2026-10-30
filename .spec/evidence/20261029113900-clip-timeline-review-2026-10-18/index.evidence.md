---
id: evidence:kinekino/20261029113900
title: CLIP3트랙·공용 시간 눈금 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CLIP3트랙·공용 시간 눈금 검토

2026-10-18. CLIP-12 부분 개선이며 전체 숏폼 인수나 실제 편집·추론의 증거가 아니다.

## 참조·변경 전

short.png의 track 영역(272,666,868,240)을 기존 Rust visual-review로2배 확대했다. 파생물 runtime/design-inspection/clip-tracks-r41-2x.png. 공유 ruler, 영상→보라 자막→녹색 audio, 좌측 label rail, 독립 amber playhead와 범위 선택을 확인했다.

r41: runtime/review-runs/ui-clip-review-20261018-r41/player.exe, PID117256,1440×900. CLIP 수동에 영상 track이 없고 흰 파형 아래 일반 구간 timeline이 있었다. waveform playhead는 전체 source duration1420초, 구간 timeline은 별도 표시 길이를 사용해 같은52초 위치가 정렬되지 않는 구조였다. 실제 화면의 미시적 위치만으로 정확한 source PTS를 검증했다고 주장하지 않으며 서로 다른 분모는 소스 관측이다. caption 닫기로 종료했다.

## 변경 책임

timeline_ruler가 µs→x변환·폭에 따른 시간 눈금·amber 선/삼각 표식을 공유한다. SEG/OBJ filmstrip과 CLIP3트랙에 사용한다. view::visible_end가 source duration/마지막 구간/position 기반 공통 표시 범위를 계산하고 source 미연결에는0을 반환한다.0은 실재0초 길이 성공이 아니라 시간 미연결 표시다.

CLIP 수동은 label rail80, ruler32, video64/subtitle32/audio44와 gap4 token으로 세 track을 배치한다. snapshot timeline 구간과 같은 범위/PTS를 사용한다. 자막색·audio색·채움 opacity는 공통 token이다. 영상은 동일 정지 이미지를 반복하고 파형은 수동 fixture임을 명시한다. 실제 디코드·인식 결과가 아니다. 기존 ASR와 수동 파형 fixture 생성도 공유한다.

공용 filmstrip은 body 렌더와 ruler 렌더를 분리하며 Empty에서 작은 소스 연결 문구를12개 반복하지 않고 한 영역으로 안내한다. live에 sample을 주입하지 않는다.

## 수정 후 실제 관측

r42: runtime/review-runs/ui-clip-tracks-20261018-r42/player.exe, PID112604. 기존 가상 모니터 위치3500,100, 단일 검토 player. VM/DPI/모니터 구성 변경 없음.

- 1440×900 CLIP 수동: 영상/자막/audio 순서, 좌측3label, 보라 구간·녹색 파형,52초의 공통 amber선/삼각 표식을 확인했다. 영상과 자막 예시 구간 양끝이 같은 x에 정렬됐다. 눈금은10초 간격이었다.
- 같은 창 SEG: 공용 눈금/재생선과3행 구간 표가 함께 보였다. OBJ: 기존 수동 mask/도구와 시간 눈금/filmstrip이 함께 유지됐다.
- 760×480 CLIP 수동: crop/원본의 세로 적층 아래3트랙에 body scroll로 접근했다. label3개와 파형까지 창 안에 보이고, 폭에 맞게15초 눈금으로 바뀌었다. 같은52초는 세 track을 관통했다.
- 수동 탭 focus→Right로 AI 추천 진입→Left로 수동 복귀: 수동 track scroll 위치가 유지됐다. 추천 탭의 전체 후보 인수는 이번 범위가 아니다.
- 760 Empty: 반복 이미지·자막 blocks·파형·재생선·실제 시간값이 제거되고3label/비어있는 track과 입력 안내가 유지됐다. 소스 안내는 영상 strip 한 영역에만 표시됐다.

caption 닫기로 r42 종료. 화면은 도구 반환 이미지 관측이며 캡처 파일 저장을 주장하지 않는다. 필수 검사: cargo fmt --check, cargo test -j2(78통과/8ignored), cargo clippy -j2 --all-targets -- -D warnings, cargo build -j2 --bin player 통과. 중간 Clippy 숫자 그룹 표기2건을 수정한 후 최종 검사를 통과했다.

## 남은 시각·기능 범위

선택 clip/cue 실제 편집, ruler seek/zoom, track glyph, 시안의 세로 in/out grip, 단일 선택 범위/다중 구간 구분, cue 본문·실제 amplitude와 PTS adapter, crop safe area·handles·subject 유지, 모든 tab/state 실제 검토는 별도다. generic timeline의 예시 구간 label을 실제 자막으로 간주하지 않는다.
