---
id: adr:kinekino/20261029113511
title: "ADR: 새 capture의 저장소 origin과 프로젝트 v1 해시 보존"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR: 새 capture의 저장소 origin과 프로젝트 v1 해시 보존

상태: 2026-10-21 PM이 physical root identity 결속과 v1 None 생략/v2 Some·mixed None 읽기 전용 및 공통 guarded mutation 방향을 수락했다. 최종 source35는 [실제 파일·CLI 검증](../../sots/verification/application/capture-mutation-cli-2026-10-21.md)을 통과했고 PM이 root541b4c8에 채택했다. UI 활성화·전체 제품 완료를 뜻하지 않는다. [조사](../../plans/player/presentation/capture-live-binding-recovery.md), [기존 프로젝트 계약](../../sots/player/application/api/projects.md), [명시 mutation 계약](../../sots/player/application/api/capture-mutation.md)을 따른다.

## 문제

현재 capture에는 backend origin이 없다. content/capture SHA, 같은 Arc/ExecutionHandle, Live/Saved 태그는 출처 증명이 아니다. origin 없는 과거 capture를 현재 backend로 소급 승인하지 않는다. 기존 project canonical 해시와 불변 파일도 바꾸지 않는다.

## 제안한 데이터와 발급 경계

ProposalStore가 제공하는 optional capture producer를 통해 현재 StoredProposal과 origin을 같은 WriterReservation 아래 읽는다. 미지원 adapter는 origin 없는 capture/read-only를 명확히 반환하거나 guard를 Unsupported로 거절한다. UI나 요청 payload가 origin을 발급하지 않는다.

ProposalFiles는 명시적인 새 live capture 요청에서만 root의 별도 version1 metadata를 읽거나 최초 생성한다. 기존 root를 열기·조회·saved capture 재저장만 할 때 metadata를 자동 생성하지 않는다. 발급은 기존 writer.lock 아래 create_new/write/sync로 단일화하며 부분/손상/unknown-version metadata는 보존·거절하고 새 ID로 덮어쓰지 않는다. 안정 ID는 기존 Rust/OS GUID 발급을 재사용한 비밀 아닌 dataset identity이며 권한 토큰이 아니다. 새 전역 registry나 보안 프레임워크를 만들지 않는다.

복사된 metadata만으로 다른 실제 root가 같은 origin을 주장하지 않도록, metadata의 발급 root identity를 원 directory handle에서 얻는 volume/file identity와 결속한다. 다른 root/지원하지 않는 filesystem이면 guard를 fail closed하며 path 문자열로 fallback하지 않는다. 같은 volume rename은 실제 identity가 유지되는지로 판정하고, 복사/다른 volume 복구는 기존 identity와 구별하며 mismatch를 자동 재발급하지 않는다. 이 물리 identity 확인은 infrastructure 책임이고 domain origin DTO에는 OS handle/path를 노출하지 않는다. 기존 Windows crate의 최소 filesystem feature/adapter 확장만 source 후보로 추가한다.

## v1을 깨지 않는 v2 규칙

1. ProjectProposalSnapshot의 끝에 versioned `proposal_origin: Option<...>`을 추가하는 후보를 사용한다. `serde(default, skip_serializing_if = "Option::is_none")`로 None을 원 JSON에서 완전히 생략한다. 기존 필드 순서·serializer·canonical_sha256 알고리즘은 유지한다.
2. schema1은 origin None만 허용한다. 원 v1 raw/파일명·revision_sha256/capture_sha256를 그대로 읽는다. old record를 재직렬화해 덮어쓰거나 자동 이관하지 않는다.
3. origin을 포함하는 새 명시 publication은 ProjectRevision schema2를 쓴다. v2는 saved legacy(None)와 새 capture(Some)를 함께 담을 수 있다. None은 언제나 읽기 전용이며 일부 origin이 있다는 이유로 다른 항목을 승인하지 않는다.
4. saved ref는 저장된 captured/origin을 그대로 재사용한다. live ref는 구성된 ProposalStore producer가 현재 원문과 origin을 발급한다. 원래 capture_sha256는 StoredProposal 전체의 canonical 해시로 유지하고 origin은 project revision 해시에 별도로 포함한다. 두 해시를 혼합하거나 새 의미로 재사용하지 않는다.
5. 기존 struct literal의 None 추가는 호환 수리로 분리한다. 새 reader의 실제 legacy golden bytes→typed→canonical SHA 일치, 기존 파일 unchanged, v2 mixed capture 및 old reader의 new-version 거절 경계를 Rust 시험으로 입증한다. None 생략 속성만으로 해시 보존을 증명했다고 하지 않는다.

## 원자 mutation의 다음 계약

guard는 caller가 조립한 capture 권한 대신 구성된 ProjectStore의 정확한 불변 revision/capture ref를 읽는다. 구성된 ProposalStore의 실제 origin과 기대 source/full StoredProposal/revision을 동일 writer 아래 대조하고 기존 domain 정책·locked publication/CAS를 재사용한다. ProjectStore의 current latest나 cross-root transaction은 보장하지 않는다. accept의 기존 early receipt return에도 같은 guard가 먼저 적용돼야 한다.

