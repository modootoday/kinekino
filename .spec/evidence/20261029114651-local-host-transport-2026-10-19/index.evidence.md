---
id: evidence:kinekino/20261029114651
title: Windows 로컬 host 전송
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [application]
---
# Windows 로컬 host 전송

## 범위와 공통 경계

`LocalHost::start(ApplicationClientHandle, TransportOptions)`는 명시적으로 주입한 client를 가진 owned server다. `EndpointClient`는 같은 ApplicationClient 계약으로 Request/Response를 전달한다. CLI `--endpoint ENDPOINT`만 해당 transport를 선택하고, 옵션이 없으면 기존 SubtitleFiles/조건부 FFmpeg의 in-process SDK를 사용한다. presentation/main은 이 작업에서 수정하지 않았다. 기존 GUI status 및 root/UI의 native control 조립은 유지하며 실제 GUI+mpv 원격 적용은 별도 통합 gate다.

domain `LocalEndpoint`는 `pipe:<stable_product_id>.v1.<PID>.<process-start-FILETIME-16hex>.<CSPRNG-128bit-32hex>`만 허용한다. prefix는 `PRODUCT.stable_product_id`에서 온다. URL, UNC/원격 host, 임의 파일/프로그램, 슬래시, 기타 protocol/version, 부정확한 숫자는 허용하지 않는다. OS 경로는 검증된 식별자에서 내부 생성한다. 표시 제품명·가제 실행파일 이름은 protocol 식별자가 아니다.

## 보안과 소유권

- Windows BCryptGenRandom nonce, 현재 process token의 user SID 하나만 허용하는 protected DACL, non-inheritable handle, PIPE_REJECT_REMOTE_CLIENTS를 사용한다. 같은 계정의 다른 로컬 세션은 별도로 차단하지 않는다. service/방화벽/보안 설정을 변경하지 않았다.
- 첫 인스턴스는 FILE_FLAG_FIRST_PIPE_INSTANCE이며 최대 OS 인스턴스는 2다. 각 연결마다 새 비동기 pipe 객체를 만들되, 이전 이름 소유 handle을 닫기 전에 다음 인스턴스를 만든다. 따라서 정상 운영 중 이름 소유권 공백을 만들지 않는다. 동시에 처리하는 연결/SDK 실행은 각각 하나다. same-user 프로세스에 대한 denial-of-service나 코드 서명 attestation을 보장하지 않는다.
- client는 OS의 GetNamedPipeServerProcessId, OpenProcess/GetProcessTimes, server/current token SID 비교를 **request 바이트 전송 전** 수행한다. endpoint가 바인딩한 PID/start/user와 다르면 permission_denied이며 요청을 보내지 않는다. 신뢰할 수 있는 host에서 endpoint를 받아야 한다. endpoint는 credential이나 권한 상승 토큰이 아니다. 같은 사용자 권한 자체가 경계이며 관리자/같은 사용자 계정 전체 침해를 격리하는 기능은 아니다.
- client의 Tokio 기본 SECURITY_IDENTIFICATION QoS를 유지한다. 외부 문자열로 프로그램·셸·GUI 자동화나 임의 파일 연결을 실행하지 않는다. endpoint/raw OS 오류/원본 JSON·토큰·미디어 URL을 transport 오류에 반사하지 않는다.

실제 생성된 pipe handle에서 DACL을 읽어 `D:P(A;;FA;;;<current SID>)` 하나만 존재함을 시험했다. 처음 설정의 GA가 OS에서 file rights FA로 정규화된다. 두 번째 first-instance 생성 실패와 다른 PID/재사용된 PID의 다른 시작 시각 거부도 실제 OS pipe로 시험한다. 다른 Windows 계정 로그인/원격 SMB 접속을 직접 수행한 시험은 없으며 원격 거부는 생성 flag와 endpoint 문법 경계다.

## 프레이밍·상한·정리

한 연결은 request 1개 → response 1개 → 수신 ACK byte `0x06`이다. request/response는 각각 4-byte unsigned little-endian length와 UTF-8 JSON이며 0 length는 거부한다. request 최대 1,048,576 bytes(기존 CLI input 한도와 공통 상수), response 최대 4,194,304 bytes다. 초과 선언은 할당 전 거부하며 serialization도 bounded writer로 제한한다. 추가 frame을 실행하지 않고 연결을 닫는다. JSON 파싱 깊이 제한과 strict Request/Response 필드·API version/request_id/성공·오류 일관성을 검사한다.

응답 수신 ACK는 mpv의 적용 ACK와 관계없다. 서버는 byte 수신 확인 전 연결을 정리해 읽지 않은 응답을 버리지 않도록 ACK 또는 deadline/단절을 기다린다. 불완전·잘못된 frame, read/write 정체는 deadline 안에서 종료한다. client ACK는 결과를 이미 읽은 후 best effort이며 ACK 쓰기 실패 때문에 읽은 SDK 결과를 불확실한 결과로 바꾸지 않는다.

전송 deadline은 기본 5초, Rust TransportOptions에서 10ms..30초로 제한한다. host는 accept 대기에 무기한 있을 수 있지만 연결의 request read/SDK 대기/response write/ACK 전체에 deadline을 적용한다. client는 연결 준비/검증/write/read 전체에 deadline을 적용한다. CLI --timeout-ms는 기존 SDK operation context이며 전송 deadline을 늘리지 않는다. 긴 자막/프레임 작업은 5초 이후에도 계속될 수 있다.

