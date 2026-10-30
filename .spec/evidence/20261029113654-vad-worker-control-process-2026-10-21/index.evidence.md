---
id: evidence:kinekino/20261029113654
title: VAD worker 원 제어·실제 자식 프로세스 검증 — 2026-10-21
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VAD worker 원 제어·실제 자식 프로세스 검증 — 2026-10-21

중앙 배정 gate `runtime/verification/vad-worker-control-process-r1`에서 고유 **139개**가 통과했다. 일반 회귀129·경로 helper2·실제 자식 시나리오8을 구분한다. Cargo helper2 재실행은 중복 집계하지 않는다. 최종 source39는 반환 뒤 PM 책임 commit `048808af6d10c0329637bcd003e8cb3482987c89`로 채택됐다. production native/trusted release 지원·모델 품질·전체 제품 완료를 뜻하지 않는다.

## 입력과 최종 채택 자료

실행 baseline은 root `561ae92601ad0929b941bbcfdc27a6e065b06ccb`, library `0b9ea031ce5c08a5bf99ed9852b668cf2bfce038`, root/S lock `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2`다. 원 후보57db/9c와의 root 변경 경로 및 최종39의 교집합은0이다. 기존 경로의 원 preimage를 실제 대조했다.

| 고정 자료 | SHA256 |
|---|---|
| SDK control12 | `59C460D0AEAEE183EE8DEA217F682A38E1EDDF49F265BAE259DB05CECAADEC1E` |
| Media child17 | `9D95D7255375A5268B634AEF0BD0250E3963BD0B63129A623B123C5DCB88D34F` |
| Media fixture6 | `9F3E191AF8D53AB1D73DA2188F8C8471898766F9C3088636B7833C36054C09C9` |
| SDK parent5 | `A2CEF0E54FDDA92F28DBC447324E496246C45AACF3E554EDCD79000286AC4633` |
| 원 합성38 | `D9361D1801A0D63FC3F3FEF7A1A315336E6A5974BCEF0C4A125F4D031044C9DA` |
| 최종 source39, 기존11/new28 | `E9C690DA803B8F8C69D41A916768024DAE8D2EB120EDC495B8F62E6BC5C2D799` |
| 최종 root-before39 | `530D34F6EBDED3B8A526CA8491677A1EE897D5E5FE2DFC10FB146E3D3314F032` |

원12/17/6/5와 원 실패는 덮지 않았다. [최종 source39](../../../../runtime/verification/vad-worker-control-process-r1/final-source39.json), [before39](../../../../runtime/verification/vad-worker-control-process-r1/final-before39.json), [수리 마지막 차이](../../../../runtime/verification/vad-worker-control-process-r1/final-delta-from-verbatim-r2.json)가 채택 입력이다. 각 원파일과 S 적용 bytes39를 최종 재해시해 차이0이다. native Sink에는 기존 identity의 creation_filetime/executable 관측만 추가했으며 별도 override로 보존했다. 후속 shortened-cap39·release37·UI 후보는 합성하지 않았다.

## 실제 빌드·검사

동일 S/native T와 canonical resource-runner conservative2/mask3/논리CPU2/BelowNormal/descendant Job, offline/locked/j1을 사용했다. 새 target/전체 clean/다운로드/기존 registry 변경0이다. 각 heavy 단계 RAM16GiB·VRAM3GiB·C12GiB 기준을 확인했다. library 미러는 승인479의 변경15/삭제2만 갱신했고 동일 bytes의 mtime을 유지했다. 뒤의 PM 승인 exact1 cache 복구 예외를 별도 기록했다.

원 및 최종 offline metadata의 package tuple 차이0이다. 추가 feature는 worker의 빈 `test-control-process`뿐이고 active ORT0·lock bytes 불변이다. 최종 전체 fmt24=0/4466ms, root·관련 공통7 package all-target strict25=0/1598ms, fixture-feature strict19=0/995ms, Cargo helper 시험26=0/1403ms다. 실행 전 실제 test 목록을 읽어 각 고정 사본을 exact/nocapture/thread1로 선택했다. `cargo test --no-run`을 실제 통과로 세지 않았다.

| 고유 일반 회귀 범위 | 통과 |
|---|---:|
| 과거81을 현재 S에서 재빌드한 고정7 BIN으로 재실행 | 81 |
| SDK control 계약·arithmetic/clock/signal·Windows 원 guard | 11 |
| Media child fallible control/guard/bridge | 9 |
| 기존 entry8 / main diagnostic1 | 9 |
| NativeTransport/동일 owner cleanup 직접 영향 회귀 | 19 |
| 합계 | **129** |

