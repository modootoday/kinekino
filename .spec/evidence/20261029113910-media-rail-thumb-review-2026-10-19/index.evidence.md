---
id: evidence:kinekino/20261029113910
title: MediaRail 손잡이 토큰 시각 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# MediaRail 손잡이 토큰 시각 검토

2026-10-19. VIEW-07a 후속. 기준 `v2/viewing/native-player-playback-v2.png`의 (973,910,210,48)4배 확대를 다시 확인했다. 파생물 `runtime/review-runs/viewing-volume-r127-4x.png`. 시안의 볼륨 thumb는 track 채움과 같은 amber이며 r127 실제 화면은 밝은 중성 thumb를 사용했다.

## 수정

공용 MediaRail의 track/채움/활성 thumb/비활성 색을 의미 토큰으로 분리했다. 활성 thumb는 amber, 비활성은 중성 SECONDARY를 사용한다. 기존 track3·반경5/hover6·compact18·hit32·inset6을 공용 토큰으로 옮기며 수치·입력 정책은 유지했다. 포커스는 별도 외곽선이다. 시안 raster의 광택/비정형 가장자리는 복제하지 않는다.

## 실행 관측

r128 PID77148,1180×760 무소스 일반 시청에서 inline80%와 팝업80%의 원형 손잡이가 track 채움과 같은 amber로 표시됐다. 팝업 초기 focus 외곽선은 별도로 보였다. Home0에서 왼쪽 끝 amber thumb·비어 있는 track·zero glyph,End100에서 오른쪽 끝 thumb·전체 채움·일반 음량 glyph를 확인했다. 상단 전폭 미상 seek의 손잡이는 비활성 중성색으로 유지됐다. Escape로 팝업이 닫힌 뒤 inline100% 표시가 유지됐다.

caption 닫기로 r128을 종료했다. 기존 가상 모니터만 사용하고 VM·DPI·모니터 구성은 변경하지 않았다. cargo fmt --check, cargo test -j 1, cargo clippy --all-targets -j 1 -- -D warnings, cargo build --bin player -j 1 모두 통과했다. 외부 미디어 ignored 검사는 실행하지 않았다.

## 남은 범위

실제 재생 seek·좁은 창·다른 모든 MediaRail 소비자의 상태별 인수는 별도다. 볼륨0과 mute는 별도 의미를 유지한다. HWND 합성과 popup 확보 여백은 이 변경 범위가 아니다.
