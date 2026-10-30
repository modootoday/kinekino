---
id: plan:kinekino/20261029113637
title: Physical source host 단계별 선행 계약
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Physical source host 단계별 선행 계약

후속 B source-only 배정은 SRP 추가4를 승인받아 existing7/new6 exact13으로 작성했다. `runtime/development/source-ipc-b-r1/source13.json` SHA256 `A3B1BF44E32B6DF50A5623282A926A54A007C61657B3D17BD90CE59EBDB005B0`, before13 `D7C15AC1BB92E09C5668EC99FB72D7032E1522A66A9E89E6BEB37D18D2A2E4EE`다. leaf fmt/check12 통과·작성11/type/list/actual0이다. 원 family/LaunchFailure.owner·whitelist를 유지하고 purpose lanes·절대 cutoff·pending buffer/event 수명·비상 Drop 보존을 분리했다. ConnectNamedPipe undefined count는 공식 문서 직접 확인 후 종류별 처리로 반영했다. 실제 soundness/OS/정리 지원은 미인수이며 C/D/runtime13은 불변이다. 구체 범위·비상 leak/동기 OS setup의 한계는 HANDOFF를 따른다.

후속 PM 배정으로 A만 private exact6에 작성했다(최초4+SRP분리2). `runtime/development/media-source-protocol-r1/source6.json` SHA256 `F0496B800F8C7A34EEA5A306507BB3A93E629635821BA8A3BEA348108A9CC7AF`, before6 `5B1CBA23E2747E2DDF77B60706712E0850E4BEA74BB98D19655ADBDA6D5A4D22`. std-only, Rust leaf fmt/check5 통과, 작성12/type/list/actual0이다. lib/domain 값과 codec/state를 분리했고 canonical/workspace/lock/S/T/Cargo는 변경·실행0이다. B/C/D는 계속 미배정이며 A가 replay/HTTP/물리 ACK 지원을 만들지 않는다. 정확 한계와 후속 partial-IO/clock·typed HTTP 계약은 stage HANDOFF에 보존했다.

2026-10-23 PM 후속 읽기 배정. [상위 제안](physical-close-source-host-proposal-2026-10-23.md)의 existing7/new12를 한 번에 합성하지 않는다. 현재는 계획만 작성했으며 신규 코드·Cargo·S/T·process·network0, Media runtime13-r3 불변·SDK sole-heavy다. 각 단계는 별도 exact preimage/manifest·소유 배정·컴파일/actual gate·명시 반환 뒤 다음 단계로 간다. 아래 시험 경로도 제안이며 생성하거나 등록하지 않았다.

## A. 순수 protocol/codec부터 분리

신규 `packages/media-source-host/Cargo.toml`, `src/lib.rs`, `src/domain.rs`, `tests/codec.rs`만 최초 closure로 제안한다. 소유는 공통 계약 담당, root Cargo workspace 등록/lock은 PM 직렬 소유다. 이 단계는 HTTP·OS·worker entry가 없는 순수 codec library다. owned-process에 URL/HTTP/제품 정책 의존성을 추가하지 않는다. 기존 media-contracts의 공통 오류/값 계약은 재사용하되 새 issuer로 사용하지 않는다.

고정할 wire 계약은 version·attempt/request binding·monotonic cutoff·sequence·offset과 bounded Open/Metadata/Data/Credit/Abort/BodyEnd/Failure다. Data payload64KiB, metadata envelope32KiB, parent queue256KiB/최대16 chunk는 제안 상한이며 검증된 운영 지원값이 아니다. URL/header의 세부 수·길이 상한은 envelope 안에서 별도 고정하고 무제한 문자열/맵을 만들지 않는다. length-prefix를 읽은 즉시 상한/checked arithmetic를 검사한 뒤 할당한다. body는 JSON/base64 evidence에 넣지 않는다. unknown/duplicate/version/trailing/truncated message는 실패다.

