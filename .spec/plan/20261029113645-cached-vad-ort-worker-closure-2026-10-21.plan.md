---
id: plan:kinekino/20261029113645
title: VAD P1 ORT worker·artifact·배포 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VAD P1 ORT worker·artifact·배포 경계

status: active, source-only 조사/제안. P0r3/S22는 동결하고 코드 확대·Cargo·추론·다운로드는0이다. [P0 제품 계획](cached-vad-product-slice-2026-10-21.md) 뒤의 별도 조사이며 실행 인수가 아니다.

후속 P0는 r3 strict101 보존 뒤 r4 source424B04C6의 실제 계약11을 통과하고 heavy를 반환했다. 그와 별개인 P1 wire/executor 최소 연결 제안은 `runtime/coordination/vad-executor-wire-r1/CONTRACT.md`다. SDK identity5 source를 실제 읽어 SHA 경계·원 owner·단일호출·deadlines를 구체화했으며 새 worker 구현/실행은0이다. generic expected_source_sha256의 descriptor/WAV 의미는 SDK 합의 전 미확정으로 남겼다.

## Outcome

검증된 private VAD 경로의 ORT 의존을 명시 adapter/worker로 격리하고, 기존 artifact 읽기·owned cleanup·공통 publication에 연결할 실제 source 후보와 배포 잔여를 고정한다. private 도구 일부 공개나 사용자 머신 캐시 경로를 제품 설계로 승격하지 않는다.

## Authority

| 책임 | 현재 경계 |
|---|---|
| 모델/PCM/chunk 정책 | 동결 media-vad P0 application/domain. 엔진은 이를 복제하지 않음 |
| ORT 호출/worker protocol·artifact reader 후보 | 미디어 담당 source 조사. 신규 코드 배정 전 구현0 |
| 원 attempt identity/Retained owner·host lease·shutdown ACK/retry | SDK 공통 jobs 소유. proposal의 추가 stop_owned 경계 포함 |
| 공개 source/dependency/license·런타임 배포·실행 슬롯 | PM. native 재배포 closure 미인수 |

## Evidence

추가 좁은 배포 조사는 [native closure의 공식 근거/잔여](../../../sots/player/infrastructure/inference/research-followups/vad-native-distribution-closure-2026-10-21.md)를 따른다. 고정ort-sys dist.tsv에서 f99 archive의 ORT1.31.0/windows-x64/directml 선언을 연결했고 공식버전LICENSE/TPN·NuGetDirectML별도License·VCredist문서를대조했다. 이것은archive/lib/buildcommit/licenseclosure의동일성증명이아니며제품배포승인은여전히미완료다. 실제바이너리/모델다운로드·설치·추론은0이다.

2026-10-21 실제 local registry ort/ort-sys2.0.0-rc.14의 Cargo/license/session/build, frozen r3 tool 소스, root immutable storage와 publication을 읽었다. 정확15개 문서/소스의 SHA manifest는 `runtime/coordination/vad-p1-engine-closure-r1/read15.json` SHA `0F99AA415D46BF12458C0766E560746FC6EA6FD850B18A2183B6A48A01EF2661`이다. 현재 Rust wrapper license는 MIT OR Apache-2.0이며 실제 LICENSE-MIT/APACHE를 읽었다. 이는 native ONNX Runtime/DirectML/VC runtime의 권리를 대신하지 않는다.

기존 검증 정본의 주장을 다음처럼 확인했다. 새 성능/추론 결과는 없다.

| 기존 주장 | 판정·원 경로 | 후속 검증 |
|---|---|---|
| bounded VAD는 legacy4thread와 별도 CPU2/1이다 | 확인: r3 inference/mod.rs의 run_bounded_vad는 CPUExecutionProvider·intra2/inter1, 기존 run은 intra4다 | 제품 worker에서 actual provider/thread 관측을 별도 보존 |
| CPU 실행인데 DirectML import가 필요했다 | 확인 범위: r3 build.rs는 Windows MSVC+ORT_LIB_PATH일 때 rustc-link-lib=DirectML. 원 r2 DMLCreateDevice1 링크 실패/r3 성공 근거와 일치 | 이것으로 DML EP 사용이나 portable 배포 완료를 주장하지 않음 |
| 원 스파이크 manifest를 제품에 그대로 쓰면 다운로드 기능이 들어온다 | 확인: tool의 ort features에 download-binaries/copy-dylibs/tls-rustls. local ort 기본 features에도 download-binaries/copy-dylibs가 포함됨 | 새 adapter manifest는 default-features=false와 필요한 std/api-31만을 후보로 하며 actual dependency tree/링크를 후속 gate에서 확인 |
| 모델을 읽은 후 같은 path를 재개방하지 않는 API 후보가 있다 | 소스 확인: local ort SessionBuilder::commit_from_memory는 CreateSessionFromArray를 호출. commit_from_memory_directly는 InMemorySession 수명에 bytes를 결속하는 별도 API | 기존 실제 VAD는 commit_from_file이므로 memory 경로 실제 로드/외부 tensor 불필요/동등 결과는 미검증 |
| peak query가 cleanup을 대신하지 않는다 | 확인: r3 bounded_vad는 원 Supervisor/FinishOutcome/Incomplete를 보존하고 Unfinished가 original Incomplete<WindowsFamily>를 소유 | P1은 이 원 carrier를 SDK thread/WorkerHost/ExecutionSession까지 유지해야 함 |

