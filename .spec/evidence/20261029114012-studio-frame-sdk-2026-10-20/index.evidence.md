---
id: evidence:kinekino/20261029114012
title: P1a 원 StudioFrame·실제 proposal SDK 소비 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# P1a 원 StudioFrame·실제 proposal SDK 소비 검증

구현 후 영향 검증 중. 원 로그/소스/핀은 `runtime/quality-review/studio-frame-sdk-r1/`, [계약](../../player/presentation/shared/studio-frame-slots.md), [계획](../../../plans/player/presentation/studio-frame-sdk-slots.md)을 따른다. Meari final344 inventory `64027CA55952E4114908B90675810839E20007EFE6AB955234D97457F9B72D74` 직접 재해시 차이0으로 인수했다. source7/caller5/SDK13/main/Cargo/Native344는 동결하며 공통 CSS의 workspace 배치1곳만 PM 허용 후 변경했다.

| 회차 | 원 결과와 범위 | receipt elapsed |
|---|---|---|
| check | root feature-on lib check 성공 |15333ms|
| first | 실제 SDK3 실패: refreshRequired=false인 hidden Refresh를 stale 캡처 대상으로 잘못 활성화. 실제 guard 거부를 보존 |329494ms101|
| layout-before | geometry 시험 Renderer trait import 누락 컴파일 실패. 라이브러리 변경 없이 시험 import 수리 |28905ms101|
| layout-before-r2 | 원 CSS 실제 Native1440 geometry x0/w180, workspace가 nav열에 들어가 실패 |18205ms101|
| frame-r2 | CSS grid-column2 수리 후 geometry3폭 통과. SDK3은 domain pending token을 실제 작업으로 기다려 deadline 실패 |89178ms101|
| frame-r3 | 실제 active transport 조회로 시험 대기 수정, SDK3 통과 |51182ms0|
| frame-r4 | SDK3+foreign/raw/옛generation atomic rollback 통과. recovery1은 legacy close 뒤 receipt field 기대에서 실패 |37336ms101|
| recovery-r5 | 등록 전/후 두 분기의 same prepared Arc·inspect1·same worker 재사용, 명시 typed poll/join 확인 exact1 통과 |30736ms0|
| root-on | product lib371/0/6ignored, workspace lib419/0/6ignored |207062ms0|
| consumer-on | 새 OS fixture의 HostStamp Serialize 가정 컴파일 오류. test suite 실행 전 중단 |342930ms101|
| root-strict | root feature-on all-targets clippy -D warnings 통과 |40400ms0|
| consumer-strict-on | consumer feature-on all-targets clippy -D warnings 통과 |61397ms0|
| consumer-on-r2 | 전체 consumer71통과/12ignored, 신규 SDK5와 layout1 포함 |439027ms0|
| os | 대표 owned View exact1 통과, 아래 CPU capture 한정 |6837ms0|
| consumer-strict-off | standalone source-include 경로에서 PanelHostParts.frame unread 경고로 실패. 원 source 보존 |33222ms101|
| consumer-strict-off-r2 | 공개 ownership transfer의 frame metadata를 다른 public 필드와 같은 공개 표면으로 정리, all-targets strict 통과 |7375ms0|
| consumer-off-tests | bootstrap_preparation/document_factory/native_pending_window_sdk/pending_document_sdk 영향48통과/9ignored |271145ms0|
| root-strict-final | 최종 public metadata source의 feature-on all-targets strict 통과 |19830ms0|
| fmt-root | cargo fmt --check 통과 |1716ms0|
| fmt-consumer | consumer cargo fmt --check 통과 |645ms0|

최종 checkpoint: 필수 영향 검사 완료. `final-source-recheck.json=[]`, `final-owned.json=[]`이며 모든 마지막 runner receipt는 terminal/exit0이다. PM에 단일 heavy 슬롯과 source freeze를 명시 반환한다. 아래 과정 중 문구의 진행 상태는 이 최종 checkpoint로 대체한다. 전체 UI goal은 active로 유지하며 전체13/후속 기능 미완료를 완료 처리하지 않는다.

원 실패를 숨기지 않았다. first는 refreshRequired false의 hidden control 선택 오류이며 제품의 disabled/hidden guard를 제거하지 않았다. 실제 refresh 상호운용에는 real SDK inspect를 실행한 뒤 client seam이 후속 응답 하나를 전달 실패로 바꾸는 시험을 사용한다. save→inspect 실패→명시 Refresh→accept의6명령에서 mutation은 한 번이다. 직접 SDK store 결과와 source/generated/자동적용false를 대조하고 동일 frame의 dirty/focus/useId/DOM 유지·workspace key 교체·옛 callback 거부 및 headless SDK join을 확인했다.