Open은 한 번만, Metadata는 Data 이전 한 번만, Data는 단조 sequence/연속 offset으로 받는다. credit는 누적 byte budget이며 소비한 만큼만 보충한다. 음수·overflow·재전송 credit·offset gap·terminal 뒤 data를 거절한다. credit0에서 body를 추가 poll하지 않지만 별도 bounded control lane의 Abort/원 cutoff는 계속 처리한다. BodyEnd는 실제 response EOF/최종 byte 수·length 정합이며 IPC EOF는 이 뜻이 아니다. Failure는 작업 실패이고 child exit는 프로세스 사실이다. wire의 Closed/성공 문구로 physical ACK를 발급하지 않는다.

`tests/codec.rs`의 exact case 책임: prefix 최대/최대+1·잘림·unknown/duplicate, sequence/offset/credit overflow·재사용, metadata 이전 data·BodyEnd 뒤 data·failure 순서, 64KiB+1 frame 분할 조합, disconnect와 BodyEnd 차이, operation/end cutoff를 늘리는 message 거절. 순수 시험은 pipe/OS/DNS/네트워크 사실을 입증하지 않는다.

## B. owned-process IPC를 제품과 독립 검증

기존 수정 제안은 `packages/owned-process/src/infrastructure/mod.rs`, `windows/{mod.rs,launch.rs,handles.rs}`, `packages/owned-process/Cargo.toml`, `src/bin/owned-process-test-child.rs`다. 신규 `src/infrastructure/ipc.rs`, `windows/source_pipe.rs`, `tests/source_ipc.rs`. owned-process 담당만 목적 한정 endpoint/whitelist·partial launch·원 Job/handle 수명을 구현한다. 기존 spawn/ProcessFamily/Supervisor cleanup 알고리즘과 stdin EOF 동작은 유지한다. Cargo 변경은 test-support fixture와 필요한 Win32 기능만 검토하며 새 launcher·generic arbitrary handle 상속·URL 정책은 없다. 추가 기존 경로가 필요하면 별도 요청한다.

원 child는 suspended 상태에서 동일 Job에 배정하고 whitelist endpoint만 상속한 뒤 resume한다. 부모 endpoint는 다른 child에 상속되지 않아야 한다. IPC는 bounded overlapped IO로 read/write/cancel을 poll하며 detached pump thread·blocking Drop·PID 재오픈으로 대체하지 않는다. partial-create/assign/resume 실패도 기존 LaunchFailure.owner를 원 core가 보유한다. stdout/stderr raw evidence는 제한 진단이고 body 채널은 별도다.

현재 Deadline은 process-local Instant이며 원 operation/end 두 cutoff를 constrained_by로 줄일 수만 있다. 이를 그대로 직렬화하지 않는다. Windows 공통 monotonic tick domain(단위/boot-domain 포함)의 **절대 operation/end** 전달 계약을 먼저 추가 검토한다. 부모는 tick을 먼저 관측하고 그 뒤 Instant remaining을 계산하여 변환 지연이 예산을 늘리지 않도록 보수적으로 내림/clamp한다. child 수신 시 remaining을 새로 시작하지 않고 이미 지난 cutoff를 거절한다. 부모 Instant cutoff가 최종 권한이며 child 응답으로 연장하지 않는다. `GetTickCount64` 등 실제 선택에 필요한 Win32 feature는 owned-process Cargo의 exact delta로 검토한다. SystemTime/벽시계나 직렬화 Instant는 쓰지 않는다. cross-process clock 동등성은 순수 fixture만으로 인수하지 않는다.

`tests/source_ipc.rs`: 작은 queue의 credit0·64KiB 초과 source 조각 분할·취소 lane, parent/child disconnect, EOF가 family empty 전인 경우, 지연 child/출력 포화, 원 timeout·cleanup reserve 보존, 부분 spawn 실패, receipt 실패 후 원 carrier 보존, 원 endpoint 정리와 test resource 재획득. `tests/polled_cleanup.rs`/`windows_family.rs`는 기존 관련 guard 회귀만 선택한다. 같은 원 handle의 child exit/family empty·IPC/진단 EOF·receipt 저장을 따로 관측한다. timer 반환/Drop/counter0만으로 ACK를 만들지 않는다. 이 단계는 synthetic child만 별도 승인 시 실행하며 HTTP는 아직 없다.

