---
id: evidence:kinekino/20261029114503
title: Review 재생 세션의 공개 SDK·정상 CLI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Review 재생 세션의 공개 SDK·정상 CLI 검증

2026-10-27 중앙 PM. [wire 계약](../../player/application/api/review-playback-contract.md)의 기능 검증이다. modoo.today code-change-review-verification/test-fixture-and-mock-build의 실제/합성·명령 발견/기능 인수 구분을 적용한다. 원375/private overlay·과거 실패/성공·기존 WIP/소유권을 유지한다.

## 고정 입력과 영향 검사

runtime/verification/source375-review-session-wire-input-r1/SOURCE.md SHA256=FECF71FB8644401039237268963E1026002F52F1A847C99D9050ECD973E33936, inventoryAB508804B51AEB2E4A94E6FEC4B00B7741A1230EF901415A561FD83A937BCD0E/2183파일. 331 SDK/세션7시험·332 최종 all-targets/native-proposal-window strict·fmt check·333 actual/discovery no-run 통과 후 입력을 고정했다. builder4종의 타입 전환 뒤에도 같은 session manager가 유지되고 기본 미주입의 명령/schema/capability가 unsupported임을 확인했다.

## 실제 정상 CLI

334 실제 봉인 Sprite + fixed mpv/Job + LocalHost + 정상 CLI의 정확1시험 통과/outer0/26664ms(시험25.90초). 18개의 별도 CLI process에서 capabilities/schema3/open/status/close·동일 key/close 재시도·종료 후 원 session·caller executable 필드/foreign host 거절·열기 직후 닫기·Unverified failure NeedsInput/child 미생성을 확인했다. loop 대기는 원 application의 status로 관측하고 세 명령의 기능 요청과 최종 상태는 실제 CLI 응답으로 대조했다.

runtime/verification/review-session-cli-actual-datalab-r1의 각 cli-* 디렉터리에 request.json/stdout.raw/stderr.raw/process.json을 보존한다. helper는 각 CLI의 종료·supervisor join·비강제·stdout1행/stderr0을 확인했다. 정상 CLI SHA256=9B1F6231C7A5CF17B39617E597F03606E7D726575FB179E190FB63D47ED125D3와 FileID를 guard로 실행 전후 보유했다.

- RETURN.json SHA256=59E47C3DE8A0E9501851529E8EFA3FF0F26758548FE97166666F0E85BFFD5D02.
- CLI-WITNESS.json SHA256=D2827CA13FBC360C6E7DE416B6FE3CB3C47D887461C230FBC91D9C8E0326094A.

정상 첫 session은 exit0/Job Empty/IPC 종료/비강제/first_error 없음이고 terminal status에서 owner_finished/physical_closed를 확인했다. 열기 직후 닫은 두 번째 session은 실제로 process_created=false·physical_closed/owner_finished=true였다. 이 경우의 family_empty=false를 Job Empty로 승격하지 않는다. Unverified 세 번째 session도 child 미생성·원 NeedsInput을 관측했다. async open ok는 owner 생성 수락이고 재생 성공이 아니다.

## 명령 발견

335 기본 CLI discovery 정확1시험 통과/outer0/15009ms(시험14.57초). 고유96명령과 capability/schema 상태 일치를 실제97개 CLI 호출로 확인했다. 원 inventory는 runtime/build-cache/tmp/cli-discovery-16808/inventory.json이다. 새 review playback3은 기본 구성에서 unsupported다. 기존93 발견은 이전 checkpoint로 유지하고96을 전체 기능 성공률의 분모로 쓰지 않는다.

## 통합·남은 범위

같은 입력의336~345에서 [선택190·정상2bins·기존 저장CLI3·전후2183입력동일](source375-review-session-cli-regression-20261027.md)을 확인했다. 이전6381의190/명시bins를 승계하지 않았다. 공개 명령은 Rust 공통 service를 호출하며 GUI 자동화를 사용하지 않는다. 대상은 명시 Sprite nomination이고 실제9/모델 품질/일반 모든 미디어로 확대하지 않는다.

기본 제품 host의 fresh scope/설정 공급·source-bound 재생 제어·재시작/실패/취소 전수·독립 JSON schema validator·전체 CLI/AI/GUI 인수는 미완료다. 수동 GUI·추론·다운로드·WARP·push 보류, heavy1/CPU2/BelowNormal/locked/offline/C cache를 유지했다.
