---
id: plan:kinekino/20261029114006
title: Core UX source handoff — 2026-10-25
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Core UX source handoff — 2026-10-25

## build-host 이관용 UI 체크포인트

전체 이관 권위는 [호스트 이관 정본](../player/host-migration-build-host.md)이다. 원본은 `D:/workspace/@myso-kr/com.kinekino.player`, 대상은 사용자 지정 `<원 호스트 저장소>`다. 현재 호스트의 D cache/TMP 정책은 유지하며 대상 C 루트와 혼동하지 않는다. WARP 연결/모드/라우팅 변경0, cloudflared 전송·dots 제어·heavy는 중앙 배정 전0이다.

사용자 최종 결정에 따라 기존 빌드 캐시는 전송하지 않고 대상에서 새로 빌드한다. 각 target/ 및 runtime/build-cache/는 재생성 캐시로 제외한다. runtime/development/의 실제 선택 소스·manifest·정본을 캐시로 오분류하지 않는다. UI 담당의 별도 보존 필수 runtime EXE는 없다. mpv·역사 verifier 등 혼재 필수 EXE의 별도 보존 목록은 PM/SDK 정본을 따른다. 구 검증 EXE/PDB를 UI 재현에 필수 도구로 지정하거나 대상 검증 바이너리로 재사용하지 않는다.

UI는 public canonical/index에 이 후계들을 적용하거나 커밋하지 않았다. Git clone만으로 private 선택 소스가 이전되지 않으므로 아래 경로와 해당 manifest가 참조하는 실제 source 파일을 보존한다. 빌드 캐시나 재생성 가능한 EXE와 구분한다.

- 최신 UI 표시 수리: `runtime/development/product-catalog-final-actions-r1/source1.json` SHA `C445C58B4BFFAB71F2DE94FF7D091D6D16E9359E2222437AF6BAE24DF36C2728`, payload EC11CC1E..., predecessor1/HANDOFF 포함. 다음 factory 검증은 아직 미실행이다.
- 선언/cleanup 수리: `runtime/development/product-start-source-contract-r1/source14.json` SHA `CFE922557A6189769AAEAC00EF411EDD2FB35A28C62FA977768F3C3F03E0A401`, predecessors14 SHA5A20ECB...와 전체14 source. 공통3TSX·공통Rust inventory·상속adapter mod 후계를 빠뜨리지 않는다.
- 원 계약 회귀: `runtime/development/product-caption-contract-tests-r1/{source10.json,expected4-registration-r2.json,expected-additional48.json,planned-exact145.json,REGISTRATION.md,ADDITIONAL-REGISTRATION.md}` 및 참조 source. 등록r2 SHA16CDE9B0...,145 SHA044AF042... . 원expected4의 잘못된 JOB prefix를 재사용하지 않는다.
- 통합·폰트 근거: `runtime/development/product-meari-r10-joint-r1/{HANDOFF.md,SDK-FINAL-REVIEW.md,font-closure12.json,font-admission12.json}`와 위 본문에 연결된 private 선택/전임 파일, `.agents/rules`·`.spec`의 디자인/사용자 인수 정본. 원 UI source1/2 lint 후계도 SDK 선택 chain의 일부다.

PM 이관 정본의 다음 합성375 후보 A09B51AC.../계획190 SHA082C68B4...는 실행 전으로 구분한다. 최종 단일 manifest와 custody의 현재 보관 경로는 SDK가 확정한다. UI가 별도 통합 manifest를 만들지 않는다. source14와source1의 `source`에는 원 D 절대 경로가 있으므로 대상에서 그대로 유효하다고 간주하지 않는다. 원 JSON/해시는 역사 입력으로 보존하고, SDK가 `relative` 및 실제 대상 payload SHA를 대조해 새 실행 manifest/custody를 발행한다. 코드 bytes와 역사 증거를 통째 문자열 치환하지 않는다.

### 중앙 배정 뒤 보존할 재현 명령

먼저 동일 선택 소스의 대상 합성·toolchain·Cargo target/build/TMP/TEMP 경로를 확인하고 기존 SDK recipe의 전체 feature/admission/190 집합을 유지한다. 아래는 원30 factory의 정확 등록명용 재현 명령이며 현재 실행 지시나 성공 근거가 아니다. 최소 source feature만 표시한 것으로 원 SDK 전체 feature 조건을 축소하지 않는다.