## C. parent SourceOpener와 별도 HTTP child 조립

parent 신규 `src/infrastructure/download/source_process.rs`, `source_process/{admission.rs,stream.rs,closeout.rs}` 및 `tests/download_source_process.rs`, `tests/support/source_process_fixture.rs`를 제안한다. 기존 root `Cargo.toml`, `src/application/download/mod.rs`, `src/infrastructure/download/mod.rs` 등록은 PM 직렬 배정이다. 기존 owned_session/SourceOpener/retaining core·original attempt·output/probe/CAS 정책을 재사용한다. HttpHostOwner::transport의 부모 HttpTransport 참조를 dummy로 채우지 않고 process-backed SourceOpener를 별도로 둔다.

이 단계의 fake replay capability는 test-only fixture 안에 한정하고 생산 constructor로 노출하지 않는다. 원 issuer D가 준비되지 않으면 production preparation은 Unsupported다. prepare는 launch-free, core가 opener를 가진 뒤 첫 open을 spent로 표시하고 launch한다. 원 browser current/attempt/control을 매 read/credit 전후 검사하고 mismatch 시 동일 child를 닫는다. Drop/unwind/lock poison/partial open의 owner는 기존 core로 회수한다. 첫 run 오류·원 panic과 이후 IPC/abort/terminate/receipt 오류를 분리한다. source.poll_close가 원 host를 물리 정리한 뒤 source.close_owned/partial 정리를 진행하는 기존 순서를 유지한다. publication은 child가 수행하지 않는다.

HTTP child는 A에 포함하지 않았던 `packages/media-source-host/src/main.rs`, `src/application.rs`, `src/infrastructure/http.rs`와 `tests/http_lifecycle.rs`를 별도 closure로 제안한다. reqwest/client/runtime/resolver/Response는 child 안에서만 소유한다. send/Response는 post-send validation 이전부터 원 state에 귀속시키고 header 오류·panic도 같은 closeout으로 보낸다. 큰 response frame은 Bytes+offset으로64KiB IPC 단위에 분할하고 pending 소진 전 다음 frame을 요청하지 않는다. upstream-frame 상한1MiB는 상위 제안의 미검증 후보이며 초과는 명시 실패다. 이것을 TLS/OS 포함 hard RAM 상한으로 표시하지 않는다.

`tests/download_source_process.rs`는 fake child의 open-once/실패 후 재발급 금지·원 current 무효화·정상 BodyEnd와 crash·credit/Drop/unwind·primary/secondary·ACK 전 CAS 금지를 담당한다. `tests/http_lifecycle.rs`는 별도 승인된 로컬 서버/지연 resolver fixture에서 header 직후 오류·DNS 지연·frame 분할·cancel·종료 지연·강제 family 종료를 담당한다. actual OS resolver/network 및 실제 provider는 추가 명시 배정 후 확인한다. mock resolver sleep 종료를 실제 OS DNS 종료로 표시하지 않는다. OS 공유 DNS 서비스나 원격 서버의 전역 종료는 지원하지 않는다.

새 package의 worker feature에만 이미 locked reqwest=0.12.28/default-features=false/rustls-tls와 필요한 Tokio 기능을 제안한다. 부모 protocol library 소비가 HTTP runtime을 자동 활성화하지 않게 한다. 현재 lock에 crate가 있다는 사실은 새 package 등록/feature graph/lock 무변경 통과가 아니다. PM이 offline metadata/lock 및 windows feature 정합을 별도 배정해야 하며 lock 손편집·dependency update·다운로드 fallback은 없다.

## D. 원 browser issuer를 마지막 생산 연결의 선행 gate로 인수

