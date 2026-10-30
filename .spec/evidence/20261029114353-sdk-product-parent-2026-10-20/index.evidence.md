---
id: evidence:kinekino/20261029114353
title: 제품 SDK parent 등록 실행
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 SDK parent 등록 실행

[등록 계획](../../../plans/player/presentation/sdk-product-host-registration.md)의 첫 checkpoint다. 이전 [오디오 actual 정본](native-audio-webview-2026-10-20.md)을 확정하고 heavy/root 소비 동결을 반환한 뒤 중앙의 새 배정으로 시작했다. 이번 범위는 제품 SDK parent lease·manifest/context·도구 단일 소스 재사용이며 실제 NativeWindow/renderer/제품 GUI와 fresh CLI/provenance 저장 결합은 후속이다.

## 입력과 workspace 경계

증거 루트는 `runtime/quality-review/sdk-product-parent-r1`이다. root src/tests/Cargo/lock, consumer src/tests/Cargo/lock와 plugin의 이전 manifest/독립 lock을 source-before에 보존했다. Meari의 manifest/코드는 변경하지 않는다.

첫 root path 의존+plugin exclude 입력은 cargo fmt metadata 오류와 resolve-first76ms/exit101을 냈다. 메시지는 root와 player-ui-plugins의 multiple workspace roots다. 제품 전용 plugin의 독립 [workspace]를 제거하고 root members/default-members에 편입하는 정확 manifest 변경을 중앙과 조율했다. 별도 gitlink나 Meari 본체를 제품 workspace로 이관하지 않았다. 기존 plugin Cargo.lock은 삭제하지 않으며 이후 plugin의 canonical lock은 root Cargo.lock이다.

