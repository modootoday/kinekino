---
id: evidence:kinekino/20261029114647
title: SDK owned execution S1 실행
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK owned execution S1 실행

[HP 계획](../../../plans/player/historical-import-pending-integration.md)과 [공유 Rust port](../../player/application/api/sdk-execution.md)를 따른다. UI bootstrap61/strict/fmt 반환 뒤 PM이 S1과 단일 heavy를 배정했다. SDK는 application execution/assembly를, UI는 generic codec/presentation을 소유한다. Meari Core/manifest/lock은 동결이며 Native source는 이 첫 gate에서 소비하지 않는다. 모델/GUI/새 dependency 실행·변경은 없다.

현재 상태: 최신 SDK 소스의 worker11개·assembly3개·별도 actual9+bound3task·strict·root전체420개·별도 CLI actual9/73명령·fmt·bins 검증을 완료하고 소유 프로세스[]로 PM에 반환했다. 정확 receipt/pin과 미완료 범위는 아래 **최신 입력 실행과 SDK 반환**에 있다. 그 앞의 source 작성·미실행·슬롯 대기 설명은 순차 이력이며 현재 상태가 아니다.

## 첫 구현과 실행 전 이력

증거는 `runtime/quality-review/sdk-execution-s1-r1`이다. 첫 product fmt2857ms/exit0, worker-focused-first56725ms/exit0·정확5pass·0ignored·300filtered·본문0.01s를 확인했다. 같은 session92050의 terminal 뒤에만 source를 보완했다. 처음 source를 source-first로 보존했다.

첫5시험은 가짜 ApplicationClient로 다른 thread·문맥 보존·terminal 동일 Arc/ACK 전 slot 점유, queued 취소 SDK counter0·실행 중 Requested 뒤 실제 success, invalid/oversize pre-admission0·foreign owner, wake 실패/owner Drop 후 retained handle·정리 재시도, oversized 원 Response 보존/fault 뒤 추가 SDK claim0을 확인했다. 실제5 capability의 제품 정책·원본9/CLI/GUI/wake/S2 취소 gate의 통과가 아니다.

첫 gate 후 PM 읽기 검토에 따라 size overflow의 limit+1을 exact bytes로 기록하지 않게 공유 계약을 보완했다. EncodingMeasurement를 Exact/LowerBound/SerializationFailed로 나누고 원 Response를 계속 보존한다. worker-focused-r2는54988ms/exit0·정확6pass·0ignored·300filtered·본문0.71s로 encoding fault 회귀까지 확인했다. generic-first는1295ms/exit0·typed_async_sdk_plugin 정확10pass·0ignored·본문0.15s였다. assembly-check-first는11382ms/exit0의 cargo check --lib이며 Native5 조립의 당시 입력을 컴파일했다. 이는 실제5 정책 시험이나 이후 수정의 통과가 아니다.

이후 response metadata/correlation/outcome 검증, exclusive attachment, quiescent release와 문서 재연결 경계를 보완했다. 권한 검사는 같은 mutex에서 None+None 또는 동일 Some authority만 허용한다. release 후 이전 attachment의 submit/peek/ACK/cancel은 거부하고 이전 Drop은 후속 연결을 해제하지 않아야 한다. 문서 unbind는 worker 종료가 아니며 초기 typed-data ACK도 renderer ACK가 아니다.

재배정 전 작성한 Rust 회귀는 release 후 권한 복구·후속 연결 침범 거부, release/submit 경쟁의 단일 승자, 초기 inspect→typed ACK→actual scope attach→unbind→같은 SDK/worker로 재등록·기존 registration Drop이었다. assembly의 metadata fault3종 보존 시험도 당시 실행 전이었다. 이전6/10/check 결과를 새 입력에 소급하지 않고 아래 최신 gate로 검증했다.

`sdk_execution/tests/initial.rs`에는 초기 queued 취소의 SDK effect0·typed 수용 전 ACK 거부·수용 후 raw/typed 동일 Arc·반복 ACK, canonical NotFound 수용과 잘못된 request correlation의 ACK 거부를 추가했다. 잘못된 응답 시험의 trusted handle ACK는 시험 자원 정리일 뿐 정상 initial 수용이나 제품 복구 구현의 증거가 아니다. 이 신규 시험도 아직 실행 전이다.

추가 검토에서 비특권 close도 State::authorize를 재사용하도록 통일했다. assembly fault 시험은 envelope 정상/typed StoredProposal 불일치를 더한4종이며 pump 반복 실패 후 SDK 호출 counter1·원 Request/Response 동일 Arc·Core pending·미ACK 보존을 검사한다. close의 Requested 표시는 terminal wrapper를 새로 만들 수 있으므로 원 payload Arc와 cancellation 상태를 구별한다. 이 변경도 아직 실행 전이다.

첫 실행 전 source-revocation-first 사본을 보존했다(inventory SHA256 F599255F9D59A439EE0FE41FFC82ABFF90EDCCDE47E7351EC529D22854C47EAC). 그 후 concurrent release의 AtomicBool 선검사와 mutex취득 사이에 다른 해제가 완료되면 stale을 반환할 수 있는 경로를 확인해 mutex 안에서 released를 재확인하도록 수정했다. concurrent release32회와 후속 attachment 권한 보존 시험을 추가했다. 첫 사본은 이 후속 수정을 포함하지 않으며, 실제 실행 입력은 이후 별도 사본으로 기록해야 한다. 아직 실행 전이다.

