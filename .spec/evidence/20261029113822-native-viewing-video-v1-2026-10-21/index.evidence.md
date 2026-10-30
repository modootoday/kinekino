---
id: evidence:kinekino/20261029113822
title: Viewing semantic video V1 CPU 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Viewing semantic video V1 CPU 검증

2026-10-21. 최종 source10 `5F47A4CD371F1EB493C94195BAC585F2046F26AD16C0908C2B669FE2E4206F1C`의 고유48(기존41+신규7), default/native all-target strict Clippy와 fmt가 통과했다. 세 실제 필터는 controller::tests9/player_shell_document::tests9/viewer_30이며 ignored4는 미실행이다. 같은 최종 fresh Native binary로 실행했고 원 revision 성공 수를 중복 가산하지 않았다.

동결 소스·회귀 이름·원 실패/수리·보호/프로세스 반환 근거는 [최종 gate 결과](../../../../runtime/quality-review/ui-viewing-video-v1-r1/RESULT.md)와 [r4 인계](../../../../runtime/coordination/ui-viewing-video-v1-r1/revision-r4/HANDOFF.md)에 있다. Native binary SHA는 `7383110B1477C1D784A9468E33B86F4835054D9A5214FB918F9D099C75C45358`/163621376B다. root1443/snapshot1405 및 source/provider/원 bundle/Meari/Cargo 보호 차이0/index0, 원11 terminal receipt(원101한 개 보존)/기록22PID 부재를 확인하고 heavy/shared를 반환했다.

## 관측한 계약

default/read-panel 실제 제품 graph는 닫힌 ViewingVideoId::Primary 카탈로그의 `viewing-primary-video`를 같은 prop으로 전달하며 실제 video는 하나다. 영상 source/control/autoplay/callback이 없는 비활성 자리이고 tabindex -1/aria-hidden을 유지한다. 시작 콘텐츠/chrome/transport·기존 palette/spacing 토큰을 유지했으며 실제 시각 일치 결과는 없다.

PreparedShellDocument의 명시 borrowed 검사만 ID/video tag/단일성·source 속성을 검사하고 같은 actual handle의 full HostStamp를 owned metadata로 반환한다. 범용 mount 생성·기존 DocumentFailure 및 외부 owner 흐름은 바꾸지 않았다. missing/div/multiple/src 및 duplicate id의 실제 오류 경계를 확인했고, 같은 ID라도 다른 document/source/owner stamp를 갖고 원 reproject 후에는 historical metadata와 current revision이 다르다.

실제 read-panel mount는 local callback binding7과 video1을 함께 보존했다. 카탈로그 원본은 codec provider ContentHash의 include_bytes 입력으로 포함된다. 이는 surface/등록/child 생성 또는 SDK 지원 activation을 뜻하지 않는다.

## 실패 정정과 한계

원 r3의 video callback 시험은 Meari graph parser가 button/input 전용 handler 조건으로 거절해 실패했다. r4는 시험1만 실제 source 준비 거절 경계로 수정했고 나머지9파일/생산·단일성/권한은 불변이다. rejected parser는 원 prepared를 입력받지 않으므로 준비 값 보존 단언은 호출 형상 확인이다. 이후 원 graph 실제 mount/stamp 검증과 구분한다. Native callback/event/input 방어 분기의 직접 동적 커버리지로 계상하지 않는다.

실제 GUI·Native OS view·영상/mpv·CLI·focus/입력·geometry/시각·생산 NoMedia opt-in 설치·V2 정리 조립·main/default 전환은 이번 실행0이다. 기존 SDK B/ownwindow의 OS 성공을 V1 성공에 합산하지 않는다. 이후 V2는 현재 원 handle을 다시 검사하고 실제 preparation bind에서 fullstamp를 재검사해야 한다.
