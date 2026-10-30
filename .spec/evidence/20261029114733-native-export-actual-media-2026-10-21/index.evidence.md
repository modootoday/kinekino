---
id: evidence:kinekino/20261029114733
title: Native export 실제 미디어 단계별 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native export 실제 미디어 단계별 관측

최신 별도 배정 r4는 실제 guarded 제출·native ACK·landing·consume·EOF까지 **1pass**다(아래 r4 절). 이하 r3의 원 실패/forced·child exit 미관측 기록은 그대로 보존하며, r4로 그 실패 원인이 해결됐다고 판정하지 않는다.

## 범위와 고정 입력

root `4e269b1e62ebab094b103024849d43236b8d6bce`의 정상 Native CLI와 최종 SDK integration `EA24C7D3DAE0B5792FDB086D72250128D61A18F04F9C9E41D1EEFC8F976BF7F3`로 지정 ignored 시나리오 한 번을 실행했다. gate는 `runtime/verification/native-export-actual-media-r3`다. catalog16·mpv 원 핀·target5000000/tolerance250000을 유지했다. 응답손실 feature·브라우저·재해석·새 모델 다운로드는 실행하지 않았다.

## 관측

- 정상 CLI build는 원 receipt terminal=true/exit0/160490ms다. CLI SHA는 `B20A9A6D9707C1BB97B5BBDC3D285FF444F5E945B8F6BAE0D4EFB87B6B1601CD`이며 실제 시험용 사본과 PM 재대조가 일치했다.
- 지정 actual1은 ready record 읽기 cutoff로 실패했다. 원 시험 출력은 FAILED 0 passed/1 failed이며 outer receipt terminal=true/exit101/13443ms다.
- `failure.json`의 원 closeout은 Deadline/ownership=live_owner_retained, child_exit=null, family=Empty, forced=true, stdin_eof_requested=true다. stdout/stderr는 각각 0바이트·complete=true다. 이 조합을 정상 종료나 forced=false 회수 성공으로 변경하지 않는다.
- PM의 직후 CIM 조회에서 원 host77068·outer launcher58692·test116584 PID는 보이지 않았다. PID 부재 관측만으로 원 receipt의 누락된 child exit를 보충하지 않는다. 담당의 전체 가족·보호 파일·원 handle closeout 인수는 남아 있다.

담당 후속 마감: 원 Job의 Empty/forcedtrue/child_exitnull·live_owner_retained를 그대로 유지했다. 원 outer2receipt terminal(0/101)·기록5PID 현재 부재·모든 본인 원 exec terminal 및 root1452/snapshot1412/bundle244/input16 source-copy/CLI3사본/SDK/index 보호 차이0을 확인하고 PM에 heavy/shared를 명시 반환했다. 이는 미관측 inner child exit나 graceful 종료 성공을 보충하지 않는다. closeout.json SHA256 `6F7249769CDAE887361A38A228CF13E23E304CD86E3CCAFAA584AD0A962F41C2`, 원 실패/최소 계측안은 runtime/verification/native-export-actual-media-r3/HANDOFF.md를 따른다. 추가 실행·timeout/target/URL/정책 변경0이다.

## 미검증과 다음 조치

영상 요청 이전의 준비 단계 실패다. URL 만료·봇 차단·mpv 재생 문제로 분류할 근거가 없다. 실제 media open/seek ACK/paused landing/list-peek-consume는 도달하지 못했으며 세 카테고리 종단 검증 및 응답손실 복구도 미완료다.

읽기에서 ready 전 전체 mpv path/hash 검증의 입력 차이(실제121051136B vs 과거 inert 약50B)와 서로 다른 정상 CLI SHA를 확인했다. 단계 시간이 없어 원인 확정은0이다. 원 cutoff 오류는 동기 raw cursor.poll 복귀 뒤·다음 poll_child 전에 발생하므로 startup과 관측/스케줄링 지연을 분리하지 못한다. runtime DLL이나 아직 실행 전인 mpv companion DLL을 원인으로 판정하지 않는다. 최소 Rust 원 검증 시간/host stage/raw-read 관측안을 HANDOFF에 제안했으며 아직 새 편집·실행 배정은 없다. 자동 재시도·timeout 증가·target/URL/정책 변경0을 유지한다. 현재 배정은 [실행 권한](../../../plans/player/execution-leases.md)을 따른다.

독립 소스 비교에서 기존 B2는 약50B inert mpv, 이번 입력은121051136B 실 mpv라는 차이를 확인했다. 양쪽 모두 ready 출력 전 `TrustedMpvExecutable::open`에서 전체 동기 SHA 검증을 수행한다. 단계별 시간이 없어 해시 지연을 원인으로 확정하지 않는다. 원 오류는 raw-file 관측 복귀 뒤 cutoff 검사이므로 child startup과 관측/스케줄링 지연도 아직 분리되지 않았다. mpv 자체는 이 시점에 실행되지 않으며 companion DLL 문제로 단정할 근거가 없다. 두 정상 CLI SHA도 서로 달라 동일 바이너리 비교는 아니다.

## 운영 반환 인수

