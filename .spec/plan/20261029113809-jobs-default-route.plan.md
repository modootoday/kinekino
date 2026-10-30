---
id: plan:kinekino/20261029113809
title: 기본 JOB route 연결 후속 — source 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기본 JOB route 연결 후속 — source 계획

2026-10-23 root cce969547f2011e13d30badf2c5f6cc23a0a376f 채택 뒤 읽기 조사만 수행했다. 코드/Cargo/S/T/실행 변경은 없다. [actual138](../../../sots/verification/presentation/union22-sdk-connection-2026-10-23.md)은 opt-in 패널 근거다. 채택 시 captured-proposal-panel.tsx의 끝 두 빈 줄 정규화는 원 검증 SHA와 구별하며 새 root 전체를 동일 BIN으로 시험했다고 주장하지 않는다.

후속 PM 배정으로 runtime/development/ui-jobs-route-r1 독립 stage에 소스를 작성 중이다. 첫 presentation8과 공통 ReadConsumer 추출16 checkpoint를 각각 동결했다. checkpoint-source16.json AC54A63595DA3B4990C9E6F13CCA29CF9F1EC16EFFA8F4714F94EDFFF889A130, before16 5F2C69525E927D933F1C91E5AA24D2D2FA98A76C62173D2371DC221F71A5DB8E, 기존5/신규11·작성6/컴파일0/실행0이다. root 공유 파일은 불변이고 직접 leaf fmt/check만 수행했다. 기본 route graph/owner 인계/callback 조립 및 자동 연결 fixture가 아직 없어 이 checkpoint를 실행 가능한 완성 입력으로 계상하지 않는다.

## 확인한 연결 공백

routes.json에 studio.jobs와 /studio/jobs가 선언돼 있지만, player_shell/preparation_tests.rs는 해당 경로의 SourceNotIntegrated를 명시한다. 선언 존재는 mount/React Router 전체 호환 근거가 아니다. StudioApplication은 기존 StudioFrame에 workspace/inspector ReactNode를 받는다. JobsWorkspace는 단일 row DTO를 받는 표시 선언이며 실제 목록 조회/선택을 제공하지 않는다.

MountedJobDetails::mount는 별도 NativeDocumentHandle을 생성하고 소유한 ExecutionAttachment를 받는다. 이 handle을 StudioApplication의 ReactNode로 그대로 넣을 수 없다. 기존 JobDetailsPanel graph/projection/owner를 동일 제품 document의 owned inspector slot과 fresh callback session에서 소비하는 조립 경계가 필요하다. owner의 private 생성자를 임의 공개하거나 독립 document를 숨겨 유지하는 방식은 계획하지 않는다.

SDK ExecutionHandle::attach는 한 번에 하나의 consumer attachment만 허용한다. 기존 SdkPort::shutdown은 entries가 비어야 해당 attachment를 release한다. 그러므로 ViewerCallbacks와 JOB owner에 같은 Arc를 단순 복제하거나 독립 attach를 병행하는 것은 안전한 조립 계약이 아니다. 이 lifetime 공백을 해결하기 전 route source만 등록해 완료로 표시하지 않는다.

## 제안 순서와 책임 경계