직전 cached native directory `.../f99aaef23f12986abb05924c944d8433521263ad756b462df762a5a982c98405`의 직접 파일목록에는 onnxruntime.lib359948106B와 DirectML.dll18527776B만 있었다. 이 폴더에서 LICENSE/NOTICE/THIRD_PARTY 고지를 확인하지 못했다. 원 library/DLL hash·실제 loaded module5는 [VAD 실행 정본](../../../sots/verification/ai/cached-vad-resource-adapter-2026-10-21.md)을 참조한다. OS 전체/다른 폴더에 고지가 없다는 주장도 아니며 캐시 파일을 지금 재배포하지 않는다.

모델 pinned README는 license: mit 선언만 있다. 별도 원문·ONNX 변환 계보·모델 재배포 closure를 README 한 줄로 완료 판정하지 않는다. Sintel25초 자료는 기존 실제 입력이며 제품 synthetic fixture와 다르다. 이 조사에서 미디어를 복사/재배포하거나 입력 권리를 새로 승인하지 않았다.

## Unverified

native ORT build commit/config/third-party notices·DirectML 및 VC runtime 재배포 조건·엔진 archive 원출처와 local static lib의 재현 build 연결은 미인수다. Rust crate license가 위 누락을 채우지 않는다. P0 runtime ArtifactPin 하나는 full executable/module closure가 아니다. actual runtime/module 관측·model graph precision/type/shape·memory loading/외부 data file 사용 여부도 별도 gate다.

중앙 조사에 따르면 mask3은 같은 물리 core의 SMT pair였고 mask5는 서로 다른 core 후보다. 현재 runner EEA9의 mask3은 유지한다. 새 물리 분산 profile·가속 수치·물리core2 보장은 구현/검증 전 사용하지 않는다. 기존 intra2는 thread 설정이며 물리core 분산 증거가 아니다.

## Phases

### P1-A: 후보 source와 공급 경계

다음 경로는 미래 구현 후보이며 배정된 exact source 수가 아니다. `packages/vad-ort-adapter/src/{lib,session,worker_protocol}.rs`와 `src/bin/vad-worker.rs`가 ORT/API 타입·tensor/session·single finite worker를 소유한다. native 링크가 필요하면 같은 package build.rs만 책임진다. `.cargo` 전역 설정이나 root build.rs, P0 domain에 ORT를 넣지 않는다.

`media-vad`의 향후 infrastructure artifact reader/worker executor/output publication은 별도 배정한다. P0 `VadEngine` in-memory fixture port만으로 원 process owner를 반환할 수 없으므로 임의로 그 port 뒤에서 cleanup 실패 owner를 drop하는 adapter를 만들지 않는다. SDK typed outcome 합의가 P1 실행 구현의 선행 조건이다.

Static 후보는 기존 검증된 lib/import 방식의 연속성이 있으나 큰 native closure와 DirectML/import/VC 배포 고지가 필요하다. Dynamic 후보는 별도 pinned DLL 공급과 load policy가 필요하고 실제 검증된 동작이 없다. ort load-dynamic은 disable-linking을 켜며 캐시 static lib를 자동 DLL로 바꾸지 않는다. 따라서 static의 읽기 근거만 확보한 상태로 두고 배포 방식은 미확정이다. 새 native payload 다운로드0이다.

완료 조건: PM이 공개 authored source·기존 local dependencies·Rust registry license·native notices/빌드 pin·배포 layout을 서로 구별한 exact 목록을 인수한다. 미인수 payload는 bundle하지 않는다.

### P1-B: reader와 worker 입력