실제 후보를 읽어 확인한 현재 포트는 `BrowserProviderPort::admit_media_stream/check_media_stream`과 `BrowserWork`의 대응 메서드다. `OwnedMediaStream`은 ledger의 MediaStreamAdmission·ticket·document_generation·native_navigation_id를 보유하고 current 검사는 원 network session/record에 도달한다. `BrowserOriginalRequest::admit`도 이 포트를 소비한다. 이들은 원 metadata lease이며 **새 GET replay 허가를 발급하는 typed 포트는 현재 없다.** default Unsupported인 provider 구현도 성공 issuer가 아니다.

별도 설계 후보는 `issue_replay`/`check_replay` 성격의 원 issuer 포트와 non-Clone/non-Serialize 일회 ReplayAdmission이다(이름은 미확정). 발급에는 원 MediaStreamAdmission/current/세션·generation·원 response identity와 provider별 명시 replay policy가 필요하다. GET/200/Referer/Cookie 부재에서 public replay를 추정하지 않는다. request_url JSON을 재입력하거나 child가 복제한 token으로 재발급하지 않는다. replay 발급/소비도 원 시도의 spent 상태에 기록하고 실패·취소 뒤 재발급하지 않는다. auth/쿠키/원 response 필요 경로는 명시 미지원으로 둔다. 무조건 true/default permit 구현은 없다.

exact 기존 후보 경로는 private overlay의 `src/application/browser_provider/{network_contract.rs,mod.rs,state.rs}`, `src/application/browser_network/{request_ledger.rs,capture_state.rs,mod.rs}`, private `src/application/download/provider_session.rs`다. native forwarding이 필요하면 `src/infrastructure/browser_provider/mod.rs`를 별도 배정받는다. 이 기존 metadata7/원 candidate는 root 채택된 것으로 가정하지 않고 각 parent/preimage를 다시 동결한다. 발급 책임과 ledger spent/current 검사는 Browser 담당, parent consuming adapter는 Media/SDK 담당이다. pure protocol crate에 browser policy를 넣지 않는다.

신규 정확 시험 경로 제안은 `src/application/browser_provider/replay_tests.rs`, `src/application/browser_network/replay_tests.rs`이며 각 기존 mod의 test mount만 별도 exact delta다. stale ticket/generation/session·이미 spent·permission 부재·원 response 재사용 전용·auth 상태 불명·reload/expiry·redirect/retry 거절과 성공1회/current 유지·실패 후 재발급0을 시험한다. 실제 native owner forwarding은 별도 `tests/browser_replay_admission.rs`에서 원 provider fixture로 검증하되 synthetic를 실제 사이트 replay 허용/전송 인수로 확대하지 않는다.

## 최종 실행·지원 경계

A→B→C의 합성 fixture gate는 개발 계약 검증이다. D의 독립 replay 권한이 없으면 생산 source host를 등록하지 않는다. D 전에도 A/B/C의 pure/synthetic 작성은 별도 source 배정으로 병행할 수 있으나 코드 합성·실행은 개별 중앙 배정을 따른다. 이후 고정 worker/실제 source·original fd/probe·SDK 제품 연결13·정상 CLI의 성공/오류/취소/저장·원 물리 정리까지 인수해야 지원을 표시한다. HLS/Range/resume/재리졸브/재시작복구/modes/batch와 browser fullstream은 별도 미완료다. UI 클릭으로 SDK/CLI 공백을 대체하지 않는다.

상위 제안에서 실제 읽은 modoo.today `modootoday/software-qa/test-fixture-and-mock-build` 본문 SHA256 `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`(추가 필수 참조 없음)을 계속 적용했다. 각 실제 port/응답·시간을 fixture 입력으로 명시하고 resource setup/teardown 뒤 재획득을 확인하며 pure/synthetic/actual service를 구분한다. 새 속도 측정은 not measured다. 스킬을 이유로 새 승인 절차나 비Rust 검증 도구를 도입하지 않았다.
