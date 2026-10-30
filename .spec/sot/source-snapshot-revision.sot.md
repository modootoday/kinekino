---
id: sot:kinekino/source-snapshot-revision
title: 프로젝트 source·작업 snapshot과 불변 revision
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 프로젝트 source·작업 snapshot과 불변 revision

2026-10-21 후속: origin-bearing live capture는 schema2를 기록하고 schema1의 origin None은 기존 bytes/해시와 읽기 전용을 보존한다. saved 참조는 원 origin을 그대로 재사용한다. 별도 [capture mutation 계약](capture-mutation.md)과 [실제 CLI 검증](../../../verification/application/capture-mutation-cli-2026-10-21.md)은 PM root541b4c8에 채택됐다. 아래 기존 readonly projection은 UI의 현재 동작이며 새 SDK 지원만으로 자동 해제하지 않는다. 프로젝트 latest와 제안 mutation의 cross-root 원자 transaction은 보장하지 않는다.

상태: 공통 SDK/API·CLI 저장 구현과 [고정 기능 검증](../../../verification/application/projects-2026-10-19.md)을 완료했다. GUI의 typed 상태 표시·입력·저장 API 연결 UX는 후속이다. 기능 판정은 SDK/CLI 근거를 사용한다. proposals Source1/Source2와 저장된 draft/acceptance를 바꾸지 않고 projects.create/inspect/save_revision을 공통 Rust 유스케이스로 제공한다. 프로젝트는 비어 있는 성공 placeholder가 아니다.

## 입력과 저장 내용

- CreateProject { project_id, title, content: ProjectContentInput }.
- SaveProjectRevision { project_id, title, content: ProjectContentInput }. RequestContext.expected_revision은 현재 **프로젝트** revision이며 필수다.
- InspectProject { project_id, revision: DecimalU64 또는 null }. null은 읽는 시점 최신, 지정 값은 불변 과거 revision이다.
- ProjectContentInput { watching: WatchingSelection 또는 null, proposals: ProposalRevisionRef[] }. 둘 다 비어 있으면 거부한다. proposals는 중복 ID 없이 최대32개다. ref는 `origin: live`와 `origin: saved`의 tagged enum이며 아래 독립 재저장 계약을 따른다. working_draft는 ProposalPayload 또는 null이다.

ProposalRevisionRef는 실제 ProposalStore의 현재 저장 상태와 revision을 대조해 capture한다. source snapshot·생성 raw·수정 draft·현재 acceptance를 포함한 self-contained StoredProposal과 그 canonical JSON SHA를 ProjectProposalSnapshot에 저장한다. 제안 작업 상태(원생성 needs_repair/편집 가능/채택 typed 결과)는 이 값에서 도출한다. 작업 목록이 없다고 AI job 성공·완료0건으로 가장하지 않으며 jobs linking은 아직 unsupported다.

working_draft는 제안 저장 전의 UI 편집 buffer를 별도로 보존한다. captured StoredProposal이나 기존 acceptance를 수정하지 않는다. task/source kind는 일치해야 하며 입력 크기를 제한한다. 참조·내용의 최종 검증 결과를 absent/valid/needs_input으로 도출해 기록하며, 빈 수정문·아직 미선택한 참조를 유효 제안/채택으로 가장하지 않는다. 다시 열어 명시적으로 제안 저장·채택하기 전까지 기존 typed 결과를 대체하지 않는다.

WatchingSelection은 { label, source }이며 source는 { kind: local_video, source_id, path } 또는 { kind: episode_page, source_id, episode_url }이다. local_video는 절대 로컬 동영상 선택 문법, episode_page는 credential/query/fragment 없는 https source.example.test/e/...를 검증한다. **호출자가 선택한 locator 기록**이며 media bytes·현재 native media·재생 위치·source PTS·현재 재생과의 결합이 검증됐다는 뜻이 아니다. 저장/열기에서 URL을 열거나 프로그램·미디어를 실행하지 않는다. 시청 선택만 있는 프로젝트도 실제 source locator를 보존하며, 편집 프로젝트는 실제 제안 source/작업/결과를 보존한다.

ProjectRevision schema1은 project_id/revision/parent_revision/title/watching/proposals/work_state를 가진다. work_state의 needs_repair·editable·accepted_output 수는 저장된 proposal snapshots에서 재계산한다. StoredProjectRevision은 revision_sha256, latest_revision, project를 반환한다. 생성은 revision1, 저장은 CAS 후 n+1로만 추가한다. parent_revision은 최초null, 이후n이다. 기존 source/생성 원본/과거 project revision을 덮어쓰지 않는다.

## 일관성과 범위

### 저장된 작업의 독립 재저장

구현할 proposal 입력은 origin으로 구분한다. `live`는 proposal_id/draft_revision/working_draft를 받아 ProposalStore의 현재 값을 새로 capture한다. `saved`는 proposal_id/capture_sha256/working_draft를 받아 CAS 기준 프로젝트 revision의 기존 snapshot을 그대로 재사용한다. saved는 save_revision에서만 허용하며 create에서 거부한다. ID와 capture SHA가 기준 revision에 정확히 일치해야 하고, 별도 proposal backend 없이 작업 buffer를 저장할 수 있다. live 조회 실패를 saved로 자동 대체하지 않는다. 저장 snapshot 재사용은 현재 proposal 최신성 검증을 의미하지 않는다.

