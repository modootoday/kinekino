---
id: evidence:kinekino/20261029114237
title: IPC integrity 및 elevation 경계 후속 검증
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [application]
---
# IPC integrity 및 elevation 경계 후속 검증

## 문제와 기존 근거의 한계

[이전 transport 검증](local-host-transport-2026-10-19.md)은 당시 현재-user SID/DACL/PID/start 경계를 기록한다. 사용자가 관리자 권한으로 실행한 Desktop과 그 자식은 elevated token을 상속할 수 있다. 동일 SID는 동일 integrity/elevation을 뜻하지 않는다. 기존 DACL만으로 낮은 권한 호출이 높은 권한 host의 SDK를 실행하지 못한다고 주장해서는 안 된다.

## 명시적 허용 정책

domain `local_peer::PeerSecurity/permits_peer`는 **user SID + integrity RID + TokenElevation(bool) + TokenElevationType(Default/Full/Limited)**가 모두 같을 때만 허용한다. 비어 있거나 알려지지 않은 값, 일관되지 않은 elevation, 조회 실패는 거부한다. High↔Medium/Low, elevated↔non-elevated, Full↔Limited/Default 중 하나라도 다르면 양방향 거부다. 요청 JSON·endpoint 문자열에서 이 값을 받지 않는다. 새 error code·schema·임의 권한 획득 경로를 추가하지 않는다.

infrastructure `peer_token`은 Windows process/thread token에서 TokenUser, TokenIntegrityLevel, TokenElevation, TokenElevationType를 읽는다. 필요한 TOKEN_QUERY만 사용한다. API 실패나 예상치 못한 buffer length는 permission_denied로 닫으며 raw SID/token/OS 오류를 공개하지 않는다.

- client는 pipe의 실제 server PID/start/user 검증에 더해 server primary token과 현재 client process token의 privilege profile을 비교한 후 request를 쓴다.
- server는 connect 직후 GetNamedPipeClientProcessId와 해당 process primary token을 조회하고 host와 비교한다. 실패하면 request를 읽지 않고 연결을 정리한다.
- primary token 검사만으로는 같은 high process의 낮은 thread impersonation context를 구분할 수 없다. 따라서 bounded request를 읽은 뒤 **SDK parse/dispatch 전에** ImpersonateNamedPipeClient와 OpenThreadToken(OpenAsSelf=true)로 pipe 메시지의 실제 보안 context를 조회한다. 이 profile도 connect 당시 peer primary profile과 정확히 같아야 한다.
- message 검사는 동기 query만 하며 impersonating 상태에서 await/SDK 실행/파일 작업을 하지 않는다. 즉시 RevertToSelf 후 판단하고, 복귀 실패는 해당 owned host I/O loop를 종료한다. guard는 조기 반환·unwind에서도 복귀를 시도한다. SDK executor thread에는 impersonation token을 전달하지 않는다.
- LocalHost.start와 EndpointClient.execute는 호출 thread에 impersonation token이 있으면 거부한다. 새 OS thread를 만들면서 낮거나 제한된 호출 context를 잃고 process primary 권한으로 실행되는 우회를 막는다. 조회 실패도 거부한다. 정상 GUI/CLI 호출은 unimpersonated thread에서 사용한다.

위 정책은 같은 profile의 프로세스가 가진 모든 세부 privileges/group restriction이 동일하다고 증명하는 token attestation이 아니다. 다른 사용자/remote 차단, first-instance/name continuity, PID 재사용 방지, 고정 endpoint, bounded framing, 자동 retry 금지는 기존 경계와 함께 유지한다. 권한이 다른 host에 접속하기 위해 자동 elevation·linked token 선택·실패 후 우회하지 않는다.

## 실제 Windows 관측

2026-10-19 Rust unit의 실제 process token 조회 결과는 **integrity RID 12288(High), elevated=true, elevation_type=2(Full)**였다. 이는 이번 시험 프로세스에서 직접 읽은 값이며 모든 사용자 환경의 기본값을 뜻하지 않는다.

`current_real_token_is_observed_and_impersonating_callers_cannot_gain_a_fresh_thread`는 실제 토큰을 복제해 동일 권한의 thread impersonation을 만든 뒤 LocalHost/EndpointClient 호출이 거부되는지 확인하고, RevertToSelf 이후 원래 context가 유지되는지 확인했다.