1. UI 조립에서 공통 Shell navigation callback 수명과 route별 SDK 소비자 수명을 구분한다. 기존 ExecutionOwner는 부모가 계속 소유한다. route 요청 시 원 소비자의 query terminal/ACK/result decision을 먼저 drain하고 원 attachment release를 확인한 뒤 같은 ExecutionHandle에서 다음 route attachment를 얻는 순차 인계안을 우선한다. 새 worker·SDK 정책·전역 서비스·무조건 취소는 추가하지 않는다. navigation 자체가 drain 중 사라지지 않도록 root callback/route owner 경계를 먼저 설계한다.
2. job_details 내부에서 standalone document mount와 동일-document binding을 책임별로 분리한다. codec/pump/projection/실패 carrier는 재사용하고 제품 StudioApplication과 동일 PluginScope/HostStamp/generation, 원자적 props+callback export를 사용한다. inspector slot은 고정 wrapper의 유일한 자식으로 둔다. 비지원 mixed sibling migration에 의존하거나 Meari guard를 완화하지 않는다.
3. 실제 jobs.list/get SDK/CLI 근거와 정상 host의 read capability를 SDK 담당과 대조한다. typed JobId만 선택 입력으로 받아 목록 결과에서 명시 선택한다. 첫 행 자동 선택·가짜 성공·임의 revision·진행률/단계 이력은 만들지 않는다. 선택 없음은 빈 inspector slot과 명시 상태다. list/watch 계약 추가는 별도 배정 없이 이 계획에 자동 포함하지 않는다.
4. 기존 StudioApplication/InspectorDock/QueueFooter/공통 CSS/VerifiedAssets/Pretendard 계약을 조립한다. 좁은 화면의 inspector open 상태도 같은 host 상태로 전달한다. 문자열과 향후 다국어 키는 공통 label 책임으로 모으며 새 TS 정책/임의 제품 스타일은 만들지 않는다. URL·탭·선택 전달은 기존 Meari router/typed host 경계를 사용한다.
5. 최초 적용 전 새 root의 정확 source/preimage 및 소유 파일을 고정한다. 자동 SDK 연결 인수 뒤에만 별도 허가된 시각 검토를 수행한다. 제품 정책/저장/CAS/작업 취소 기능 검사를 GUI에서 중복하거나 CLI 지원 대신하지 않는다.

예상 UI 소유는 src/assembly/meari/studio_jobs/의 route 조립·owner 인계, src/assembly/meari/job_details/의 동일-document 소비 경계, src/presentation/web_ui_adapter/studio/의 목록/선택 표시 상태, src/presentation/web_ui/studio/jobs/ 선언이다. 기존 player_shell/ProductFrameBlueprint/문서 준비·source inventory 등록은 공유 파일이므로 PM이 exact delta를 배정한 뒤 수정한다. SDK application/domain/Cargo/library 변경은 이 계획의 소유 범위가 아니다. 실제 작성 파일 목록은 설계 검토 후 최소로 고정하며 지금 이 경로들을 실행 입력으로 간주하지 않는다.

## 필요한 자동 인수와 남는 한계

실제 기존 SDK/CLI 계약을 사용해 기본 route mount, 선택 없음, 명시 선택 A→B, in-flight A의 늦은 결과, 마지막 정상 기록+오류, fresh callback/old callback 거절, inspector 닫기·탭 이탈·재진입, 원 attachment 재부착/worker 보존을 확인해야 한다. 원 소비자 drain 실패/ACK 실패/DOM preflight 실패는 원 owner와 last-good frame을 보존하고 route를 관측 완료로 표시하지 않아야 한다. callback 충돌·한 번의 조회 중복 제출·무제한 자동 재시도도 검증 범위다. standalone mount failure/diagnostic Write·flush 실패 인수는 기존 미검증 후속으로 구분한다.

jobs.cancel/retry/openfolder/watch/실제 단계 이력은 현재 opt-in get 패널의 지원으로 계산하지 않는다. URL 선언·컴파일·실제 SDK 연결·DOM 입력/접근성·실제 render/visual·사용자 품질 승인을 각각 구분한다. 전체 화면 디자인 인수와 가상 모니터 검토는 현재 승인되지 않았으며 이 문서로 실행 권한이 생기지 않는다.

## 적용 스킬

## 2026-10-23 PM 대기 중 소스 체크포인트29