단일 저장 record는 canonical JSON UTF8 3MiB, 전체 working_draft는 512KiB로 제한한다. 요청 및 응답 전송 한계 안에서 검사하며 ID/revision 예약 전에 초과를 거부한다. 실제 구현·회귀 전까지 이 문단은 계약이며 지원 성공 근거가 아니다.

ProjectStore의 writer lock 아래 최신 revision을 비교하고 단일 불변 publication을 발행한다. SHA 이름 파일·create_new/write/sync·중간/손상 상태 거부 원칙은 proposal 저장과 같다. 크기 제한은 ID 예약 전에 검사한다. 서로 다른 revision의 edit/save 경쟁은 revision_conflict로 거부한다. 모호한 응답 후 stale save를 자동 재실행하지 않는다. 조회로 저장된 revision을 확인한 뒤 사용자가 결정한다.

제안 snapshot은 capture 시점 proposal revision과 대조한다. 별도 proposal/project root 간 원자 transaction·publication 순간의 proposal 최신성·현재 native media binding을 주장하지 않는다. 기존 proposal accept의 project_current_revision_verified=false 한계도 이 기초 저장만으로 true가 되지 않는다. 향후 실제 project-bound acceptance는 별도 원자 계약이 필요하다.

프로젝트 조회는 self-contained snapshot을 읽으므로 제안 backend가 없어도 가능하다. 생성/저장에 live refs를 넣으면 ProposalStore가 필요하며, saved refs 재사용은 필요하지 않다. 원래 제안이 이후 바뀌어도 과거 project snapshot의 source/raw/result는 불변이다. source 재연결·export·실제 SRT/card/media 렌더·자동 재생 복원·지속 jobs는 이 첫3명령의 완료로 표시하지 않는다.

## 조립과 typed UI port

### 소스 locator 재연결 후속 — 구현 중, 실제 인수 전

`projects.reconnect_source`는 `ReconnectProjectSource { project_id, expected_source, replacement: WatchingSelection }`과 필수 양수 `expected_revision`을 받는다. 최신 project와 기존 source 선택을 대조하고 source kind/source_id를 유지한 locator 선택만 새 불변 revision에 발행한다. title·제안 capture·working buffer·원문·generated·origin은 그대로 복제하며 과거 revision을 변경하지 않는다. historical capture의 기존 입장 검증도 생략하지 않는다. 다른 writer와의 발행 CAS는 기존 ProjectStore가 담당한다.

응답은 `{ state: StoredProjectRevision, scope: "watching_locator_only", content_identity_verified: false, source_opened: false, playback_rebound: false }`다. 이 작업으로 미디어 파일 존재·바이트 일치·재생 권한을 입증하지 않는다. 현재 project revision 또는 expected_source가 달라지면 revision_conflict, kind/ID 변경과 잘못된 locator는 invalid_argument, watching 선택 부재는 needs_input이다. timeout/idempotency 및 실행 중 취소는 지원하지 않는다. 실제 CLI 검증과 새 입력 통합 회귀 전에는 인수 완료로 계상하지 않는다.

시험 설계에는 modoo.today `modootoday/software-qa/test-fixture-and-mock-build` 본문(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)을 적용한다. 격리된 실제 ProjectFiles/CLI로 성공·충돌·과거 상태 보존을 확인하고, 합성 locator가 실제 미디어 검증을 증명하지 않는 한계를 유지한다.

프로젝트에 보존된 capture를 live proposal 편집기에 자동 재연결하지 않는다. 같은 ID/revision이라도 다른 backend/source일 수 있으며 현재 SDK에는 검증된 rebind 계약이 없다. 프로젝트 projection은 최대32개 snapshot을 읽기전용으로 표시하고 명시 선택하며, task별 하나의 현재 buffer를 자동 덮어쓰지 않는다. capture에서 live edit/accept 활성화는 향후 SDK의 mutation 시점 기대 capture/source SHA·revision 원자 guard가 마련될 때까지 미지원이다. UI 자체 비교나 current inspect 한 번을 그 guard로 대체하지 않는다.

명시 --project-root ABSOLUTE_DIRECTORY에서 Rust ProjectFiles adapter를 주입한다. 기본 backend는 unavailable, remote endpoint와 local roots를 동시에 쓰지 않는다. ApplicationClient에는 create_project/inspect_project/save_project_revision(context,payload)를 추가한다. 같은 명령/오류/CAS/손상 거부를 CLI와 GUI가 사용한다. UI는 현재 제안 선택의 ID/revision을 저장 입력에 넣고, conflict 때 buffer를 유지하며 명시 재조회한다. UI가 파일·OS API를 호출하거나 revision으로 결과를 추측하지 않는다.
