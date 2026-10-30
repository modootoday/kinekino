---
id: sot:kinekino/demuxer
title: 검토 재생의 입력 분류와 demuxer
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 검토 재생의 입력 분류와 demuxer

로컬 검토 재생은 이미 보유·검증된 ReviewInputIdentity.classification을 사용해 mpv의 강제 demuxer를 선택한다. 파일 확장자·caller MIME·경로 이름으로 선택하지 않는다.

|분류|mpv demuxer|추가 옵션|
|---|---|---|
|IsoBmffHeader|lavf/mov|enable_drefs=0|
|MatroskaHeader|lavf/matroska|MOV 전용 enable_drefs 미사용|

공통으로 protocol_whitelist=file, no-config, scripts/ytdl/자동 외부 파일·자막·playlist 로드 차단과 source/tool guard를 유지한다. 이번 CLI 기능 시험은 vo=null/ao=null로 실제 디코딩·IPC 상태와 제어를 검증하며 화면/오디오 출력의 시각·청각 품질 승인이 아니다. 분류 성공·process 생성·IPC 연결·실제 시간/상태 관측·사용자 품질 판정은 별개다.

R80의 MatroskaHeader 입력에 MOV 인자를 사용한 첫 실패·원 raw는 [검증 기록](../../verification/application/review-host-transport-budget-20261028.md)에 보존한다. R81 수정의 실제 성공은 별도 실행 근거가 있어야 한다.
