---
id: plan:kinekino/20261029113543
title: Typed runtime 관측 생산 최소 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Typed runtime 관측 생산 최소 제안

## root740b 계약 호환·남은 실제 검증 순서

중앙 후속 한정 읽기 배정으로 root `740b144edb75bfb93e1ce9fb370117e7c2dcce9f`를 확인했다. 89c→740b의 VAD/protocol/worker/executor 및 제품 VAD 경로 변경0, 현재 before13의 existing9 SHA 동일/new4 부재, 채택 notice10 SHA10/10 동일이다. borrowed common fixture도94E9F5BA…EC6BC로 동일하다. 따라서 예상27의 계약·소스 이름에 새 불일치는 없으며 실제 BIN 등록을 확인한 것은 아니다. 대조는 gate-plan-r1/root740b-preimage13.json과 root740b-notice10.json에 보존했다. finalr3 raw/source13과 expected27 원문은 변경하지 않았다.

다음 순서는 (1) 중앙 별도 배정 뒤 현재 root/preimage·fresh 자원·원 S/T/BIN 보호 재확인, (2) exact13 합성의 default fmt/strict·JSON no-run·실제 목록과 예상27 대조·선택 실행, (3) 반환 후 별도 native feature 타입/링크/ABI worker BIN 고정, (4) 정확 native 입력·고지·manifest/독립 release 선택을 공급한 guarded load gate, (5) 실제 session/loaded module→응답의 pin/path·미측정 필드None·동일 run 결속, before_inference 실패/취소·재사용 시 stale 관측 차단과 원 guard/drop/owner 정리 확인, (6) 공통 SDK·일반 CLI audio2 제품 통합·사용자 품질 인수다. 각 단계의 선행 근거와 자원 배정 없이 뒤 단계를 실행하지 않는다.

현재 default27은 synthetic metadata/기존 계약 회귀이며 native runtime 실제 관측 인수가 아니다. 미측정 provider/precision/thread·System32 hash/full closure/OS build 지원을 확대하지 않는다. PRODUCT_RELEASE=None, 신규 구현·Cargo·worker·모델·추론·다운로드0, Meari sole-heavy를 유지한다. modoo QA의 source/actual·관측/설정 구분을 같은 기준으로 적용했으며 새로운 source 재검토나 모델 자료 탐색은 하지 않았다.

중앙이 별도 허용한 leaf rustfmt/check13은 edition2024/skip_children=true로 모두 통과했다. 포맷 변경6개는 부모r2 불변인 별도r3에 고정했고 source13 SHA256은 `CCAF56EE2E356E9C5121258DE28BCA5F1B643089896E577E5B396B2319F3E148`다. 작성8·type/list/actual0, root/S/T/Cargo0을 유지한다. 이후 ‘fmt0’은 이 후속 전 작성 이력이다.

최종 source-only r2는 `runtime/development/vad-runtime-observation-r2/source13.json` SHA256 `8D2AF3994E963FD86B8DF0021A2683F1E5925E165C747DBC9CA6E252F214A833`다. 소스 검토에서 engine reset의 미사용 must-use Option을 피하도록 `take()` 폐기 대신 None 대입만 수정했다. r1 원문/manifest를 보존하며 exact13·작성8·fmt/type/actual0은 동일하다.

후속 PM exact existing9/new4 source-only 승인으로 `runtime/development/vad-runtime-observation-r1`에 후보13을 작성했다. source13 `248981D33FFAB7AD90383378002C56A68F20C4B3E504BD34A653D4B7FAEA3926`, before13 `C43528A5DBB78140D1E1884E286EF302A4E775FA144D44EC0AFC7727A325244F`, 작성8/format·type·actual0이다. canonical preimage9 차0/new4 부재, raw13과 HANDOFF에 실제 조립·borrowed fixture·미검증을 고정했다. 아래 제안에서 공유 파일 수정0은 canonical 기준이며 별도 stage9는 승인 범위에서 작성했다. Meari sole-heavy 중 실행·S/T 소비는 없다.