경로 대응은 원본 루트 아래 `runtime/development/...`, `.spec/...`, `.agents/...`, 제품 소스·필수 폰트/SVG 자산의 상대 경로를 유지해 대상 루트에 놓는 방식이다. 예를 들어 원 `D:/workspace/@myso-kr/com.kinekino.player/runtime/development/product-catalog-final-actions-r1/source1.json`은 대상 `<원 호스트 저장소>/runtime/development/product-catalog-final-actions-r1/source1.json`이다. 원 JSON은 보존하고 source 절대경로를 갱신한 실행용 manifest는 SDK가 새 핀으로 발행한다. Dot은 대상 PC 수신 준비만 소유하며 원본 설정·전송은 PM 범위다.

```text
Set-Location -LiteralPath '<원 호스트 저장소>'
cargo test -p kinekino-player --lib --features native-proposal-window assembly::meari::viewer_callbacks::tests::native_playback::actual_catalog_root_factory_and_owned_callbacks_accept_nonempty_rows_and_captured_values -- --exact --nocapture --test-threads=1
```

실제 target별 --list 각1등록과 원 producer/executable 확인 뒤 실행한다. 최종190은 이 한 시험으로 대체하지 않는다. 대상 호스트의 CLI/factory·normal binary·GUI 인수, 실제9 사용자 품질·range/영상PTS 연결은 모두 별도 미완료다. 원 r4의29pass/30fail과 source1 미실행을 대상 성공으로 이전하지 않는다.

## 현재 빌드·검증 자원 경로

사용자 최신 정책과 실제 `.cargo/config.toml` 기준으로 target/build cache는 `D:/workspace/@myso-kr/com.kinekino.player/runtime/build-cache/native`, 빌드 임시 TMP/TEMP는 `D:/workspace/@myso-kr/com.kinekino.player/runtime/build-cache/tmp`다. 프로젝트 로컬 jobs1이며 전역 Rustup/Cargo 환경은 변경하지 않는다. runner의 CARGO_TARGET_DIR·--target-dir·TMP/TEMP도 이 D 경로와 일치해야 하며 과거 C gate/runner 경로로 덮지 않는다. HDD 직렬 I/O와 heavy jobs1/CPU2/BelowNormal은 SDK 중앙 배정으로만 수행한다. 캐시 이전 중 UI의 Cargo/cache writer/GUI 실행은0이다.

기존 C 경로 receipt·바이너리 식별은 역사적 근거다. 새 D 합성 바이너리의 producer/source/경로/FI를 확인하기 전 현재 검토 빌드로 재사용하지 않는다. 남은 실제 검증은 Catalog 후계 C445...를 포함한 동일 pin CLI/factory190, 새 normal binary provenance, 이후 중앙 배정 정상 GUI 연결/사용자 시험이며 아직 완료로 계상하지 않는다.

Owner: UI. PM retains root application/index, central Cargo allocation and GUI allocation. Scope is the previously approved core UX and three raw-review display omissions; no new feature scope.

Review candidate and exact composition: [private handoff](../../../runtime/development/product-core-ux-handoff-r1/HANDOFF.md). Product effective55 SHA `3C7729407D44240C83AF68882966CE1C493093BB732BBC62D0B34A798E1CC1F9`; separate Meari effective55 dependency SHA `0CDBADB100F0161031AEDB0CEAB67B3E178B844E336A33D01674D1C4E4E86050`. Individual Rust38 formatting checks passed. Full baseline type/test/Clippy, test registration and visual acceptance have not run.

Authored: episode panel grid recovery; observation-derived fullscreen frame; basic error/uncertainty/retry display; selected/current/connecting episode separation; idle interaction pins and original earliest timer; shared CaptionView and original callback adapters for Viewing/Studio/capture/JOB; original User-close veto result; compact range parts/color-token consumer; raw-review known issues, unverified range state and original decision ID display.

Remaining UI-owned source review: route-specific JOB/recovery and capture foreign/stale export callback assertions; required/type caption negatives; reconcile with the exact PM baseline and Meari field/producer dependencies. These are incomplete and must not be recorded as executed or accepted. Preserve sealed source; corrections require successor overlays and updated effective composition.

PM-requested SDK294/Meari55/product55 read-only reconciliation is recorded in [joint handoff](../../../runtime/development/product-sdk294-joint-r1/HANDOFF.md). Candidate effective367 SHA `D50BDA1C237A0F88FCDB5B804E87D1499AFA541108C96CE162D4703DA5B33945` selects immutable source references without copying sources. All input source hashes matched; SDK/product29 differences and seven direct Meari predecessors are recorded. Central application and outstanding negative tests remain unperformed.