frame-r2 대기는 actual active ticket이 끝나도 domain pending token이 사용자 Refresh 전까지 남는 계약을 오인했다. deadline은 늘리지 않고 기존 session.operation_pending을 bridge의 읽기 getter로 노출해 transport와 domain 복구 대기를 구별했다. 원 SDK 정책·원 버튼 source·Meari344는 변경하지 않았다.

frame-r4 실패 당시 last_close_progress 값의 Debug는 원 로그에 없어 None/기타를 사후 실측으로 단정하지 않는다. 원 source의 legacy Parent.close는 execution_owner.close를 호출하며 poll snapshot field를 채우지 않는다. r5는 동일 owner의 poll_close를 실제 호출하고 Closed(worker_stopped=true/unack0)까지 확인했다. 후속 전체 consumer 회귀에는 legacy close 전후/typed poll 전후 원 Debug를 남겨 회차를 구별한다. Drop/Complete만으로 join 성공을 추정하지 않는다.

원 CSS와 사본·before/after pin을 보존했다. grid-column2만 추가한 뒤1440은x180/w960,1180은x180/w1000,760은x48/w712이며 right overflow0이다. 같은 token/breakpoint/원 StudioFrame을 사용하고 Empty를 fake wrapper/hidden node로 채우지 않았다. full navigation/Dialog는 P1b/P2 미완료이며 해당 전체 조합 시각 인수를 대신하지 않는다.

HostStamp는 권한 타입의 serde 표면이 없으므로 OS metadata는 명시 `*_debug` 필드로 기록하도록 시험만 수정했다. 일반 serde/라이브러리 ABI를 확장하지 않는다. 공개 product core composition helper의 Rust visibility는 canonical/기존 off source-include consumer 간 재사용 경계에 맞췄다. 별도 node registry·raw Content·HTML 이식은 없다.

consumer-on-r2의 실제 Debug는 두 recovery 분기 모두 `after_legacy_close=None`, `after_typed_poll=Some(Closed(...worker_stopped:true,unacknowledged:0))`다. 앞선 r4의 미기록 값을 소급 단정하지 않는다.

대표 OS 증거는 `runtime/quality-review/studio-frame-sdk-r1/os/97896-1792469010555500500/`다. 제출 HostStamp와 캡처 직전 HostStamp는 동일 owner/revision/turn2, viewport1180×760/device_scale1, backend vello-cpu, RGBA SHA `6804c569d9e477a86fe5ab846dc7bc15ba0a24b05a1fb8851c191e519cca2947`다. owned View exact1과 SDK typed join/unack0을 확인했다. CPU DOM raster이며 compositor/물리 입력/전체시각13 검증으로 확대하지 않는다. primary3440×1440 한 대를 관측했고 VM/DPI는 변경하지 않았다.

실제 PNG 전체와 header2×/caption4× 및 기준 `v2/viewing/native-player-studio-v2.png` caption4×를 열어 검토했다. 실제 캡션은 우측126×32 고정/각42×32, 상시 사각 버튼 배경 없이 최소화선·최대화 사각·닫기 대각선으로 대응한다. 현재 세 버튼은 disabled이고 기준보다 낮은 대비이며 실제 최대화/복원·hover/pressed/focus/drag/resize는 미검증이다. 기준은1586×992 생성 시안이므로 픽셀 크기 자체의 동일성을 주장하지 않는다. 실제 proposal 본문은 SDK 필드이고 기준의 영상/좌측 탐색/우측 분석 dock/queue는 이번 Empty 슬롯 범위 밖이다. 화면이 아직 완전한 시안과 다름을 그대로 기록한다.

consumer-on-r2/OS는 PanelHostParts.frame의 pub(super) 시점 결과다. 마지막 수정은 같은 필드의 public visibility뿐이며 동작/스키마 변경이 아니다. 최종 소스에는 off strict와 영향48시험을 적용했고 root strict/fmt를 이어 실행한다. 이전 회차를 최종 바이너리 해시로 소급하지 않는다.

최종 소스10 사본/핀은 `final-source-inventory.json` SHA256 `DE08215D1ABC4CFDE4B00236771D74CABBA3803B15604B16A4F4174B9DA8F426`이다. Meari344/SDK13/source7/caller5 직접 재해시 차이0은 각각 final 차이 JSON에 기록했다. 최종 fmt/재strict/owned[] 인계는 진행 중이다. P1b Dialog·P2 route·기본 native launcher/video/transport는 후속이다. SDK13의 이전 binary/CSS 핀을 새 CSS 또는 frame 실행으로 소급하지 않는다. stage/commit/push0이며 PM의 정확 root index 배정 전 현재 기준선과 미완료 상태를 유지한다.