`real_lower_integrity_pipe_context_is_denied_without_changing_process_or_account_privileges`는 **복제한 시험용 token만** Medium RID8192로 낮추고 별도 시험 thread에서 pipe를 실제로 열어 byte를 전송했다. 원본 process/account token, 계정 권한, 전역 보안 설정, 사용자의 앱은 변경하지 않았다.

이 fixture는 TokenIntegrityLevel만 설정하며 UAC로 필터링된 Medium/Limited primary token을 생성한 시험이 아니다. elevation bool/type의 불일치 거부는 명시적 domain matrix로 검사하고, 실제 pipe 관측은 integrity/message-context 차이의 거부로 한정한다.

관측 결과: `host=12288 message=8192 denial=message_token`. 즉 **OS connect/write는 허용됐고** peer PID의 primary token은 High였지만, pipe의 실제 message context가 Medium인 것을 추가 검사로 발견해 거부했다. 이 결과는 DACL/SID 또는 primary token 비교만으로는 부족한 경계를 실제로 보여 준다. fixture는 SDK를 호출하지 않는 보안 검사용 owned pipe이며 실제 GUI/mpv 적용 시험이 아니다. 마지막에는 thread impersonation이 사라지고 원본 process profile이 그대로임을 확인했다.

기존 실제 CLI subprocess의 정상 host 9개 transport 시험은 상속된 같은 High/Full profile에서 동작해 client의 server token 조회와 server의 다른 client process token/message 조회가 실제로 성공하는 허용 경로를 검사한다. domain matrix는 Medium/High, IL만 다른 경우, elevated flag만 다른 경우, elevation type만 다른 경우, 다른 SID와 알 수 없는 상태의 양방향 거부를 시험한다.

## 미검증 범위

별도 Medium/Limited primary token 프로세스와 elevated GUI 사이의 end-to-end 연결, 다른 계정/원격 SMB, 전체 UAC 설정 조합, 같은 profile 안의 모든 restricted/linked privilege 조합, RevertToSelf OS 실패의 강제 주입은 수행하지 않았다. 실제 lower **message context** 시험을 lower-primary-process 전체 통합 시험으로 확대해 주장하지 않는다. 이 경우에도 token query/equality 실패는 허용하지 않는 정책이다. 전역 설정·권한을 바꾸거나 사용자 앱을 재실행해 시험하지 않았다.

## 검사 결과

실제 token/context focused 시험 2개 통과(`--lib local_host::peer_token -- --nocapture --test-threads=1`). 이후 unreadable process/message handle을 permission_denied로 닫고 thread/process context가 유지되는지 확인하는 시험 1개도 추가했다. domain에는 알려지지 않은 IL 및 privilege mismatch 거부 시험 1개를 추가했다. 강화 후 기존 IPC integration 9개도 모두 통과했다.

최신 코드 최종 검사(전용 sdk target, -j1, locked/offline):

- `cargo fmt --check`: exit0.
- `cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 1 --quiet`: exit0, **120 passed / 0 failed / 8 ignored**. lib84, CLI unit2, cli_contract4, cli_request_identity1, frame_capture2, local_host9, playback_control4, playback_status3, sdk_capability_parity2, subtitle_conversion9 passed.
- `cargo clippy --locked --offline --all-targets --target-dir runtime/parallel-build/sdk -j 1 -- -D warnings`: exit0.

추가 보안 시험은 domain1 + token3이다. 이전 116/119 체크포인트와 focused 재실행을 중복 합산하지 않았다. 기존 외부 미디어 ignored 시험·GUI/mpv gate는 실행하지 않았고 이전 동결 transport 검증 수치는 당시 사실로 유지한다.

## Windows 근거

- [TOKEN_INFORMATION_CLASS: integrity/elevation 조회](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ne-winnt-token_information_class)
- [TOKEN_ELEVATION_TYPE의 세 종류](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ne-winnt-token_elevation_type)
- [GetNamedPipeClientProcessId](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getnamedpipeclientprocessid)
- [ImpersonateNamedPipeClient의 마지막 message 보안 context](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-impersonatenamedpipeclient)
- [Mandatory Integrity Control과 DACL의 별도 경계](https://learn.microsoft.com/en-us/windows/win32/secauthz/mandatory-integrity-control)