runtime/development/ui-jobs-route-r1/README.md의 host-hooks-r1에 기존13/신규16을 고정했다. 원 ExecutionOwner를 유지한 list/get 순차 인계, 동일 문서 callback export, ViewerHost 진입·복귀·close와 실제 receipt/frame arm 연결을 소스로 작성했다. source29 F5736F64B394E19098B4CAA9062C3C44C95AD20B4EE5AFA57CB44172C6D7731E, before29 15BEB8334BFEC75B742B0EE541EDB8410E2D94BA54B9ACE2D0A8325B8B54E761. root ec609170592e1098e3dddba933d8fc47ee53c2cb의 해당 preimage 차이0이며 library7fb53c99c1f550d651009d1f4fda6aa7a6a1a77b와 작성 기준0b9의 호환은 검증하지 않았다.

작성 시험8(표시6·기존 get 문서 인계2), 컴파일0·실행0이다. direct rustfmt24 leaf만 수행했고 Cargo/S/T/GUI는 건드리지 않았다. 기본 JOB 전체 route fixture, strict lint/compile, commit 뒤 frame-arm 실패 시 현재 export를 통한 복구가 남아 있으므로 채택 후보/완료로 표시하지 않는다. 세부 미검증 범위와 스킬 적용 판단은 stage README를 따른다.

### 기존 적용 스킬 기록

최신 r3는 runtime/development/ui-jobs-route-r1/HANDOFF-R3.md다. source34-r3 6FFDF31C529FE3F789475EA1491AFB583F369C3384B7755B34CFEBE5275493A6 / before 2A47F15C31998F593A427DA60D3E49997FB4D9D2E328FE64D37BD3B02C885BB5 / r2대비6경로delta 74A370CA602691EF91B51A2DB5B7DFADE72B882425A0CDB89D293636F557103B를 별도 고정했다. 기존16/신규18, root89c/library7fb preimage차이0·r2원raw33차이0이다. callback drain의 재투영 제거, 실제 NativeHostRuntime의 단일commit/turn, 원startup notifier 정상one-shot을 작성했다. 작성19/실행0이며 실제input revision 및 next_turn sequence 때문에 원receipt 일반rearm은 현재API로 불가능하다는 확정공백을 유지한다. PM이 Meari retained-observation 계약을 별도 준비하며 guard완화·forged stamp·receipt합성0이다. r2 gate-ready 표현은 과거 검토시점 기록으로 한정한다. source-only 잔여업무는 계속하고 goal상태를 변경하지 않는다.

2026-10-23 r2 소스 검토 입력은 runtime/development/ui-jobs-route-r1/GATE-READY-R2.md다. source33-r2 A71CC1A7531246F55A4791462E465835C600861A242FA1F37DCE5C8CB541C0A4, before33-r2 A971CD785ACECC4A2CE74D6D90040CE4D8E7510F5DDF22EA95B892B7AB2FBF68, 원29 대비delta10 ED497775A13C893AABAC7BB530467C99979D0C6D651E927816021D49515A02A3을 별도 고정했다. root89c0fa0da2ea63aa82cd36b79ad7994e128ac06a/library7fb/lockE469의 preimage차이0. 반환commit 뒤 원receipt/fullstamp와 현재Viewing export를 통한 명시rearm을 구현하고 전체host fixture9를 추가했다. 작성17/컴파일0/실행0이며 observer borrow실패와 native Busy guard, synthetic frame정책과 실렌더를 구별한다. PM 독립review 이후 별도 gate 배정 전까지 Cargo/GUI/ST실행0을 유지한다.

본문을 읽은 modoo.today modootoday-free/design-delivery-plus/frontend-first-journey의 공통 frame/typed port 및 미구현 상태의 명시를 mount/route/목록 공백 구분에 적용했다. modootoday/software-qa/test-fixture-and-mock-build의 원 자원 소유·정리·실패 증거 분리를 exclusive attachment 인계 및 rollback 인수 계획에 적용했다. 원 Rust API/구현을 읽어 판단했으며 새 fake backend·JS 정책·추측 SDK API는 도입하지 않는다.