새 경로 helper2는 최종 parent BIN에 동일 소스를 `#[path]`로 포함해 실제 등록·실행했다. harness=false child에서 제거되는 `#[test]`를 실행 근거로 세지 않았다. 상속 parent adapter3은 목록에 있으나 SDK normal11 안에서 이미 실행했으므로 추가로 실행·집계하지 않았다. 최종 parent 목록13은 actual ignored8 + helper2 + inherited3이다.

## 최종 고정 실행 파일

| 역할 | bytes | SHA256 |
|---|---:|---|
| 실제 parent test | 2246656 | `3232FD7BB73BD4C3C01A9A32652B8B64DC00655577FF1E8806893CEC31F8D9AD` |
| harness=false child fixture | 1398784 | `08BD85E4722BF93C893E4164D58853C1A6E6D36ABF02950A3B448AC2850EB72C` |
| production worker auxiliary, 실행0 | 1060864 | `8FED72DFE3567202EFEAC391C3B4A5B2B34FF439C5974ECFCEDA322ED0F68C46` |
| player-cli auxiliary, 실행0 | 30659584 | `22448308342F8E00E3CF685A97CCDBB0E087499BBB94D446A212F03A7D6173F1` |
| GUI auxiliary, 실행0 | 80641024 | `EA5B1DA631936146518CD2C25F9534C480C0746CC6368C8DD22B0B490FAEAF92` |

[fixed-final-binaries5](../../../../runtime/verification/vad-worker-control-process-r1/fixed-final-binaries5.json)에 package/target/profile/features/fresh/original/copy를 보존했다. normal129의 재빌드7 BIN과 초기 parent/child5 사본도 별도로 보존했다. fixture runtime pin은 최종 child 실파일에서 부모·자식이 독립 측정한 값이며 원 request/runtime/manifest와 대조한다. synthetic model/license/profile는 production admission이나 추론 품질 근거가 아니다.

## 실제 자식8 관측

준비 전 원 Deadline은 total8s/operation4s/cleanup4s다. 같은 원 deadline/guard/owner를 유지하고 원 request SHA와 QPC sidecar를 결속했다. 자식은 OS에서 원 live parent/creation identity/QPC를 검증하고 실제 전달된 fallible checkpoint를 읽는다. 새 전체 timeout·wall-clock deadline·정책 복제·PID 재개방/kill sweep0이다.

| 실제 case | 확인한 결과 |
|---|---|
| NativeTransport Complete | 동일 공개 factory/launch_read, scores[0.2,0.7], marker false, 원 transport close |
| RunningCancel | engine.ready 실제 bytes/guard 뒤 원 SDK signal; genuine Partial512/scores[0.2] |
| BeforeEngineCancel | startup ready→원 signal→release, Cancelled Failure, engine.ready 없음 |
| DelayedStart | 원 operation 끝 뒤·원 cleanup 끝 전 release, Timeout Failure, engine.ready 없음 |
| RunningExpiry | signal/release 없이 작업 중 원 cutoff, Timeout Failure |
| SignalFault | 원 cancel.signal 역할에 directory, PermissionDenied Failure; Cancelled/Timeout로 변환하지 않음 |
| BindingMismatch | argv control SHA만 다른 원 입력, SourceExpired Failure, engine.ready 없음 |
| FlushFailure | fixture writer flush BrokenPipe 주입; stdout0/stderr35/child exit8 |

첫 case만 실제 NativeTransport를 쓰며 나머지7은 동일 기존 Supervisor와 원 SDK sidecar/signal adapter를 직접 쓴다. 최종8 모두 family Empty·stdin EOF 요청 true·forced false,7개 child exit0·flush child exit8이다. request/control/runtime pin·creation identity·원 stdout/stderr16·cleanup 영수증을 [actual-child-evidence8](../../../../runtime/verification/vad-worker-control-process-r1/actual-child-evidence8.json)에 연결했다. NativeTransport의 마지막 `poll_close`는 실제 family_empty/streams_eof를 요구했다. regular-file raw 끝까지 읽기·stdin EOF 요청·물리 family 종료는 각각 별도 관측이다. named-pipe EOF나 OS pipe 단절 시험으로 확대하지 않는다.

## 최초 실패·제외·최소 수리

