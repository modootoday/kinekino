---
id: plan:kinekino/20261029113905
title: 프로젝트 캡처와 현재 제안 편집의 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 프로젝트 캡처와 현재 제안 편집의 경계

원 host 시험의 실패 소유권 수리 E23B08F1는 PM 소스 검토 범위에 적합하다고 확인받았고, [시험별 IPC 인수 후보](ipc-capture-negative-gate.md)로 동결했다. 실제3은 각 별도 프로세스/명시 배정이 필요하며 아직 실행하지 않았다.

최신 실제 근거는 [캡처 진입 차단51](../../../sots/verification/presentation/project-capture-readonly-2026-10-21.md)이다. source7 F73F5EA8·동일 BIN CCDB8F20에서 선택56 중51 통과/제외5를 확인하고 슬롯을 반환했다. 실제 callback 거부는 통과했으나 public caller/retained opening의 LocalHost 회귀2는 미실행이며 panic 정리 보강도 필요하다. 아래 source-only 작성 기록은 이력으로 보존한다. 원자 SDK guard는 여전히 미구현이다.

[프로젝트 API 정본](../../../sots/player/application/api/projects.md)의 `조립과 typed UI port`가 기준이다. SDK 담당이 재확인한 현재 지원 범위는 저장 캡처 읽기 전용이다. `proposals.edit/accept`의 draft revision CAS와 current inspect의 source tuple 비교는 mutation 시점의 기대 capture/source SHA·revision 원자 guard를 대신하지 못한다. 기대 guard를 제공하는 SDK 계약은 아직 없다.

원 UI28 CPU 실제75 결과와 root3a2870b는 실행 이력으로 보존한다. 그 결과를 캡처 기반 live 편집 권한이나 지원 승인으로 승계하지 않는다. 실제 selector는 아직 제품에 연결되지 않았다.

독립 수리 `runtime/coordination/ui-capture-readonly-boundary-r1/`는 실제 local callback의 opaque key 확인 직후 queue admission 전에 거부하고, 공개 route 요청과 자동 capture→inspection→editable bind 진입도 차단한다. 원 읽기 attachment·queue·ShellState·capture·working draft를 유지한다. 이미 남아 있는 capture-origin terminal/registration은 편집기로 올리지 않고 원 close 소유자에게 보존한다. 일반 scoped live 컴포넌트의 권한을 캡처로 우회하지 않는다.

작성한 실제 source callback 거부 회귀는 SDK 명령0·queue0·원 attachment 보존·borrow 전 거부를 검사하지만 아직 실행되지 않았다. 공개 caller/원 whole-host의 부정 회귀도 필요하다. 수리 source5는 source-only이며 현재 SDK gate나 기존 불변 UI evidence에 합성하지 않는다.

후속 현재 저장본 재열기는 새 사용자 의도, 실제 current store의 typed StoredProposal, 원 context/host generation, 명시 live 선택을 별개로 구성해야 한다. old capture/queued intent의 이름이나 origin만 바꿔 승계하지 않는다. current source identity가 기존 capture와 다르면 자동 재연결하지 않는다. saved ProjectRevision·working draft는 자동 변경하지 않으며 새 capture 발행은 기존 명시 save_project_revision/CAS 계약을 따른다. 이 분리는 아직 구현 완료가 아니다.

IPC 시험 후속 `runtime/coordination/ui-capture-host-retaining-r1/`는 private 시험1파일에서 body panic 뒤 원 ViewerHost close를 관측하고, 최초 panic·종료 panic·종료 오류와 원 owner를 실패 carrier로 함께 반환하도록 작성했다. partial inspection도 begin/poll 전 원 host에 넣어 정리 소유자를 보존한다. 새 panic 주입 시험을 포함한3개 모두 기본 제외/미실행이다. 직접 포맷만 확인했으며 타입·실행 및 실제 실패 정리는 다음 명시 배정에서 검증한다. 결과 보고 이후 Rust의 자연 Drop을 정상 close 증거로 사용하지 않는다.