Central gates after allocation: compile and exact test list against the reconciled full baseline; appropriate Cargo checks and original lifecycle/range regressions. Then separately allocated native/Web geometry, retained input identity and actual window behavior. SDK/CLI functional gates precede GUI acceptance. Actual9 user quality approval remains with the user; no review records were written.

The formerly missing caption assertions have now been authored as a test-only successor: [caption contract handoff](../../../runtime/development/product-caption-contract-tests-r1/HANDOFF.md). source10 SHA `3A4490B18570F2F310639AF014E3DA72954C5081C345839D1A7B6D34E0A47D1F`, effective371 SHA `518817E3AE13B78B2038F98DE87BF81C6EAE5630B5F41DB5C1EEE5EB9606D70D`, expected4 SHA `5A148C9E58DDC67B8145ADA36051D1866EE2388BB9EBF03F0CD377B12F1A2B40`. Four original factory paths contain92 required/type/signature negative cases. JOB active/recovery and capture original exports have all-three-action foreign/stale/retired/bounded/closing assertions, unchanged SDK request/ticket assertions and original cleanup. Rust10 individual formatting checks passed; compilation, registration and execution remain0. PM's new CPU-only Meari Cargo input and required font/Web product contracts are distinguished in the handoff; no parent source or Cargo was changed.

Skills, exact hashes, application reasoning and authored test names are recorded in the handoff. Source evidence does not satisfy visual matching or user quality approval.

Contract acceptance requires the Rust producer/default, TSX declaration/forwarding, original binding/signature and actual factory positive/negative tests to be compiled, registered and exercised from the same exact composed source pin. A rules document, helper test or previous packet's pass does not satisfy this chain. PM reports Meari r8 effective57 `0AD233A9145CD820CAD5BC7B7C3F6D0F39C0AC1FD1075CB2A5FAD3A52E9D6004` passed fmt/strict/CoreNative no-run; all44 exact tests remain planned. Product source pins remain fixed pending final validated Meari/preimage comparison, with Cargo/GUI execution still outside this session's allocation.

Central registration preparation is complete as source metadata: [registration map](../../../runtime/development/product-caption-contract-tests-r1/REGISTRATION.md). Use expected4-registration-r2 SHA `16CDE9B0B4A44CEE23E88A1C6D9609C33602833A5F3476479DC87DB0A5933EC3`; it corrects only the JOB module prefix to jobs::route::product_route_tests. No duplicate source registration was added. All source pins remain immutable; actual listing/pass0. Final Meari CPU successor/auxiliary closure reconciliation remains the central gate, with r4 reported pending rather than passed.

PM 배정 추가 시험 목록은 [추가 등록 계획](../../../runtime/development/product-caption-contract-tests-r1/ADDITIONAL-REGISTRATION.md)으로 보완했다. 제품55/계약10에서 기존97 밖의 실제 #[test]48건(일반41/ignored7), source/전체등록명/target/feature를 기록했으며 helper·92assertion은 제외했다. planned-exact145 SHA `044AF0426EA485202BD9E6B5AC9E07A0026B471C6BDBFF852617FECAB8CB0214`는 lib139(일반119/ignored20)+jobs6, SDK88∪factory4 중복1 및 추가48 중복0을 반영한 계획이다. 최종 Meari 집합은 별도 결합 대상이며 실제 등록·실행은0이다. source371/등록r2/Cargo/GUI 변경0을 유지했다.

검증된 라이브러리 r10/CF98FC...와 제품371의 읽기 전용 합성은 [r10 합성 후계](../../../runtime/development/product-meari-r10-joint-r1/HANDOFF.md)에 기록했다. 제품용371 SHA `86C6E9BE5D4BA7561D2C7E667B92E85B31A516605CE1038DF3AD06899AF24AE2`는 기존 Web PointerEvent 선언 Cargo를 보존하고, 별도 CPU 후보371 SHA `3DAA38917798E027F0724BAEF2FD67DAE325603553D77EF8B3DEBFE119A0233E`는 r10 CPU Cargo를 선택한다. source copy/Cargo 파일 수정/실행0, 부모371 불변. 폰트 코드·TTF·OFL 폐쇄12 및 정확 전임57/변경9 연결을 별도 manifest로 고정했다. 제품145와 Meari44를 같은 입력에서 검증하는 중앙 입장은 아직 미실행이다. OS NotReady 수정은 Meari가 본체/API 변경 없는 fixture1로 확인했으므로 후속 OS 증분을 제품에 넣지 않는다.