2026-10-23 PM 요청에 따른 실제 source 읽기 제안이다. root89c0fa0/library7fb/lockE469 기준이며 구현 배정 전 공유 파일 수정0이다. Cargo/tool/worker/native/모델/추론/다운로드/S/T0, PRODUCT_RELEASE=None. Meari lexical gate의 heavy 자원을 소비하지 않는다.

## 확인한 현재 경로

`parent_configuration.rs` native branch가 WindowsLoadedRuntimeAdmission→OrtVadEngine→production::run_released_request→run::run_request→공통 encode_worker_response로 연결된다. encoder는 actual_runtime을 항상 null로 쓰며128KiB BoundedOutput과 기존 decoder 최종 검증을 수행한다. decoder는 이미 optional runtime/modules를 지원하고 modules32·case-insensitive ID 중복 금지·canonical_path4096·원 request/descriptor 결속을 검사한다. domain 관측 필드는 pub(crate)이므로 worker에서 임의 struct literal로 생성할 수 없다.

Windows admission의 observe()는 실제 loaded module handle을 보유한 snapshot, ordinary file/ancestor guards, canonical path를 확보하고 pinned native module과 대응시킨다. before_inference에서는 모든 configured native module이 실제 loaded 상태여야 한다. 이 성공 지점을 생산 근거로 사용한다. `runtime_admission.rs`의 concrete 구현 미완성이라는 주석은 현 구현과 달라 해당 책임 수정 때 바로잡는다.

OrtVadEngine의 CPU EP·profile thread 설정은 관측이 아니다. f32 IO도 전체 graph 정밀도를 측정하지 않는다. 현재 코드에서 session의 실제 provider/정밀도/실행 thread 수를 독립 측정한 값은 없다.

## 최소 생산 범위와 수명

최초 단계는 **실제로 열거되어 고정 guarded file과 canonical path가 일치한 configured native modules만** 보고한다(최대16개). pin은 read_verified_artifact로 이미 검증되고 해당 file/module guards가 생존한 자료를 사용한다. 설정 목록 전체를 실제 모듈로 복사하지 않으며 실제 일치 항목만 수집한다. System32 허용 목록은 SHA를 측정하지 않았으므로 보고하지 않는다. worker main image·OS build·full transitive closure도 이번 wire 범위에 추가하지 않는다.

protocol에 modules-only typed 생성 경계를 추가한다. 내부 필드 공개/Deserialize grant/새 wire field는 없다. provider/precision/intra/inter는 모두 None이다. 생성자는 기존 decoder와 동일한 module 개수·중복·문자열/ArtifactPin 유효성을 유지한다. 외부에 노출되는 생성은 worker-reported observation일 뿐 부모 admission 권한을 만들지 않는다.

LoadedRuntimeAdmission에 ModuleGuard에서 optional 관측을 얻는 default-None 메서드를 제안한다. 기존 fixture 구현을 강제 변경하지 않는다. concrete Windows guard가 성공한 관측을 소유하고, OrtVadEngine은 매 run 시작 시 저장값을 비우며 before_inference 성공 뒤 typed metadata만 수집한다. module guard·session/environment의 기존 drop 순서와 원 owner/control/deadline은 그대로다. metadata는 guard 복제가 아니고 cleanup ACK도 아니다.

worker-local 새 run/production 변형은 기존 VadEngine/P0 정책을 수정하지 않고 optional typed 관측 공급자를 연결한다. 기존 공개 함수는 None wrapper로 보존한다. configured native branch와 기존 native serve entry는 새 내부 조립 경로를 함께 소비하도록 연결한다. complete/원 genuine partial에만 준비된 관측을 직렬화한다. engine/admission 실패에는 None을 유지해 후속 관측 오류로 원 primary failure를 덮지 않는다. 새 encoder도 기존 encode_worker_response의 None 동작,128KiB 출력 제한·score 제한·decoder 최종 검증을 공유한다. oversize 경로/응답은 실패로 닫으며 임의 축약·관측 누락 fallback으로 통과시키지 않는다.

