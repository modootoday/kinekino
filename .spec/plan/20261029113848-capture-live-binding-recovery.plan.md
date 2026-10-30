---
id: plan:kinekino/20261029113848
title: Capture에서 명시 편집 연결을 복구하기 위한 계약 조사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Capture에서 명시 편집 연결을 복구하기 위한 계약 조사

2026-10-21 읽기 조사에 따른 후속 계획이다. 확정 API나 구현·실행 성공이 아니다. 현재 capture-origin editable 경로의 최소 차단을 유지한다. [프로젝트 계약](../../../sots/player/application/api/projects.md), [전체 완료 조건](../full-product-integration-gates.md), [현재 실행 권한](../execution-leases.md)을 따른다.

## 기존 책임과 확장 후보

`application/proposals`의 edit/accept는 현재 상태를 읽고 기존 정책을 적용한 뒤 ProposalStore::publish를 호출한다. `infrastructure/proposals`의 실제 publication은 기존 WriterReservation 안에서 current 상태·immutable_equal·draft revision CAS를 확인한다. source/generated/historical의 불변성 검사는 존재하지만 요청한 capture와 같은 출처라는 권한을 제공하지 않는다.

SDK 담당은 이 동일 쓰기 잠금 안에서 기대 source 전체·원 상태 해시·draft revision을 대조할 guarded mutation 계약의 최소 확장을 검토한다. 기존 도메인 편집/채택 정책과 locked publication helper를 재사용한다. 지원하지 않는 저장소 구현이 pre-inspect 비교로 성공을 가장해서는 안 된다. accept의 idempotent early return도 guard 전에 성공을 반환하면 안 된다. 타입·포트의 구체 모양은 담당 검토 전 확정하지 않는다.

## 해시와 출처

프로젝트 capture SHA는 typed StoredProposal 전체의 canonical JSON 해시다. 원 자산 bytes의 source SHA, source typed canonical SHA, 디스크 raw JSON 해시는 서로 다르다. 최초 명시 연결이 허용될 때 원 capture 전체 기대값을 검사하고 편집 후에는 새 revision과 새 기대 상태로 전진해야 한다. 원 capture를 덮어쓰거나 그 해시를 모든 후속 mutation에 그대로 적용하지 않는다.

내용 해시 일치는 backend identity 증명이 아니다. 현재 capture에는 신뢰할 backend origin이 없으므로 과거 capture를 현재 endpoint/root로 자동 재연결하지 않는다. 경로 문자열이나 caller가 정한 임의 backend ID를 승인으로 사용하지 않는다. 기존 SDK/store가 발급·검증하는 origin/lease 재사용 가능성과 과거 origin 없는 capture의 호환 정책은 별도 검토한다. 프로젝트와 제안 저장소 사이의 cross-root 원자 transaction을 이 요구로 가장하지 않는다.

## 최소 인수 조건

- 같은 ID/revision/선언 SHA라도 실제 source·generated·backend가 다르면 mutation과 publication을 거절한다.
- guard와 실제 publication 사이의 경쟁은 동일 lock/CAS 경계에서 거절한다. UI 사전 조회로 대체하지 않는다.
- 허용된 최초 연결→편집→새 기대 상태로 명시 채택 및 재호출에서도 guard가 우회되지 않는다.
- 기존 capture·working draft를 자동 변경하지 않으며 출처가 없는 과거 capture를 소급 승인하지 않는다.

SDK가 공통 계약과 저장소 경계를 구현·CLI로 검증하고 UI는 반환된 typed 권한만 소비한다. 임시 차단 해제는 그 근거를 인수한 다음이다. 현 native closeout gate에 이 범위를 추가하지 않는다. 연구 서브에이전트는 코드·문서·빌드·OS를 변경하지 않고 경계만 조사했다.

## PM이 배정한 출처·호환 방향

SDK의 후속 조사에서 현재 capture에 영속 backend origin 발급 계약 자체가 없음을 확인했다. 다음 방향으로 작은 ADR과 실제 타입 정합을 먼저 확정하도록 배정했다. 구체 필드·API 구현이나 검증 완료를 의미하지 않는다.