root immutable_storage_paths::{root,file,directory}는 ordinary local root/reparse·share guard를 확인하는 기존 infrastructure 후보다. 제품 reader는 opaque artifact ID를 registry에서 해석하고 원 file handle로 bounded actual bytes/SHA를 확인한다. 검증 뒤 `PathBuf`만 엔진에 넘겨 같은 파일을 무보호 재개방하지 않는다. 이 module을 복사한 두 번째 공통 정본을 만들지 않으며 SDK와 추출/port 연결 범위를 정한다.

모델2.24MB는 검증된 owned bytes를 worker 안에서 다시 hash한 후 commit_from_memory로 로드하는 후보를 우선 비교한다. process 경계를 건너는 transport는 versioned bounded worker descriptor와 attempt 전용 immutable files/기존 input evidence 중 하나로 확정해야 한다. descriptor 자체 SHA와 모델/전체WAV/PCM SHA는 별개다. parent 원 identity는 SDK의 getter에서만 받고 원 requestID 부재는 None으로 남긴다.

worker executable은 parent가 fixed runtime manifest와 실행 전 ordinary file/hash를 확인하고 원 process identity에 결속한다. worker JSON의 executable/hash/cleanup=true만으로 승인하지 않는다. modules 기대 pin과 실제 load 관측을 구별한다. 현재5개 목록은 통제한 non-OS module 근거이며 OS 전체 closure가 아니다.

VAD tensor는 원 chunk512/context64/state256(2×1×128)·input576·sr16000 경로를 유지하되 score뿐 아니라 stateN 길이/dtype/finite와 graph input/output 이름·shape의 명시 검증이 필요하다. 원 스파이크의 `[0]` indexing/임의 Vec 할당을 그대로 제품 API에 복제하지 않는다. CPU2/1, cancel/deadline은 chunk 경계에서 확인하고 cancel handshake1frame은 실험 목적이므로 일반 제품 프로토콜의 고정1frame 정책으로 승격하지 않는다.

완료 조건: 모델 없는 Rust protocol/reader fixture와, 별도 중앙 gate에서 실제 memory load·shape/type mismatch·bounded output·source pin 변경 거부를 확인한다. 실제 추론은 또 다른 명시 배정이다.

### P1-C: 원 owner와 결과 게시

SDK proposal의 run_queued 오류 분기와 stop_owned 최종 unpublished ACK sweep 양쪽이 matching settled proof를 요구해야 한다. Retained 원 owner/원 error/진단 쓰기 오류/host lease를 보존하고 retry admission도 차단한다. Failed나 details label만으로 이 수명을 닫지 않는다. 원 child/family exit/EOF/forced 관측과 engine Complete/Partial은 독립이다.

결과는 원 request/profile/input 문맥을 유지한 candidate를 검증하고 original settled cleanup 뒤 common publication CAS를 통과한다. media-file-publication은 create_new→write/sync→hard_link의 no-overwrite 경로를 제공하지만 호출만으로 guarded root·job CAS·원 cleanup이 증명되지는 않는다. 기존 infrastructure guard와 함께 조립한다. 파일 게시 뒤 record CAS 실패는 orphan/recovery 근거를 남기고 새 attempt로 조용히 채택하지 않는다. P0 Complete/Partial 타입은 그대로 후보 의미이며 제품 지원false를 유지한다.

완료 조건: SDK typed owner/identity·diagnostic 지속성·CAS/stop/retry 회귀 인수 후 실제 registry+CLI normal/cancel을 별도 검증한다.

## Rollback

현재는 조사 문서와 read pins만 추가해 동결 P0/source/S22/공통 jobs/Cargo/index를 바꾸지 않았다. 미래 adapter 실패 시 원 source/native pin과 실패를 보존하고 registry에는 등록하지 않는다. 원 owner가 Retained면 rollback도 owner를 버리는 수단이 아니다. 원 immutable evidence/output은 덮어쓰거나 삭제하지 않는다.

스킬 적용: modoo.today code-change-review-verification 본문과 references/verdict-table.md(SHA `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`)를 읽어 위 **기존 주장 검증 표**에만 confirmed/미검증·실제 call path/다음 검증을 적용했다. 새 구현 후보는 별도 phases로 분리했고 리뷰 확정 결과로 표시하지 않았다. runtime-bridge/skill-change-security-review 검색 결과는 현재 배포 조사와 달라 채택하지 않았다. 기존 modoo plan-authoring의 여섯 절과 local architecture trade-off 원칙을 적용해 static/dynamic 선택을 미확정으로 유지했다.