`async_proposals/tests/policy.rs`에 실제 PlayerSdk/ProposalFiles ordinary3task Pending create/inspect/edit/accept·foreign/stale 거부·acceptretry·원 source/raw 보존·fresh SDK 재조회와 미설정 import Unsupported 시험을 작성했다(예상 worker25회). 별도 ignored 실제원본9 시험은 명시 HISTORICAL_VERIFIER_EXE·기존 e00c1726… pin·companion7a7cd05e…/16566bytes를 사용한다.9개 Native Pending import/inspect와 Gemma3task의 bound edit/foreign·stale 거부/acceptretry/보존/fresh configured reopen을 확인하도록 구성했다(예상 worker36회). verifier15초 deadline보다 긴20초 관측 한도를 사용한다. 모두 소스 작성 상태이며 compile/실제 verifier 실행·CLI 회귀 통과의 증거는 아직 아니다. project capture 및 실제 OS wake도 이 시험 범위에 포함하지 않는다.

PM 요청으로 cargo/rustc/resource-runner 프로세스0을 확인하고 SDK heavy 슬롯을 일시 반환했다. Native 담당의 별도 gate 동안 SDK는 application/assembly 시험 source와 계약 문서만 작성했다. 당시 남아 있던 fmt/test/strict 및 실제5/원본9·3task 정책 검증은 재배정 후 아래 결과로 완료했다.

## 최신 입력 실행과 SDK 반환

위 미실행/슬롯 대기는 실행 전 이력이다. PM이 Native headless 인수 후 SDK에 단일 heavy를 재배정했고 다음 원 receipt/stdout의 terminal을 확인했다. 모든 cargo는 offline/locked/-j1, RUST_TEST_THREADS=1, 전용 C target, resource-runner2CPU/BelowNormal 및 descendants job으로 실행했다. consumer/Native GUI/모델은 실행하지 않았다.

| 증거 prefix | 관측 |
| --- | --- |
| revocation-fmt-first |1508ms, exit0, product scoped cargo fmt |
| revocation-worker-first |52249ms, exit0,11pass/0ignored/304filtered, 본문0.05s |
| async-policy-first |1542ms, exit0,3pass/1ignored/311filtered, 본문0.23s |
| native-nine-first |18409ms, exit0, 명시 ignored exact 시험1pass/0ignored/314filtered, 본문17.40s |
| strict-first |35349ms, exit0, cargo clippy --all-targets -- -D warnings |
| root-full-first |351902ms, exit0, 전체420pass/0fail/23ignored, rootlib310pass/5ignored |
| cli-nine-first |43328ms, exit0, actual_nine_binding_fresh_cli_accept_retry_and_project_capture 정확1pass/0ignored, 본문38.38s |
| fmt-check-final |1299ms, exit0, cargo fmt --check |
| bins-final |996ms, exit0, root player/CLI bins build |

worker11개는 초기 failure2개와 release/revocation/concurrent release 및 submit 경쟁을 포함한다. assembly3개는 실제 scope의 initial→ACK→attach→unbind→sameworker 재등록, fault4종의 원 Request/Response Arc·counter1·미ACK 보존, ordinary3task의25회 worker 정책 실행이다. actual9는 고정 verifier e00c1726… 및 companion7a7cd05e…/16566bytes로 Native Pending import/inspect18회와 Gemma bound3task edit/foreign·stale 거부/acceptretry/inspect18회, 합계36회 동일 worker thread를 확인했다. 실제 verifier process가 실행됐으며 모델 추론이나 후보 품질 승인이 아니다.

CLI 별도 gate는 `cli-actual-nine-first`에73개 명령 영수증을 보존했다.9개 import/freshinspect,3task edit/acceptretry와 원본 출처 보존, project capture/freshreopen, scratch 변조·원본 pin 불변 등 기존 Rust harness의 assertions를 현재 CLI로 다시 실행했다. Native worker와 CLI는 같은 application 정책/DTO를 사용하며 이 시험은 실제 OS window/wake 증거가 아니다.

최신 포맷 전 소스 사본 source-revocation-r2 inventory는 BEC03670EFF4691FCA49A05EFA1E6961246C0F23964905692784218D5BD38A2B, 포맷 후 source-formatted inventory는82BE6857E42FAC8E3766D0605BB7B8400F0008BD74362A433A1AF5AB3BF2AAB3다. 최종 parent/Cargo 포함 `final-source-inventory.json` SHA256은 CAF1353C6BF0E850CBF79A6C5192C67D5FF5CA896D0E0FE8AB51FA8FD6406325다. 포맷 이후 검증 중 SDK 소스 변경은 없다.

`final-binary-pins.json`은 CLI203767B91C2A09F327F84383360C5975B17E500912EE1CF101787E9F86D875A4(실제 CLI gate 사본과 동일), GUI CA702B43E97A2D72961EC2F926358B2FF8C0EB9E4B1266AB61C810097E48FB52를 기록한다. GUI는 빌드만 확인했다. root Cargo.toml59d90f58…/Cargo.lock526da706… 및 UI generic5개 pin은 인수된 값과 동일하다. `final-owned-processes.json`은 cargo/rustc/link/resource-runner/player-cli/verifier 이름 기준[]이며 해당 소유 실행은 모두 terminal이다.

이 SDK checkpoint는 shared application/assembly의 S1 근거다. UI15개 후보의 parent export/consumer 실행, 실제 OS wake·렌더/시각 검토, project UI의 비동기 소비, 강한 in-flight cancellation/publication CAS(S2)는 여전히 별도 미완료다. invalid typed terminal의 명시 복구/아카이브도 구현하지 않았고 fail-closed 소유를 유지한다. manifest provider/schema pin은 부분 source identity이며 전체 canonical schema/provider attestation으로 승격하지 않는다. 전체 제품 goal 완료를 뜻하지 않는다.
