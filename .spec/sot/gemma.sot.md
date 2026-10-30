---
id: sot:kinekino/gemma
title: Gemma 엔진 후속 기술 소유권 인계 준비
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Gemma 엔진 후속 기술 소유권 인계 준비

2026-10-19. root의 중앙 PM 전환에 맞춘 **수정 계획**이다. [현재 인계](../../../../../../runtime/coordination/gemma-context-boundary-current.md), [startup 구현](../../../../verification/ai/generation-gemma270-startup-lifecycle-implementation.md), [server receipts](../../../../verification/ai/text-server-receipts.md)와 현재 Revision/보존 도구를 읽었다. vendor·engine-lab·model-spike·SDK·제품 source를 수정하거나 build/추론/다운로드를 시작하지 않았다. live handle의 상태는 root 기록/전달값이며 이 세션에서 poll하지 않았다.

## 소유와 잠금 경계

root가 live handle 모니터와 자원 admission·단일 추론/취득 worker 집행을 계속 소유한다. 본 세션은 엔진 후속의 source/worker pin·receipt·통합 계획과 모델 검토 준비를 인수할 수 있다. 현재 model 검토는 **제품의 비교→사람 수정→명시 채택 연결과 통합 검증 뒤 마지막 단계**이며 먼저 사용자 평가를 요구하지 않는다.

36328 CLI build(cargo57940, 마지막 전달 rustc39548)와80163 core/CLI strictClippy가 terminal되기 전에는 vendor/engine-lab source·도구 EXE를 변경하거나 빌드를 재시작하지 않는다. 이후 상태는 root가 제공한 최신 receipt로 판단한다. 완료 handle을 다시 poll하거나 새 native cargo를 병렬로 띄우지 않는다.

고정 source는 FEA `fea352797c0a162f22407fdfc6323cd52ff2967306ca7c5ec420e4f326c5bd87`,24개 변경 경로/설정4개,patch `d61faa0e331fbab5a0f77b47d5cac728bbb0e2502289759111902b3d7b6b0ed7`다. core1385pass/5ignore·check/fmt 통과는 SOT의 실제 상태이며 새 EXE/모델 실행 통과와 다르다. 현재 SDK의 boundary-observation v3 변환은 별도 소유 세션이 개발 중이므로 임의 수정·동시 build하지 않는다.

## 실행 순서와 담당 가능한 변경

| 단계 | 본 세션이 담당 가능한 구현/검토 범위 | root/타 소유자의 실행·선행 조건 | 종료 조건 |
|---|---|---|---|
|1 새 EXE 보존|terminal build result의 bytes/SHA/source pin 검토,engine-lab cli_snapshot에새StartupLifecycle stage·원문로그/EXE보존 연결|root가80163/36328 terminal·로그·EXE를확정. 그전수정금지|새archive manifest와SHA/size·전후동일성. 기존97a3/R1/fixed/baseline불변|
|2 Revision/worker pin|model-spike infrastructure/generation_engine.rs의별도StartupLifecycle Revision·engineID/path/hash,명시applicationcommand/허용profile·traceplan허용분기|새EXE SHA실측뒤만삽입. 현재InternalTraceDiagnostics를대체하지않음|fallback없음·Gemma전용·누락/hash불일치거부·고정명령/plan·관련Rust gates|
|3 worker선보존|generation_trace의schema3/lifecycle수집·explicitshutdown receipt연결·spike_build_snapshot 입력확인|현재worker01e7...68tests/206inputs receipt는기존관측. 새수정에는새hash/gates필요|새workerEXE/compiledinputs·요청/모델/engine hash·실행계획동결|
|4 실제동일3case|실행전계획/판정계약·실행후source/raw/HTTP/lifecycle대조|**root 한 worker만** 자원admission후실행. 이세션새추론없음|startuppermit/drain/close와cohort3개actual완결;실패원문보존·무시/재시도통과금지|
|5 RunReceipt 통합|launch/shutdown/raw전후SHA와outermeasure실exit를SDK형식에연결하는adapter/source계획|SDK v3freeze와계약확정,root실제worker종료receipt|실제child와outerexit분리;필수증거없으면strictjoin실패|
|6 runtimecontrols 후속|Engine::new/pipeline/loader의최소read-onlycallback 설계·후속patch인수가능|현재FEA live/build를먼저봉인. 새hook은별도sourcepin/compile/EXE|configured/resolved/unavailable명시,미관측상수금지. 새actual필요하면root예약|