PM 후속 배정의 최종 결합은 SDK가 작성한 단일00-supply371-product-os1 SHA `4D52B89EF53B84BC5B8A84DF715D4E4D888324E170CB7AF726D3B8EF8386BB8D`에 정렬했다. [SDK 최종 읽기 검토](../../../runtime/development/product-meari-r10-joint-r1/SDK-FINAL-REVIEW.md)에 원371 대비 OS fixture1 전임/실제SHA 일치, 제품 Cargo 유지, 원 font stage7·상속5를 기록했다. 새 source manifest 복제·증분 삽입0.00-exact190 SHA `082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464`는 제품145 집합 차이0+Meari44+별도OS1이며 실제 등록·실행은 중앙 배정 전이다. OS record의 input 전임/producer 의미는 SDK에 전달했고 임의 변경하지 않았다.

4D52 중앙 strict에서 확인된 collapsible_if2는 [최소 후계2](../../../runtime/development/product-joint371-strict-r1/HANDOFF.md)로 수정했다. source2 SHA `E39754AAC3C07670410F83C4CFD17C5A0C68EEF9834C58CE5ED7648911627EC8`, predecessors2 SHA `7FC5A23D4212B28A53FF190A97859557D65E4712F7DC09863D99F526954A359C`. User veto의 원 cause/permitted/borrow 순서와 range display의 shape/단락평가/중복insert 검사를 유지한 조건 결합뿐이다. 개별 Rustfmt2 통과, 자체 Cargo/GUI0·새 시험0. SDK가 단일 후계 합성과 동일190 재검증을 계속 소유한다.

r2 D551의 진단용 Clippy에서 발견된 창 상태 시험 field_reassign_with_default1은 [시험 초기화 후계1](../../../runtime/development/product-window-ui-test-init-r1/HANDOFF.md)로 수정했다. source1 SHA `EB9BBD7212B2AC9B52EF3D691B09E5BF4C2B72DA00A17CBD8A140A753DB9F116`, 원 전임998C12B7...→후계26C04E71... . 원 assertion·mut·production 정책과190 목록은 유지했고 개별fmt/check만 통과했다. 후계의 Cargo/strict/실제시험은 SDK 배정이며 UI 실행0이다.

F0E9의 실제 준비 실패와 독립 range cleanup 실패는 [선언 계약 후계14](../../../runtime/development/product-start-source-contract-r1/HANDOFF.md)로 구분 수리했다. source14 SHA `CFE922557A6189769AAEAC00EF411EDD2FB35A28C62FA977768F3C3F03E0A401`, predecessors14 SHA `5A20ECBFBF249C9432DAA083A1C357CA267BC785442B8B0D1F8BAE05952D2421`. 공통3컴포넌트를 파일1Props/1export로 분리해 공통 Rust inventory를 Viewing/Studio/JOB/capture가 소비하고 Range를 등록된 로컬 source로 연결했다. state/controller/Busy/owner 정책은 유지하며 기존 준비 개수9/16과 원 오류 진단을 맞췄다. range 시험은 원 terminal1/중복거부를 보존하고 원 consume/ACK 뒤 기존finish를 호출한다. Meari 읽기 검토는 완료됐고 Rust7 개별fmt만 통과했으며 후계 compiler/시험은 아직0이다. 원9pass/10fail 및 tail102diagnosticpass/33fail을 다음190 성공으로 이전하지 않는다.

r4/source37541DB의29 exact pass 뒤 원 factory30은 기술 단계 UI 노출 assertion에서 실패했다. [Catalog 표시 후계1](../../../runtime/development/product-catalog-final-actions-r1/HANDOFF.md), source1 SHA `C445C58B4BFFAB71F2DE94FF7D091D6D16E9359E2222437AF6BAE24DF36C2728`는 고급 패널의poll/read/inspect/play DOM 버튼5개만 제거하고 원 assert/diagnostics=true/typed callbacks·자동SDK·190을 유지한다. 일상 transport 설계 정본에도 고급 패널을 통한 기술 단계 재노출 금지를 반영했다. TSX diff 검토 외 실행0이며 캐시 정리는 SDK 단독 소유다.