## 배정 요청 exact existing9/new4

| 소유 | 기존 경로 | 한정 변경 |
|---|---|---|
| 공통 protocol — PM 배정 필요 | packages/media-vad-protocol/src/domain/mod.rs | 새 typed modules-only 생성 모듈 등록 |
| 공통 protocol — PM 배정 필요 | packages/media-vad-protocol/src/application/mod.rs | 기존 encoder 유지, 관측 인자 변형 재노출 |
| 공통 protocol — PM 배정 필요 | packages/media-vad-protocol/src/application/worker_response.rs | 공통 bounded encoder의 optional typed 관측 소비와 test leaf 등록 |
| Media worker | packages/media-vad-worker/src/infrastructure/runtime_admission.rs | default-None 관측 메서드·주석 정합 |
| Media worker | packages/media-vad-worker/src/infrastructure/windows_loaded_runtime.rs | 실제 일치 항목 projection과 guard 보유 metadata 연결 |
| Media worker | packages/media-vad-worker/src/infrastructure/ort_engine.rs | run별 reset·검증된 관측 보관/회수; session 설정값 복사0 |
| Media worker | packages/media-vad-worker/src/run.rs | 기존 함수 None 유지, 관측 변형·실패 원인 보존·test leaf |
| Media worker | packages/media-vad-worker/src/production.rs | issuer/manifest/reader 검증 공통 경로와 관측 조립; 기존 native serve도 동일 소비 |
| Media worker | packages/media-vad-worker/src/parent_configuration.rs | 기존 guarded native branch에서 새 관측 조립 소비 |

신규4: `packages/media-vad-protocol/src/domain/runtime_observation.rs`(bounded typed 생성+순수 경계 시험), `packages/media-vad-protocol/src/application/worker_response/tests.rs`(None/typed roundtrip·128KiB·binding), `packages/media-vad-worker/src/infrastructure/windows_loaded_runtime/report.rs`(일치한 모듈 metadata projection·순수 시험), `packages/media-vad-worker/src/run/runtime_observation_tests.rs`(complete/partial/failure/반복 run·원 None 회귀). 새 test leaf는 path-mounted 규칙에 맞는 명시 path를 사용한다. 목록 외 파일이나 P0 VadEngine trait·WorkerResult enum·decoder schema·executor·root issuer 변경은 요청하지 않는다.

## 향후 검증과 명시 미지원

배정 후 source-only 작성 단계에서 exact before/new와 raw manifest를 새 root에 재동결한다. 순수 constructor/encoder/projection 및 synthetic engine 소비 시험은 default gate 후보이며 실제 DLL 측정 인수로 계상하지 않는다. 특히 duplicate/count/path/응답 bound, None wrapper 호환, original failure 보존, fresh run 초기화·stale metadata 차단, genuine partial metadata를 검증한다. type/목록/actual0 상태로 인계한 뒤 별도 중앙 default fmt/test/strict 배정을 따른다.

실제 Win32 module snapshot→native session→응답 관측은 별도 native feature compile/load·기존 승인 입력/worker gate 없이는 미검증이다. 실제 provider·thread·전체 precision·OS build·System32 hash·full module closure·guard cleanup ACK·부모 재검증/제품 권한 발급은 지원 범위가 아니다. 그 필드를 채우기 위한 다운로드나 모델 실행도 제안에 포함하지 않는다.

이미 읽은 modoo.today `modootoday/software-qa/code-change-review-verification` 본문9F73865F…D64B/필수 verdict-table9D9619A8…2A9의 관측과 설정·source/actual 분리를 적용했다. 정확 전체 출처 SHA는 [notice 실제 정본](../../../sots/verification/ai/vad-runtime-notice-bounds-2026-10-23.md)에 보존돼 있다. 현재 산출물은 파일 목록·설계 제안이며 구현/등록/실측0이다.