edit 뒤에는 새 기대 state로 전진해야 하며 저장된 project capture는 자동 변경하지 않는다. 첫 연결의 full capture 기대와 후속 state 전진·accept 재호출을 어떻게 결속할지는 별도 소규모 port/lease 계약으로 확정한다. 원 pre-accept hash를 post-accept current에 그대로 비교하거나 UI 재조회로 guard를 refresh하지 않는다. 이 ADR만으로 one-shot만 지원하는 축소 API나 매 edit 뒤 강제 project 저장 흐름을 도입하지 않는다. GUI와 CLI가 같은 application 유스케이스를 사용해야 한다.

## 소스 동결 이력과 실행 검증

최종 실행 후보35 `90714B052FC7ACD38518B9587E7320F9515599DC87462E62D255ADA8D29B9408`는 원 union35 위 시험2파일의 최소 수리이며 guard 정책/DTO는 유지했다. default fmt/all-target strict, rootlib22 exact, 실제 CLI1/원 child12 및 수리 회귀2를 확인했다. 원 runner34 receipt와 child12 family/EOF·raw, 보호 차이0 및 슬롯 반환을 기록했다. 전체 suite·UI·모델/미디어 인수는 아니다. 아래 source20/10/12의 실행0 문구는 각 동결 시점 이력이며 현재 결과는 위 검증 정본을 따른다.

API/CLI 후속 후보는 `runtime/development/proposal-capture-api-r1`의 source12/before12/parent-overrides3/HANDOFF/EXECUTION에 고정했다. source12 SHA256 `B4155D23B8D9B602426B90B4339638492B37DB54DE8AFC16AD30FB7C2FE25C06`, 부모 순서대로 합친 union35 SHA256 `B4E16D551238FB7D2E7E614F3C871767D67B59EDD3C17FED885D57BB3677818D`다. `proposals.edit_capture/accept_capture`의 strict DTO·schema·capability·typed client와 CLI 도움말을 연결하고, 고정 실제 CLI를 12회 호출하는 ignored 시험1 및 API 시험2를 작성했다. leaf12 직접 포맷/검사만 통과했으며 Cargo·type·실제 CLI 실행0이다. UI에 명시 anchor/continuation/error 계약을 전달했으며 실제 guard 통합 검증 전 차단해제는 허용하지 않는다. EXECUTION은 중앙 배정 후 재현할 절차이며 실행 권한이나 결과가 아니다.

guard core 후속 후보는 `runtime/development/proposal-guarded-mutation-r1`의 source10/before10/parent-overrides4/HANDOFF에 고정했다. source10 SHA256은 `965DDBE70D7413F791E25ACDCAEF8AAF007BA55AEB958E0F91C85EBEACB85415`다. configured immutable project anchor와 기존 physical origin·불변 source/generated/historical을 writer session 안에서 검증하고, continuation은 마지막 성공의 revision/full-state SHA라는 비밀 아닌 CAS 기대값으로만 쓴다. accept 재호출은 원 pre-accept 불변 revision 해시와 도메인 전이가 계산한 post-state 전체를 current와 대조한 뒤에만 receipt를 반환한다. 기존 일반 경로도 같은 도메인 전이·private locked publication을 공유한다. 새 시험9를 작성하고 leaf10 직접 포맷/검사를 통과했지만 컴파일·실행0이다. API/CLI request routing·schema·client와 실제 CLI 검증은 아직 연결하지 않았으며 UI 차단은 유지한다.

producer·호환 후보는 `runtime/development/proposal-capture-origin-r1`의 source20/before20/HANDOFF에 고정했다. source20 SHA256은 `16D02EA64EC9B499F60DB5BFF6C05D9CBAB864D61BAA5E421523D39EA4CC6EE8`이며 root3ebb449 기준 existing14/new6이다. UI가 제공한 struct literal 호환4의 before/raw를 대조해 포함했다. Rust leaf19 직접 포맷은 통과했지만 Cargo fmt/test/clippy와 실제 filesystem 동작은 미검증이다. 작성한 시험에는 v1 frozen bytes/canonical SHA, v2 mixed, old strict snapshot reader, schema 경계, root rename/copy, 부분 metadata, writer 경합 및 saved-origin 재사용이 포함된다. 실제 다른 volume 복구·미지원 filesystem은 실행 근거가 없다. root 제품 소스·index·lock은 변경하지 않았다.

SDK는 검증된 최종 후보를 PM에 인계했다. UI는 확정 typed anchor/성공 continuation/원 accept retry payload와 원형 오류만 소비하며, 원 host 및 별도 UI gate 전 capture-origin 읽기 전용 차단을 해제하지 않는다. caller 문자열이나 현재 inspect를 권한으로 승인하지 않는다. 미지원 filesystem·다른 volume 복구·동시 다중 CLI 경쟁·전체 제품 흐름은 추가 실증이 필요하다.