- 과거 origin 없는 capture는 원 bytes·버전·canonical 해시와 읽기 전용 상태를 보존한다. 소급 승인·자동 이관을 하지 않는다.
- 새 capture는 구성된 ProposalStore가 실제 저장소 메타데이터에서 발급한 안정된 origin을 공통 capture producer가 기록하는 방안을 우선한다. caller 문자열·Arc identity를 출처 승인으로 쓰지 않는다. 안정 ID를 비밀이나 보안 권한 토큰이라고 주장하지 않는다.
- guarded mutation은 구성된 ProjectStore의 불변 revision/capture 참조를 실제로 읽고, 현재 ProposalStore의 출처와 기대 source/state/revision을 동일 writer 경계에서 검사하는 방안을 검토한다. caller가 조립한 capture DTO만으로 권한을 만들지 않는다.
- 불변 프로젝트 revision을 기대값으로 사용하며 project latest와 proposal 사이의 cross-root 원자성을 주장하지 않는다. accept의 재호출도 기대 상태 검사 전에 성공하지 않는다.
- 버전·optional field·canonical 표현은 기존 저장 데이터의 검증 해시를 깨뜨리지 않아야 한다. 구체 호환 방식은 기존 read/write 코드와 대조해 SDK 담당 ADR로 확정한다. 기존 publication/CAS를 재사용하며 새 전역 registry나 보안 프레임워크로 확대하지 않는다.

## Producer20 읽기 감사와 mutation 후속

2026-10-21 source20 manifest `16D02EA64EC9B499F60DB5BFF6C05D9CBAB864D61BAA5E421523D39EA4CC6EE8`의 실제20과 before20을 읽기 조사로 대조했다. 확정 P1/P2는 발견하지 못했다. 원 writer 안에서 명시 live capture만 발급하고 원 root File의 volume/file identity와 metadata handle을 유지하며 Saved는 원 origin을 재사용하는 경계를 확인했다. 컴파일·동시 writer·복사/이동·지원 filesystem·v1 golden 시험은 실행 전이다. 이 감사는 구현 인수나 mutation 허용이 아니다.

SDK 후속은 configured ProjectStore의 immutable revision+capture 참조를 실제 읽은 anchor로 삼는다. continuation은 마지막 성공 mutation의 revision/full state SHA라는 CAS 기대값이며 비밀이나 권한으로 사용하지 않는다. 동일 writer 아래 물리 origin·원 source/generated/historical·기대 current state를 확인하고 기존 domain transition/publication을 재사용한다. accept 재시도는 guard 이전 receipt 반환을 허용하지 않으며 원 pre-accept 상태와 결정적 post-state/receipt의 일치를 검증해야 한다. 임의 continuation·다른 project anchor·복사 root·legacy capture·반복 edit→accept→retry의 양성/부정 시험을 담당에 배정했다. 구체 API·소스·실행 결과는 아직 인수 전이며 UI 읽기 전용 차단을 유지한다.

후속 core10 `965DDBE70D7413F791E25ACDCAEF8AAF007BA55AEB958E0F91C85EBEACB85415`는 producer20 위 override4/new6으로 동결됐다. 별도 읽기 감사에서 raw10·before10·부모 override 핀 일치와 확정 P1/P2 미발견을 회수했다. 공통 domain transition·원 writer session의 origin/complete anchor/CAS·accept retry의 pre/post 전체 SHA 대조를 확인했다. 작성9는 실제 파일 fixture7과 domain2이며 아직 타입·실행0이다. API/schema/client/CLI 연결은 별도 후속 묶음으로 계속 구현한다. SDK는 계약 동결 뒤 UI와 typed selector·continuation·오류 소비를 직접 조율하며 실제 CLI 통합 인수 전 차단을 해제하지 않는다.

API source12 `B4155D23B8D9B602426B90B4339638492B37DB54DE8AFC16AD30FB7C2FE25C06`과 합성 union35 `B4E16D551238FB7D2E7E614F3C871767D67B59EDD3C17FED885D57BB3677818D`도 읽기 감사에서 실제 raw35/unique35·부모 핀 일치, 확정 P1/P2 미발견을 확인했다. strict DTO/schema·typed client·CLI가 같은 guarded 유스케이스로 연결되며 capability는 authority를 대신하지 않는다. 작성 lib22와 ignored CLI1은 타입·실행 전이다. CLI fixture의12 원 프로세스 호출은 고정 실행파일·deadline/family/EOF/raw와 실패 owner 보존을 재사용하도록 작성됐으나 실제 프로세스 증거는 아직 없다.