SDK가 `closeout.json`으로 heavy/shared를 명시 반환했고 PM이 원 두 receipt와 해당 closeout을 읽었다. root1452/snapshot1412/bundle244 및 입력16·원SDK/CLI 사본/index 보호 차이0 보고, outer handle0·기록5PID 현재부재와 원 familyEmpty 관측을 보존했다. `innerChildExitObserved=false`, `innerGracefulSuccess=false`를 유지한다. 운영 슬롯 반환은 실패 시험의 정상 종료 판정이 아니다. 추가 실행 없이 startup 진단안을 준비한다.

## r4 — 검증된 새 checkpoint의 실제 media/export1

2026-10-21 root **c7fe45510d05ba6297240354abc149ae46cc6f66**, [union21/help](native-ready-help-union-2026-10-21.md) 및 [실제 ready/EOF1](native-ready-only-2026-10-21.md) 인수 뒤 PM이 별도1회를 배정했다. Fresh gate `runtime/verification/native-export-actual-media-r4`; 같은 정상 CLI **F7D7D9AB2A3A935997AC663AF1BA36C6BC1743BE4777E4ADF077AFA0327C7310**/30148096B, SDK **DEF882AA42D6336D2D2DC3D49EEDC553AC832E45266F64F0D6DBBEC88B9D3D0F**/5826560B. 실제 feature native-proposal-window. 원 catalog16/JSON79478334…ec8e31a0/artifact5c23ef…128aa3/mpvF8BF121051136B/target5000000µs/tolerance250000µs, total90/cleanup10 및 기존 phase 예산을 유지했다. 새 build/source/input/URL/allowlist 치환·warmup·예산 증가·자동 재시도0이다.

기존 exact ignored actual_export::actual_original_guarded_export_ack_landing_and_consumption **1passed/0failed/39filtered**, outer terminal0/8335ms, libtest8.32s. 이 1pass는 아래 단계들의 한 시나리오 인수이며 단계마다 별도 pass로 중복 가산하지 않는다.

| 실제 단계 | 원 관측 |
| --- | --- |
| Open/원 binding | playback.open 접수는 completion unobserved, engine1·원 resolution identity. 이후 실제 native observation에서 connected/paused/duration을 확인한 뒤 진행 |
| 원 guarded submit1 | 수동 interval·원 accepted annotation rev1/verified1/request rev3/pass1, actual target5000000. 원 admission ref/immutable metadata·post_error null. elapsed6293789µs |
| Native ACK | peek0의 원 ref/metadata 일치, state command_accepted, write_committed=true, failure=null. elapsed6604352µs. Transport framing ACK 또는 접수 receipt와 구별 |
| 별도 landing | 동일 engine1/resolution, paused=true, position5009733µs: 목표 오차9733µs로 ±250000µs 안. elapsed6731563µs. duration1420002080µs/1920x1080/audio2/has_error=false/volume0. 수동 청취 품질 인수는 아님 |
| Consumption | consume1이 원 terminal 결과 반환, 후속 list items=[], 두 번째 consume은 원 not_found/exit4. 기대한 nonzero를0으로 바꾸지 않음 |
| EOF/원 media 정리 | engine1 engine_exited/ipc_worker_exited/stdout_worker_exited/stderr_worker_exited 모두true. pipe/sdk joined, dispatch completed, poisonfalse. 정상 종료 receipt 존재 |

원 host59324/creationFILETIME134370172262003190는 child exit0/familyEmpty/forcedfalse/stdinEOFtrue. 요청18+host1의 원 family receipts19가 전부Empty/unforced/exit관측이며 두 번째 consume의 기대 exit4만 nonzero다. outer2+inner19 기록21 PID는 현재 부재, 원 exec/handle 회수 완료, PID reopen/kill/타 앱 종료0. mpv 자체 PID를 새로 검색해 종료 근거로 쓰지 않았으며 원 engine 종료 flags와 원 host descendant family 종료 근거를 유지했다.

Native stop diagnostics는 complete=true/read_failed=false/truncated=false, stdout776B/stderr0. copied catalog에 새 diagnostics metadata/body2 파일이 게시됐고 **기존 원16개와 source-copy16 hash는 그대로**다. 디렉터리 전체가 바뀌지 않았다고 주장하지 않는다. Root1457/snapshot1417/bundles532/index/source/SDK·CLI original/fixed/Fixture 보호 차이0. Closeout SHA **5FA4C4A79464FBF21F54DA17F3FF479B13781C65FFFBBA6DB9A70A92ACBE102D**, heavy/shared를 문서보다 먼저 명시 반환했다.

이 checkpoint는 정상 미디어 경로의 원 guarded 결과 wire와 실제 ACK/landing/consumption을 입증한다. **실제 endpoint 응답손실 뒤 원 origin list 복구는 아직 미검증**이며 validation9는 별도 핀으로 남는다. 세 category/provider 광고차단 종단·회차전환·장시간·사용자 품질·수동 GUI 인수도 이 한 시나리오의 성공으로 승격하지 않는다. r3 실패의 원인 확정/반증은 없다. 후속 실행 권한은 execution-leases를 따른다.
