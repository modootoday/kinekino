---
id: sot:kinekino/viewing-preferences-and-resume
title: 시청 설정과 이어보기 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 시청 설정과 이어보기 계약

2026-10-26 설계 기준선. SDK는 정책·저장·API/CLI, UI는 임시 편집/선택·창 표시, infrastructure는 원 source/engine와 저장 기술을 소유한다. 현재 조사에서 ‘resume’ 함수 이름 대부분은 callback/스레드 재개였고 영속 이어보기 지원을 입증하지 않았다. 이 문서는 새 제품 기능 지원 선언이 아니다. 기존 현재 회차·volume/mute/pause intent·패널·이전 창 bounds 요구를 구체화하며 별도 JS 설정 저장소를 만들지 않는다.

## 독립 값과 저장 범위

| 값 | 책임과 내용 | 저장하지 않는 것 |
|---|---|---|
| ViewingPreferences | schema_version·stable product ID·revision, volume_percent[0,100]·muted·pause_intent·episode panel preference | 엔진 관측인 것처럼 보이는 가짜 duration/position, source URL/credential |
| WindowPreferences | 정상 창의 논리 bounds와 저장 당시 display/DPR metadata·fullscreen 이전 bounds | HWND/native handle·foreground 권한·다른 프로세스 identity |
| ResumePoint | stable source/media/work/episode identity·source revision/fingerprint·선택 track ID·observed engine position_us·known duration optional·관측 timestamp·revision | signed URL/cookie/프로세스/engine_instance_id/lease를 재사용 가능한 권한으로 저장하지 않음 |
| PresentationState | route/selection·scroll/focus·볼륨/seek 편집버퍼·열린 popup·dirty prompt | SDK 원본 raw/draft/store의 복제본 또는 실행 권한 |

preferences와 resume은 별도 revision이며 project revision·media generation과 혼합하지 않는다. JSON 정수는 DecimalU64 정책, bool/enum·unknown field·한도를 strict 검증한다. volume은 추정 실제 음량이 아니라 마지막 사용자 의도다. 저장값을 적용한 뒤 엔진이 거절/미관측이면 실제 상태와 구분해 표시한다. private preference root는 제품 stable ID로 조립하며 표시명 변경은 [migration](../architecture/release-and-trust-boundaries.md)을 따른다.

## 기록과 재개 순서

1. 관측 publisher는 현재 source/engine generation에 속한 position만 기록 후보로 제공한다. control accepted·UI 드래그 예상시각·decoded frame PTS로 resume position을 만들지 않는다. 명시 checkpoint/일시정지 관측·회차 전환 전·정상 close에서 coalesced 저장을 요청한다. 매프레임 파일 쓰기 금지, 실패 시 재생을 강제로 종료하거나 새 무한 retry timer를 만들지 않는다.
2. application은 current source identity와 revision CAS로 저장한다. stale engine event·역순응답·회차 전환 뒤 옛 position은 최신 resume을 덮지 못한다. 자동복구로 conflict를 무시하지 않는다. 저장 실패는 관측·사용자 의도를 유지하고 경고/명시 재시도 경로를 제공한다.
3. 재개는 사용자 선택한 원 episode/source ID를 통해 정식 resolver를 다시 호출한다. source/track/fingerprint 일치와 새로운 engine readiness를 확인한다. 이전 URL이나 engine authority를 복구하지 않는다. 일치 불명·만료·로컬파일 소실은 NeedsInput/SourceExpired이며 다른 회차·첫 항목·새 파일로 암묵 대체하지 않는다.
4. 검증된 새 duration이 있으면 saved position이 범위 내인지 검사한다. 마지막 위치가 duration 이상이거나 time mapping이 달라지면 사용자에게 처음부터/다시선택을 안내하고 조용히 clamp/seek 성공을 만들지 않는다. 엔진이 seek를 지원하지 않으면 Unsupported를 표시하고 저장된 resume 기록은 유지한다.
5. 새 source의 명시 open→관측 ready→seek 접수→같은 engine의 실제 위치 관측 순으로 진행한다. 정확 frame PTS·sample 정확도나 청취 품질은 이 근거가 아니다. resume intent가 paused면 paused 상태를 유지한다. 자동 다음 회차는 별도 정책/명시 source이며 사용자 동의 없이 이어보기 기록으로 활성화하지 않는다.

## 계획 SDK 표면·저장

현재 미등록 계획 표면은 viewing.preferences.inspect/save, viewing.resume.inspect/save, viewing.resume.prepare다. inspect는 무부작용, save는 expected revision(CAS)과 원 source generation 검증, prepare는 새로운 source 해결 계획을 반환하며 저장 URL/엔진을 자동 실행하지 않는다. 실제 open/seek는 기존 playback API와 같은 application coordinator를 사용한다. 하나의 거대 새 playback SDK나 GUI-only 기능을 만들지 않는다.

첫 save create는 expected 없음, update는 양수 expected revision 필수. idempotency는 동일 context key+정규화 입력으로 같은 receipt, 변경 입력은 RevisionConflict이며 기존 다른 명령의 미지원 context를 변경하지 않는다. 저장 root가 없으면 Unsupported, source 미지정/불명이면 NeedsInput, 손상/schema-newer이면 보존 후 NeedsInput/Unsupported다. no-overwrite immutable revision과 원 writer reservation·publication primitive를 재사용하고 완료 record/replay receipt를 한 envelope로 저장한다. publication 전 취소 no-write, 이후 응답 유실은 [실행/저장](execution-storage-boundaries.md)의 unknown 조회 계약을 따른다. strong cancel 미지원은 명시한다.

창 bounds 복구는 현재 사용 가능한 work area에 보이는 범위로 맞추되 원 논리값과 적용 보정값을 구분한다. 없는 display ID/달라진 DPI는 정상 fallback 위치 안내이며 display 설정/VM/DPI 변경은 하지 않는다. fullscreen 상태 복구가 OS capability를 주지 않으며 창/문서 생성 전 native 권한을 합성하지 않는다. 기능 인수는 SDK/CLI, 실제 창 배치·focus·접근성은 UI 고유 인수다.

## 인수 기준

VP01 preferences validate/strict schema/volume-mute 분리. VP02 resume 기록의 실제 engine 관측과 generation·역순/stale 거절. VP03 CAS/같은key replay·충돌/재열기/프로세스 중단·손상/권한/용량. VP04 URL/쿠키·원 engine 권한 비영속과 source 재해석 identity 불일치. VP05 paused/seek unsupported/duration unknown/경계·source 만료/없는파일. VP06 사용자 선택→새 ready→seek→원 engine 관측·정리. VP07 display 변화/bounds 가시성·fullscreen 전 상태와 UI focus. VP08 GUI/CLI 같은 usecase/저장 실패 안내·원 프로젝트/원본불변. 기존 helper/설정값 존재로 실제 이어보기 지원을 계상하지 않는다. 모든 새 실행은 후속 gate이며 수동 GUI 보류를 유지한다.

연결: [시청](../presentation/viewing/viewing.md), [일상 transport](../presentation/viewing/everyday-transport-contract.md), [SDK 표면](api/host-support-boundaries.md), [추적 R16](../../../plans/player/design-completion/requirements.md).