UI는 별도 source3 `B15505E4905C27FC87DBC74CBB043D239DAA88F5C6ADF37D725C5B769D0AD584`를 준비했다. union35와 겹침 없이 exact anchor·성공 continuation·원 accept retry payload·pending 선택·원 오류를 소비하는 adapter와 작성7이다. SDK 제공자와 독립 읽기 감사에서 실제3/before 핀 일치·확정 불일치 미발견을 회수했다. foreign completion은 원 owner/serial/pending을 대조해 소유권을 반환하며 성공 응답만 state를 전진시킨다. 작성7은 typed client와 기록 응답을 쓰는 전달·소유 단위 범위이며 실제 backend·worker cleanup·callback/frame을 입증하지 않는다. 실행 중 panic/결과 미관측 복구·close는 기존 ExecutionOwner와 통합할 후속이다. root readonly·host/callback 활성화는 변경하지 않았고 GUI/IPC·타입/실행 인수도 아니다. 실행 순서는 현재 배정 문서만 따른다.

## 검증된 선행과 원 execution owner 연결 후보

위 단락들은 당시 후보 단계의 기록이다. SDK35 실제 CLI 인수는 root541b4c8,
UI adapter7·영향51·원 IPC3의 고유61은 root11829b6에 채택됐다.
[UI 실제61 근거](../../../sots/verification/presentation/capture-ipc-adapter-2026-10-21.md)를
새 연결 후보의 결과로 승계하지 않는다.

PM 후속 배정으로 `runtime/development/ui-capture-execution-owner-r1/HANDOFF.md`에
source13 `A0467220D781A40BBC6D594E159E74635C5C131196E5B8AB4885DA22EF1A0101`을
동결했다. 기존6/new7이며 작성 중 root가 direction 채택4d14cee/Meari9c1b6a로
전진한 사실과 기존6의 차이0을 별도 기록했다. SDK의 기존 attachment를 기존
ViewerExecution/ExecutionOwner에서 얻어 같은 CardRoute가 원 intent/request,
operation/terminal, 명시 소비·폐기와 ACK·해제·join 순서를 보유하는 후보다.
SDK에는 abandon API가 없으며 UI 명시 폐기는 원 응답·진단을 보존한 ACK 결정이다.
running cancel을 mutation 미발생으로 바꾸지 않고, ACK 실패 시 재적용·재전송하지 않는다.

작성 시험은 원 worker 계약9와 별도 ignored LocalHost2다. LocalHost fixture의
ProjectStore 미구성 Unsupported 경로는 양성 guarded mutation 인수가 아니다.
직접 rustfmt/check13만 통과했고 Cargo/타입/실행0이다. 실제 origin이 저장된
guarded edit/accept/retry IPC, 실패·닫기 소유권, 최종 callback/UI 바인딩과 별도
채택 전까지 기존 캡처 읽기 전용 차단을 유지한다. 현재 실행 권한은 중앙 배정을 따른다.

## 명시적 편집 진입과 양성 IPC 후보

`runtime/development/ui-capture-positive-entry-r1/source10.json`의 SHA256은
`850A8DD60FB23D97729AF6946671B896A06349E1278BB52E16EED87488E337EB`다.
부모는 원13에 명시 module path 수리를 적용한 gate final13
`EB62DCE7336511EC677C9803C607B8BA02A059ED90D2DE067A20E3CA3E6C6E29`이며,
override4를 적용한 union19는
`893A1B43D789D733E24A9850B75F8062D455A1252871404DEAFE9AB9F26D6BE7`다.
독립 읽기 검토에서 실제 raw10·union19·부모 핀이 일치했고 확정 P1/P2는
발견하지 못했다. 이는 컴파일·실행 인수가 아니다.

후보는 host-held StoredProjectRevision의 표시 행을 불투명 식별자로 선택하고
원 capture anchor를 구성한다. origin Some은 진입 가능성만 나타내며 권한은
동일 ExecutionOwner→EndpointClient→LocalHost와 구성 저장소의 guarded backend가
판정한다. 기존 onCardSelection 거절을 유지하고 별도 opt-in onCapturedCardEdit를
사용한다. 원 read consume/ACK/release와 편집 adapter attachment의 소유를 보존한다.

작성된 ignored IPC7은 edit/edit/accept/원 preaccept exact retry, stale·legacy 거절,
미소비 close, commit 뒤 client panic과 consumer decode fault를 다룬다. 아직
실행하지 않았고 현재13 빌드에 합성하지 않았다. 구체적인 capture ACK 실패 주입,
생산 mutation 입력 callback·Card mount·frame·route 활성화와 실제 사용자 흐름은
미완료다. 담당은 현재13 actual 반환 후 별도 IPC 검증과 최종 연결을 이어간다.