| 원 기록 | 관측과 별도 수리 |
|---|---|
| fmt01 exit1 | parent `#[path]` 소비점에서 하위 clock/signal/tests 경로 해석 실패. SDK 원파일을 보존하고 기존3 모듈의 경로만 명시한 module-path-r1, 후속 fmt0 |
| build08 terminal0, 검증 제외 | SDK가 S 대신 D root cwd로 실행한 오류. 해당 provenance/로그/산출물을 후보 근거에서 제외했으며 실제 시험0. 원본 source/index/lock 차이0 |
| 올바른 S build09 exit101 | 앞08이 같은 T의 executor rlib를 다른 source로 갱신해 새 control import6개가 누락됨. 원6 진단 보존. PM 승인으로 S executor src/lib.rs **한 파일**의 동일 bytes/hash·전후 mtime을 기록하고 현재 timestamp만 갱신해 정확 S 재빌드10=0. 다른 touch/clean0. 이후 각 Cargo 직전 resolved cwd·명시 manifest·target dir를 기록 |
| actual01 exit101 | 자식 current_exe VerbatimDisk가 시험 helper의 ordinary root에서 거부돼 ready 전 exit6. 원 spawn/출력/identity/cleanup(Empty/EOF요청/forcedfalse) 보존. Media verbatim-r2 source2 `C3E82CD7C798F5906AAB3B1766F198EC5967FFEF7DD3343987E1A5EF2C21F949`로 좁은 로컬 표현 변환+실제 file/parent FILE_ID_INFO 동일성·guard 수명·actual BIN/runtime/request/manifest pin을 확인. 전역 guard 확대0 |
| fixture strict18 exit101 | harness=false에서 #[test] 함수가 제거돼 helper module glob import만 unused로 남음. 별도 harness-import-r3에서 import를 두 시험 함수 안으로만 이동. assertion/제어 정책 변경0, 후속 strict19=0 |

Media r2 합성39 `04B20BCEE0B7E39F186E2E4E972247AD7C43A42C0BF58D77348E8750E580736F`와 최종 E9C6의 추가 차이는 **한 파일** `packages/media-vad-worker/tests/support/fixture_executable_path.rs`다. SHA `3F3931C4AE4CE58112F9CBA5143850907EC2EBBF557C2BFB206254B75132F593` → `129EA24A07F536D03763B0E6E375FA2D4C70426A7B2A7FF69240B1C791D6C1ED`. 원Media2는 그대로다.

모든 최초 실패 receipt4와 source/BIN/raw를 보존했다. 실패 case를 skip해 통과 수를 만들지 않았고 최종8을 새 원 evidence root에서 모두 재실행했다. wrong-cwd terminal0은 후보 성공 집계에서 제외했다.

## 보호·반환·남은 범위

runner receipt173 모두 terminal,169 exit0/원 실패4다. 원 exec 세션을 전부 회수했다. root1224/library479/index·docs4/S 비입력3103 해시 차이0, 최종 source39 raw/S 차이0, old BIN97의 고정 사본97 모두 불변이다. 보호된 원본 중 CLI18F9→2244·GUI CF64→EA5B 두 경로만 의도된 auxiliary 재빌드 교체다. SDK의 root source stage/index/commit/push0이며 이후 PM의 책임 채택은 위 commit으로 구분한다.

반환 시 별도 graph의 외부 `cargo test -p compact-index ...`1은 관측만 기록하고 조작하지 않았다. global PID 부재를 원 cleanup 근거로 삼지 않는다. SDK 원 handles/terminal/EOF/family와 tool 세션 반환이 근거다. [slot-return](../../../../runtime/verification/vad-worker-control-process-r1/slot-return.json)으로 heavy/shared 슬롯을 명시 반환한 뒤 이 문서만 저비용으로 마감했다. 이후 새 build/OS/GUI/ORT/model 실행0이다.

고정4s 자연 만료에서 부모 NativeTransport observer와 child Timeout 응답 간 경합 관측은 별도 미실행 gate다. 이후 작성한 자연 만료 source3/union40과 단조 축소 caller-cap source3 모두 실행0이며 이 actual139에 포함되지 않는다. [남은 관측 계획](../../../plans/player/inference/worker-control-observation-followups.md)을 따른다. 생산 native issuer/loader/ABI/배포·실제 미디어/모델 추론·제품 operation 등록·사용자 품질은 여전히 별도다. 원 [ADR](../../../adrs/player/adr-worker-monotonic-control.md)와 [이전 파일계약 actual81](vad-parent-worker-integration-2026-10-21.md)을 현재 실제 process 근거와 구분한다.
