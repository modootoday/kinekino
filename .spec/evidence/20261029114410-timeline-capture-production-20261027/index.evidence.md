---
id: evidence:kinekino/20261029114410
title: 원본 영상 origin·끝점 capture producer
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 영상 origin·끝점 capture producer

2026-10-27 중앙 PM. 기존 `frame_capture`의 PTS/PNG 검증과 새 공통 FFmpeg 인자 plan을 재사용한다. 기존 SDK capture도 같은 plan을 소비하되 기존 process 경로와 공개 계약은 보존한다. 새 timeline component는 원 Job/deadline/pending·source/video/tool guard를 유지한다. 이것은 개별 관측이며 결합된 Verified 권위가 아니다.

## 입력과 검증

SOURCE `B1FE4AA4BA813E16528F878CFF6978D996D8B6D412A18774023534F44937D109`/2210파일, inventory `B7D8AC44129F6609EE347AC36A0BE86483E6DCF816CEFFAD5EC8C9C20D4C5D6C`, `runtime/verification/source375-timeline-capture-input-r1`. 직전 B1B46C92/2208의190/정상bins와 구별하며 canonical 승격이 아니다.

| 로그 | 실제 결과 |
|---|---|
|477|native-proposal-window/all-targets strict 통과|
|478|시간축 정책22통과/5ignored; 끝점 앞선 관측·첫 적합 tick·범위 밖 거절 포함|
|479|frame_capture6통과/1ignored; 공통 plan·PNG 손상·원 deadline child 정리 포함|
|481|실제 시험 no-run 통과|
|482|실제1/13.82초 통과; 영상·clock·PCM·재생/clip 회귀와 origin/끝점 capture·생성 후 취소|
|483|fmt --check 통과|
|484|verify-current exit0/39373ms·전후2210파일 content/identity 동일·stage 없음|

실제 source time_base는1/12288, 세 PNG 모두1718×720이다.

| 관측 | PTS | PNG SHA256 | 원 child |
|---|---|---|---|
|첫 decoded origin|0|6C2AC82030E7A1A584E85EF3053863287C376A00133F2E9462F480E693BAA38F|PID22648/creation134375287514603833/exit0/비강제|
|17초 첫 source frame|208896|AC5D6D071C716A943852261BAD441A695AA7CB53583340A8A8D6349FF108EED0|PID33368/creation134375287516100764/exit0/비강제|
|25.5초 첫 source frame|313344|BF0E3D8CCFBEC31FE03745FC45521BBAEFF0D69253F594E324643E431B2E360F|PID23240/creation134375287523704930/exit0/비강제|

끝점은 source 처음부터 decode해 select한 frame PTS를 보유한 FFprobe window의 첫 적합 frame과 교차검증한다. origin은 n=0을 선택하며 window보다 뒤의 origin을 거절한다. 임의 호출자 PTS/요약을 권위로 받지 않는다. origin 전용 accessor는 endpoint에서 거절된다.

capture 생성 후 취소는 최초 Cancelled·원 PID35124/creation134375287532985654/exit124/forced=true/Job empty/cleanup_error=null과 observation_published=false를 보존했다. pending owner는 source/tool에 더해 원 video window도 보유한다. 취소를 정상 성공으로 계상하지 않는다.

원 증거는 `runtime/verification/timeline-capture-production-actual-datalab-r1/RETURN.json`, CAPTURE-CANCEL.json 및 component별 INPUT/VIDEO-INPUT/STARTED/PROCESS/RETURN, source375-datalab-b0-r1-logs/477~484다. 도구는 기존 FFmpeg8.1 SHA09948D4CDD0650DA6FF5A87577469F2A218DC2615AE379F8F734D24C49DE0F73, 기존 FFprobe/audioFFmpeg 핀을 사용했다. 새 설치나 PATH 변경은 없다.

## 보존·한계·다음 연결

preimage는 timeline-capture-production-preimage-r1의 PREIMAGE-SHA256.json에 분리했다. 이름이 같은 두 mod.rs는 production/mod.rs와 frame-capture-mod.rs로 구별한다. audio/pcm_lineage/process는 capture 인자 추가만 역적용해 복구한 preimage이며 SHA가 직전 B1B46C92 inventory의 C1F55DDE/E2F90DE7/52340D59 전체 핀과 각각 일치함을 확인했다. 원375·기존WIP·소유권·원 실패·이전 증거는 보존한다.

기존 SDK parser의 음수 source timing 미지원, CPU RGB24의 HDR/ICC/player color parity 미검증을 유지한다. PNG 기술 관측은 GUI current-frame·시안 일치·AI 품질 승인이 아니다. 개별 관측을 함께 보유한 결합 proof·custody/admission·제품 startup/전체CLI·actual9 AI/GUI는 남는다. 최신 전체190/정상bins는 변경 전 B1B46C92 입력이며 이번 B1FE4AA4에서는 미실행이다.

modoo.today `modootoday-free/ai-quality-plus/agent-run-postcondition-and-replay-check` 본문 SHA61843099b7d54c3d7ac226218a5ee1b6ccdbe36ef84268762dbc33a68fbb58b5/worked-example SHA80c9fe8a8a79dd8e08d84b36483d7742f273e55fb2769d2e6156a1111fdd5c11의 원 실행·후조건·주장 분리를 적용했다. QA fixture/code-change 원칙에 따라 합성 경계/기존 SDK 영향/실제 source 관측을 구별했다. 수동GUI·추론·다운로드·WARP·push 보류 유지.