resolve-r2는39566ms/exit0이었다. 그러나 metadata-r2는 Meari core까지8member를 관측했다. 정확한 core exclude를 보완한 r3 check11450ms/exit0 뒤에도 metadata-r3는8member였다. root members의 명시 `.`만 제거하고 default-members의 `.`는 유지한 metadata-r4에서 루트 패키지를 포함한 제품7member/default7 및 Meari 제외를 확인했다. 이 환경의 실제 전후 관측이며 모든 Cargo 버전에 대한 일반 원인 단정으로 확대하지 않는다. 공식 [Cargo workspace 참조](https://doc.rust-lang.org/cargo/reference/workspaces.html)와 구분해 원문 metadata/debug trace·각 manifest/lock을 보존했다.

최종 경계의 offline 최소 resolve-r4는12001ms/exit0이었다. 새 core 의존이 요구하는 syn 정확 pin 때문에2.0.119→2.0.117 변경이 필요했고, root lock의 다른 기존 pin을 임의 갱신하지 않았다. Meari가 root member가 아닌 dependency가 된 뒤 wasm-bindgen-test 등 불필요 dev-only edge를 정리한 r2→r4 diff도 따로 보존했다. 이후 실행은 offline/locked/-j1·2CPU/BelowNormal·기존 cache·단일 runner로 한다. 독립 consumer lock은 root lock을 복사하지 않고 자체 최소 resolve 후 고정한다.

## 코드 책임

src/lib.rs→assembly/meari 부모에 제품 proposal adapter와 canonical proposal_manifest/request_context/ProposalSdkRegistration을 등록했다. register는 호출자가 전달한 ApplicationClientHandle·같은 Rc registry/context·actual scope를 사용하며 SDK/store를 생성하지 않는다. close 실패는 handle/registry를 보존하고 명시적 close 재시도를 허용한다. Drop은 best effort이며 성공 영수증이 아니다.

tools/player-ui-sdk-bridge는 product_proposals·proposal_manifest·scoped_context를 제품 root에서 재수출한다. 도구의 중복 함수 본문과 proposal adapter의 과거 path 소스 재컴파일을 제거했다. 제품이 tools에 의존하지 않는다. subtitle conversion의 기존 독립 경로는 이번 proposal 등록과 혼동하지 않는다.

현재 request/reply pin은 proposals mod 각1파일을 입력으로 한 호환성 metadata이며 별도 document_source/stored DTO 전체나 canonical JSON schema inventory를 포괄하지 않는다. provider pin도 adapter1파일이며 request context·typed plugin 전체의 attestation이 아니다. 이번 이관의 단일 소스와 기존 동작 인수를 production trust 전체 완료로 부르지 않는다. 후속 [읽기 검토](../../../plans/player/presentation/sdk-product-parent-review.md)의 canonical inventory 범위를 추적한다.

consumer의 held registry close 실패는 UI session 정리에서 먼저 멈춘다. 이를 product parent.close 자체를 실패시킨 증거로 확대하지 않고, 그 경계는 root lease의 별도 Busy/handle 유지 회귀와 구분한다.

SDK 소유 root 검사는 foreign scope 등록 거부, registry borrow 중 close Busy/handle 유지/명시 재시도, context borrow 중 비실행, ticket별 요청 상관 ID와 CAS/idempotency/timeout 옵션 보존, Native4 capability의 role-separated pin을 포함한다. 같은 SDK/store를 쓰는 consumer 검사는 기존 fixture/assert/budget을 바꾸지 않고 승인된 document_factory test 파일에 제품 lease 위임과 새 검사만 추가했다. 아직 실행 결과로 인수하지 않았다.

## 현재 검사

root locked focused는114213ms/exit0, 실제4pass·0fail·0ignored·282filtered(본문0.04s)다. foreign scope·recoverable close·ticket context와 manifest pin 역할을 확인했다.

첫 consumer all-target check15132ms/exit101은 새 검사에서 registry borrow를 host 필드로 직접 잡은 E0502였다. 첫 source를 보존하고 같은 Rc를 별도 지역 변수에 clone해 의도한 runtime borrow Busy를 검사하도록 수정했다. r2 locked check5568ms/exit0이었다. consumer lock diff는 기존 kinekino-player package dependency에 meari/player-ui-plugins 두 edge만 추가했으며 버전 목록을 임의 통일하지 않았다.

이어진 같은 shared native target의 consumer 선택 실행1865ms/exit101은 player-ui-plugins의 Serde E0277/E0308 12오류로 테스트 전에 실패했다. -vv 재현1564ms/exit101도 같은 오류이며, 타입 변환/allow/버전 임의 변경으로 우회하지 않았다. root/tool lock은 동일 serde_core1.0.229/source/checksum 하나를 보유한다. cargo tree의 두 serde_core 표시는 normal과 build-dependency의 별도 문맥이었다.

verbose 로그는 Fresh meari 뒤 shared debug/deps/libmeari.rlib라는 해시 없는 파일을 extern으로 참조했다. root/consumer의 Meari fingerprint는 서로 다른 path·dependency hash를 갖고 같은 파일명을 썼으며, 직전 root 집중 빌드가 그 rlib를 생성했다. fingerprint JSON·rlib SHA·실제 extern·feature tree와 전체 첫 오류를 보존했다. 이 근거는 산출물 문맥 충돌을 가리키지만 진단 분리 전 일반 원인 확정으로 쓰지 않는다.

중앙과 조율해 이번 consumer를 `runtime/build-cache/sdk-product-parent-consumer`라는 별도 D드라이브 target에 같은 offline/locked/j1/2CPU 조건으로 비교 빌드했다. 기존 C드라이브 cache와 registry source는 지우지 않았다. 첫 cold build 종료 때 D 여유는277511712768→270530326528bytes로 약6.98GB 줄었고 C 여유는57.08GB 수준을 유지했다. 새 target을 shared-target 혼선의 코드 수리로 표현하지 않는다. 이후 consumer는 그 전용 target을, root는 기존 native target을 사용해 산출물을 섞지 않는다.

격리 첫 선택은975791ms/exit101, compile16m10s였다. 기존 Serde 12오류 없이 test 본문으로 진행해 기존 factory17+panel2가 통과했지만 새1개는 ReviewFiles의 필수 packages/objects/decisions 폴더가 없는 입력으로 초기화 전에 실패했다. 첫 source/NotFound prepare_review_package를 보존하고 실제 빈 store container만 준비했다. 가짜 review package를 생성하거나 unavailable store로 대체하지 않았다.

격리 r2는21958ms/exit101로 기존19pass·새1fail이었다. 새 case의 실제 3store SDK 편집/명시 채택 저장은 진행됐으나 held registry close를 Busy로 예상한 새 assert가 실제 UI session의 Adapter 분류와 달랐다. 기존 document_session::borrowed 계약과 중단 단계를 읽고, 새 test만 정확 Adapter/session_closed=false/parent_closed=false/registration 유지→borrow 해제 뒤 종료로 검사했다. Root parent 자체 Busy 검사는 별도이며 UI 구현/기존 assert/budget은 바꾸지 않았다.

격리 r3는29092ms/exit0, factory18+panel2=20pass·0fail·0ignored였다. 이 입력은 아래 원 요청 ID 수리 전이며 뒤의 새 root/consumer 회귀로 재인수한다.

## 원 요청 ID 검증 경계

읽기 검토 후보를 실제 root의 direct SDK와 직접 제품 lease dispatch로 재현했다. context-red-first는45553ms/exit101, 단일1fail·287filtered·본문0.02s이며 첫 빈 ID에서 direct SDK InvalidArgument와 hash 치환된 dispatch Unsupported가 달랐다. 저장/채택 성공을 증명한 재현은 아니지만, 원 ID가 공통 검증을 통과하지 않은 채 정상 상관 ID로 바뀌는 경계를 입증했다. 원 source와 red 로그를 보존했다.

승인된 최소 수리는 기존 context 조건을 additive RequestContext::validate에 두고 Request.validate가 위임하며 기존 payload object 검사를 유지하는 것이다. assembly helper는 원 문맥을 공통 메서드로 검증한 뒤에만 ticket 상관 ID를 만든다. ID 규칙/문자열 검증을 Meari나 adapter에 복제하지 않았고 필드·기존 오류 문구·safe ID 처리를 유지한다. 유효한 호출의 동작과 달리, 원래 invalid인 직접 등록 경로는 실제 재현에 따라 거부를 강화했다.

context-green-first는43857ms/exit0, 실제6pass·0fail·0ignored·282filtered·본문0.07s다. 빈 ID·제어문자·129자 ID의 direct SDK/lease 거부 동등성, 유효 context의 동일 ticket 안정성/옵션 보존을 확인했다. lease 검사는 책임별 tests.rs/context_validation 하위 모듈로 분리했으며 기존 검사의 토큰을 유지했다. 이후 root strict55270ms/exit0(수리 없음)과 동일 최종 source의 root-full-first313867ms/exit0이 통과했다. root lib286pass·2ignored를 포함한 현재 workspace/integration 전체는385pass·0fail·19ignored다. 제품 plugin이 default member에 추가된 범위이므로 과거373pass와 같은 범위로 비교하지 않는다. Consumer의 최종 독립 target 전체 회귀 consumer-full-final207462ms/exit0은 실제47pass·0fail·0ignored다. Consumer strict-final382830ms/exit0도 통과했다(수리 없음). root fmt-final1294ms/exit0 및 consumer fmt-final309ms/exit0을 확인했다.

최종 root fmt/test/strict와 독립 consumer 전체 회귀/strict/fmt는 통과했다. root bins-final2364ms/exit0도 확인했다. 실제 NativeWindow·새 CLI 프로세스의 저장 재열기·provenance 원자 결합 성공은 이번 범위가 아니다.

공개 요청·응답·저장 포맷과 안정 ID 변경은 이번 추출에 포함하지 않았다. 최종 diff에서 commands의 validation 메서드 추출과 assembly 등록 외 DTO/store 변경은 없고 기존 request/reply 입력 파일 SHA도 동일하다. RequestContext 직렬화 필드/옵션과 오류 문구는 유지하고 invalid 원 문맥의 직접 dispatch만 거부를 강화했다. 이번 변경에는 데이터·설정 migration이 필요하지 않다. adapter import 정리로 provider source hash가 달라져 새 pin을 사용한다. 후속 provenance 원자 결합의 데이터 버전·fresh CLI reopen/tamper는 별도 checkpoint다.

## 최종 인수와 반환

root-metadata-final과 consumer-metadata-windows-final에서 kinekino-player/meari/player-ui-plugins의 package ID·source·manifest path·version을 대조해 차이가 없었다. root member/default member는 각각7개이며 Meari는 제품 workspace member가 아니다. consumer의 전 플랫폼 metadata 첫 조회는 offline 캐시에 없는 fontconfig-parser0.5.8 때문에 실패했고 stderr를 보존했다. Windows filter 조회는 성공했다. 이를 Linux 실행 검증이나 전 플랫폼 metadata 성공으로 확대하지 않는다.

이전 consumer manifest와 제품 manifest의 Native4·API/version·plugin ID·role prefix·request/reply/error/contract 입력·assets 구성을 소스 대조했다. 도구가 제품 함수를 직접 재수출하므로 현재 함수 본문은 한 곳이며 기존 consumer 회귀와 root manifest 회귀가 통과했다. provider 해시는 변경된 adapter 소스를 반영한다. 앞서 명시한 partial schema/provider pin 한계는 남는다.

최종 root src diff는10파일477추가22삭제, consumer src는2파일8추가76삭제, consumer tests는document_factory1파일152추가이며 기존 root tests 차이는 없다. 원본과 final source snapshot, diff, final-source-inventory.csv, final-pins.csv를 증거 루트에 보존했다. main/API 부모/application·domain proposal 입력의 전후 SHA는 같고 consumer manifest 및 Meari manifest도 기존 pin과 같다. 제품 plugin manifest의 독립 workspace 제거와 root/consumer lock 최소 변경만 앞서 기록한 범위로 인수한다.

최종 GUI pin은 `b2ccff622b273999d74c92ad5928e94ef87c142926426574a4690dd391f02a9d`, CLI는 `30d20dbab6bb8ad6c8f58abffac032bcbb5705c19b17d55ebf272499c0f0b9e1`이다. source inventory SHA는 `082c4035d80cacda86dcada6fbb18dbccd25520534297cdf23ad2d7f8225ad59`다. 실행 파일은 final-bin에 별도 보존했고 이 pin으로 실제 GUI 실행/시각 인수를 주장하지 않는다.

final-process-inventory.json은 runner/cargo/rustc/player/player-cli 프로세스의 빈 배열을 관측했다. final-disk.json의 여유는 C51532513280bytes, D268062564352bytes다. 기존 cache/registry와 실패 증거는 지우지 않았다. 정리 명령의 첫 빈 pipeline은 파일을 만들지 않아 Get-Content만 실패했으며, 배열을 명시해 재조회·기록한 최종 결과가 빈 배열이다. 제품 동작이나 테스트 실패로 분류하지 않는다.

이번 parent 등록 checkpoint의 실행과 소스 수리를 종료한다. SDK 소유 root Cargo/lock·assembly/context 및 consumer 수정 입력의 동결과 single-heavy 실행 슬롯을 PM에 명시적으로 반환한다. 다음 fresh CLI/proposal/project/provenance의 module 등록·원자 persisted binding·재조회·accept 재요청·project capture는 담당을 유지하되 PM의 다음 파일 경계/실행 배정 뒤 진행한다. 제품 전체 목표는 완료 처리하지 않는다.