Drop은 stop flag와 cancellation을 보내고 owned pipe의 overlapped connect/read/write를 CancelIoEx/DisconnectNamedPipe로 취소한 뒤 I/O thread를 join한다. 다른 앱/프로세스는 종료하지 않는다. 이미 시작한 synchronous SDK execute는 강제 취소할 수 없고 bounded executor 하나에서 완료될 수 있다. worker는 host 참조를 보유하지 않으며 자기 작업 종료 후 채널이 닫히면 끝난다. busy 동안 두 번째 작업을 대기열에 넣거나 나중에 실행하지 않는다. `LocalHost.is_running()`은 host I/O thread 상태만 보고한다.

## 결과 의미와 호환

host는 주입된 SDK의 capability/schema/data/오류/unsupported를 재작성하지 않는다. SDK가 미지원으로 보고하는 `local_host_transport` discovery 필드도 수정하지 않는다. 이는 기존 SDK metadata이며 명시 endpoint 연결 성공 여부를 뜻하는 자동 registry/discovery 필드가 아니다. registry/singleton discovery·endpoint 파일 게시·watch/jobs/revision/idempotency는 이 작업에 없다.

전송 이전 실패는 `details.delivery=not_sent`, host executor busy는 `not_dispatched`, 쓰기 시작 후 timeout/단절/잘못된 응답은 `execution_unknown`이다. 오류는 retryable=false와 automatic_retry=false다. 응답을 받지 못했어도 작업이 실행되었을 수 있으며 fallback이나 재전송을 하지 않는다. 연결 전 ERROR_PIPE_BUSY 대기는 요청을 한 번도 보내지 않은 상태에서만 제한된 deadline 동안 한다. 네 제어 명령의 accepted/unobserved receipt는 그대로 유지되고 실제 mpv 적용·성공 완료로 승격되지 않는다.

잘못된 endpoint는 기존 reserved unsupported 대신 invalid_argument/exit2로 바뀌었다. 검증된 endpoint의 미실행/종료 host는 not_found/exit4다. 유효한 request_id는 공통 Response::failure로 보존한다. 기존 in-process CLI 자막/프레임·인수·기본 schema 동작은 계속 기존 시험으로 검증한다. 과거 동결 산출물/근거를 갱신하지 않는다.

## 검증

Rust `tests/local_host.rs`는 owned host + 실제 player-cli subprocess로 SDK discovery, 전체52 schema, unsupported, Unicode correlation, 허용한 typed control receipt를 비교한다. control fixture는 명시 주입한 test port이며 실제 mpv 재생이 아니다. malformed/0/초과/truncated framing, CLI input 상한, response 상한·느린 reader, 단절/timeout/Drop, busy 무재실행, 기본 SDK 미제공 adapter를 시험한다. security unit은 실제 pipe ACL/최초 독점/server identity를 검사하고 application unit은 bounded codec과 response envelope를 검사한다.

처음 반복 연결 시험에서 이전 비동기 객체의 I/O 상태가 다음 연결에 남는 문제를 발견해 fresh-instance rollover로 수정했고, 재시험에서 discovery/52 schema/4 receipt와 malformed recovery가 통과했다. 실제 DACL 문자열의 GA→FA 정규화와 response nullable 필드 누락 검사도 실패를 관측한 뒤 수정·통과했다.

최종 검사(전용 `runtime/parallel-build/sdk`, -j1, locked/offline):

- `cargo fmt --check`: exit0.
- `cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 1 --quiet`: exit0, **116 passed / 0 failed / 8 ignored**. lib80, CLI unit2, cli_contract4, cli_request_identity1, frame_capture2, local_host9, playback_control4, playback_status3, sdk_capability_parity2, subtitle_conversion9 passed. 기존 외부 미디어 ignored 시험은 실행하지 않았다.
- `cargo clippy --locked --offline --all-targets --target-dir runtime/parallel-build/sdk -j 1 -- -D warnings`: exit0.
- SDK 단일 executor를 별도 책임 모듈로 분리한 최신 소스에 `cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 1 --lib --test local_host --quiet` 재확인: lib80 passed/1 ignored, IPC9 passed, exit0. 별도로 합산해 전체 test 수를 늘리지 않는다.

동일 작업 중 UI 담당의 기존 테스트 추가가 공유 workspace에 포함될 수 있다. 위 수치는 실행 당시 전체 집합이며 새 IPC 전용 시험은 unit3 + integration9다. 실제 GUI/remote mpv 적용, 다른 계정/원격 Windows 연결, 장시간 시험·AI·Pages·VM·DPI는 이 관측에 포함하지 않는다.

## 근거 문서

- [Microsoft named pipe security](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights)
- [Microsoft GetNamedPipeServerProcessId](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getnamedpipeserverprocessid)
- [Microsoft DisconnectNamedPipe와 unread data](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-disconnectnamedpipe)
- [Tokio ServerOptions의 first-instance/remote/security attributes](https://docs.rs/tokio/1.53.3/tokio/net/windows/named_pipe/struct.ServerOptions.html)
- 구현 handoff: `runtime/coordination/sdk.md` 최신 local host 절. [계획006](../../../plans/player/plan-006-sdk-gui-cli.md).