현재cli_snapshot은R2진단까지,model-spike Revision도Baseline/BoundaryFixed/InternalTrace/InternalTraceDiagnostics까지다. 새enum·명령은위계획의제안이며이미존재한다고안내하지않는다. exactSHA/size는새빌드결과에서확정하고기존상수를복사하지않는다. helper/엔진보존도구를수정할때도도구자신의source/EXE를새로선보존한다.

## receipt·공통 DTO 연계 시 주의

RunReceipt.worker_exited/exit_code가child인지outer인지계약을맞추고,outerworker가실제로끝난뒤supervisor측정으로조립한다. 미래exit0을worker가자기선언하지않는다. 기존LocalTextServer수집기에는명시argv/envoverride·JobClose/wait·raw전후SHA가구현·68tests통과했지만**새모델서버actualreceipt는아직없다**. 정상종료와ownedtermination을구분하고마지막flushACK/parseable/complete를혼동하지않는다.

SDK pathdependency를붙이면공통crate의모든source/Cargo.lock/feature/빌드입력도spike_build_snapshot에포함해야한다. 기존spike206inputs만으로새외부crateclosure를보존했다고하지않는다. strictdecoder/schema3/startupsidecar/독립TrustedPins/외부HTTPjoin/Observation1 verify는SDK소유자와형식freeze후연결한다. synthetic통과는실제capture/provenance/비간섭통과가아니다.

speculative/prefix/scheduler/engineflavor의실제값은현재receipt만으로모두관측되지않는다. [runtime control 증거 설계](gemma270-runtime-control-evidence.md)의callback이없으면unavailable로남긴다. argv에draft가없다는이유로false를발명하거나RAYON/OMP/MKL4를OS전체threadcap4로표현하지않는다. 새3case의schema3lifecycleactual을먼저확인해도이gap이자동해소되지는않는다.

추가hook/receipt로재실행이필요하면source/EXE를별도로고정하고root가새label·자원·단일worker를승인/집행한다. 진행중작업을덮거나quality3task max256을boundary max2와혼합하지않는다. 전체sensoroff rawIDs한계와terminalwitness/B↔C대조는별도잔여다.

## PM에 보고할 상태와 다음 의존성

후속 읽기 관측: `runtime/ai-spike/engine-builds/gemma-startup-lifecycle-cli-build-r3/result.json`과 cargo-status.json이 생성돼 exit0/2919.2819997초,99084288B,EXE SHA `b7ece1736cf49d85523fe514cd09fb65bb1e3912c8f634677e1264196462a7f8`,reviewed_source_hashes_verified_after_build=true를 기록한다. receipt의cargoPID는93572다. 이전 PM 인계의live PID와자동동일시하지않고root가소유세션terminal/최종Clippy와연결해야한다. 이세션은binary를새로hash하거나copy/실행하지않았으며archive완료로표시하지않는다. 새로운terminalresult발견을PM에즉시전달하고남은동결을해제했다고가정하지않는다.

- 준비완료:sourcechain·매핑·runtimecontrol·terminalwitness연구,모델9표본결함/최종검토필요데이터 정리.
- 실제완료근거:FEA source고정/core1385pass5ignore/checkfmt,workerreceipt수집기68tests는각SOT에따른상태. 이세션의직접실행으로합산하지않음.
- 다음의존성:root의최종Clippy/CLIterminalreceipt와새EXE SHA/size,SDKv3freeze. 이것이오기전engine-lab/vendor수정이나build재시작없음.
- 미완료:새EXEarchive/Revision/worker핀·actualschema3/lifecycle·resolvedcontrols/RunReceiptstrictjoin·비간섭. 품질모델승인0은유지.

본 문서는기술소유권이관준비와수정책임계획이며즉시수정/추론을수행한증거가아니다. root는PM/핸들/실측집행을유지하고본세션은이후명시배정된source수정범위를인수할준비가됐다. 공통정본·현재인계문서·제품/UI·Pages를중복수정하지않았다.
