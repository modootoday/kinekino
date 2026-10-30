---
id: plan:kinekino/20261029113622
title: 생산 다운로드 source host의 물리 종료 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 생산 다운로드 source host의 물리 종료 제안

2026-10-23 PM source-only 조사. 현재 root89f59f5d7ea9301dae63cef72759b6775d66994c 기준. SDK typedcancel effective8과 제품 연결13의 인수 전이며, Media runtime13은 불변이다. 구현·Cargo·네트워크·모델·process·S/T 실행0. 이 문서는 source host 지원 또는 새 worker 실행 권한이 아니다.

## 기존 코드에서 확인한 차이

| 경로 | 재사용 가능한 책임 | 현재 해결하지 못하는 책임 |
|---|---|---|
| private http_source_opener-r1/source의 HttpSourceOpener | 원 BrowserOriginalRequest/attempt/deadline·일회 open·부분 실패 owner·poison 복구·first/cleanup 분리 | HttpHostOwner::transport가 부모 HttpTransport 참조를 요구한다. subprocess host를 그대로 끼울 수 없고 concrete host가 없다. Response/Handle drop은 물리 ACK가 아니다. |
| private media-download-provider-r1의 http_stream.rs | 원 request/header·redirect 없음·identity encoding·response 검증·원 control 확인 | response를 post-send 검사 이전부터 host가 보유하지 않는다. send future/drop 후 resolver 실제 종료가 확인되지 않는다. chunk>64KiB 거절은 caller chunk 분할과 다른 제한이다. |
| owned-process Supervisor/WindowsFamily | suspended spawn→원 Job assign→resume, 원 process/Job handle, stdin EOF, poll_cleanup의 child exit/family empty, receipt 실패와 physical_closed 분리, Incomplete carrier | 현재 stdout/stderr는 regular-file evidence이고 stdin은 finite input/held EOF다. bounded body IPC·credit·nonblocking cancel/read/write 계약은 없다. 이 raw stdout을 media body 저장소로 쓰면 backpressure 없이 디스크가 증가한다. |
| browser_provider/protocol_observer.rs | 원 STA/session/response binding과 제한 getResponseBody 관측 | takeResponseBodyAsStream/IO.read/IO.close가 없다. bounded JSON capture는 fullstream이 아니다. |
| browser_provider/media_observer.rs | 원 STA의 IStream 보유·4KiB prefix·원 identity/current 검사 | 전체 body의 bounded async 소비·원 request abort/물리 close ACK가 없다. IStream의 Send 위장이나 blocking STA Read 확장은 허용하지 않는다. |

locked reqwest0.12.28 dns/gai.rs는 HyperGaiResolver를 사용한다. locked hyper-util0.1.21의 dns.rs118~160은 spawn_blocking(to_socket_addrs)과 GaiFuture Drop의 JoinHandle.abort다. Tokio1.53.3 task/blocking.rs는 시작된 blocking 작업을 abort로 멈출 수 없고 runtime shutdown_timeout은 기다림을 포기할 수 있음을 설명한다. 따라서 Handle/Response drop·카운터0·timeout 반환만으로 DNS 물리 종료를 증명할 수 없다. 이는 해당 pinned 소스의 수명 한계이며 이번 실제 DNS hang 관측을 주장하지 않는다. 다른 registry 버전은 근거로 채택하지 않았다.

## 선택과 선행 권한

**명시적으로 public 새 GET replay가 허용된 request에 한해**, 별도 Rust source child를 기존 Supervisor<WindowsFamily>가 소유하는 방식을 우선 제안한다. child 안에 reqwest client/runtime/resolver/response를 격리하고 부모는 original browser guard·attempt·출력/CAS 정책을 유지한다. 강제 종료가 필요하면 원 Job만 종료하고 원 handle로 child exit/family empty를 확인한다. 이는 child 내부의 DNS thread/socket/후손 소멸 범위다. 공유 Windows DNS 서비스·원격 서버 작업·OS cache 전체 종료를 뜻하지 않는다.

GET을 다시 해도 되는 권한은 URL·200·Cookie 부재·현재 public처럼 보임에서 추정하지 않는다. parent의 원 browser/request issuer가 replay 가능성을 명시해야 한다. opaque capability는 original Arc/attempt에 묶고 직렬화된 URL을 새 admission으로 재발급하지 않는다. child 전달값은 그 한 시도에 대한 내부 명령이며 외부 호출 권한이 아니다. URL/header/token은 argv·stdout·일반 로그에 넣지 않고 private inherited IPC로 전달한다. 실패한 첫 open도 spent이며 redirect/retry/재리졸브/다른 객체 GET은 새 권한 없이 하지 않는다.

원 response의 auth/session 재사용이 필수라면 이 child replay 경로는 Unsupported다. 그 경우 browser fullstream adapter가 별도 선행이다. 현재 IStream prefix/getResponseBody를 그대로 확장하면 shared browser runtime/DNS 물리 종료를 독립 request의 ACK로 입증하지 못한다. 원 STA에서 bounded pull을 실행하고 request/session abort와 callbacks/stream 소멸을 관측하는 계약이 먼저 필요하다. 기존 UI/browser process를 임의로 종료해 ACK를 만들지 않는다. CDP interception/stream API의 실제 의미와 frame/session binding 검증 없이는 이 경로를 승인·지원으로 표시하지 않는다.

## child 방식의 최소 계약

1. **prepare/launch 소유**: DownloadPreparation은 launch-free로 opener와 원 Supervisor pending carrier를 보유할 자리를 먼저 만든다. core가 opener를 소유한 뒤 첫 open에서만 child를 만든다. spawn 부분 실패도 기존 LaunchFailure.owner를 보유하며 새 PID owner를 만들지 않는다. worker pin/bytes/ancestor guard는 원 close까지 유지한다. HttpHostOwner의 부모 HttpTransport 반환에 dummy를 제공하지 않고 별도 process-backed SourceOpener를 구현한다.
2. **단일 원 시도**: request/attempt identity, input/object evidence, generation, 원 cutoff와 header policy를 부모가 검사한다. 부모는 매 read/credit 전후 original.check_current와 원 ExecutionControl을 검사하고 무효화 시 abort한다. child는 임의 URL 명령 server가 아니며 한 admission/한 response만 처리한다. absolute cutoff 전송의 clock 표현과 launch 시간 소비는 protocol로 고정하고 상대 timeout을 새로 시작하지 않는다.
3. **bounded IPC**: 기존 whitelist spawn에 목적 한정 inherited duplex pipe를 추가하는 계약이 필요하다. parent/child 원 endpoint를 owner가 보유하고 overlapped/nonblocking IO·bounded message parse·deadline/cancel을 사용한다. detached reader thread나 blocking Drop으로 정리하지 않는다. stdout/stderr는 제한된 진단만 남긴다. child stdout JSON status나 socket EOF 자체는 media EOF/물리 ACK가 아니다.
4. **backpressure와 큰 frame**: IPC data frame은 최대64KiB, parent queue는 기존 provider_stream의 최대256KiB/16 chunk 계약을 재사용할 수 있다. credit가 없으면 새 body poll을 하지 않고 control/abort만 계속 처리한다. reqwest가 반환한 한 frame은 Bytes+offset으로 보유해 복제 없이 caller/IPC 한도에 맞춰 분할한다. 64KiB 초과 frame을 정상적으로 처리하되 별도 upstream-frame 상한이 필요하다. 최초 제안 상한1MiB는 미검증 계약 후보이며, 초과는 ResourceExhausted와 원 close로 닫는다. 이것은 library가 이미 할당한 frame·TLS/OS buffer까지 포함한 hard RAM 상한이 아니다. 무제한 frame 수용이나 전체 body buffering을 약속하지 않는다.
5. **응답 최초 custody**: child가 send future/Response를 post-send 검사 전에 원 state에 보유한다. status/url/encoding/range/content-type/length 오류·취소·panic에서도 같은 state가 cleanup까지 소유한다. metadata→순서 있는 byte offset→명시 body-end를 구별하고 length/maximum_bytes·trusted digest 부재를 원 transfer 정책과 대조한다. EOF는 완전한 admitted body의 종료일 때만 만들며 IPC disconnect/worker crash는 실패다.
6. **원 close/ACK**: opener.poll_close가 먼저 원 abort/credit stop/IPC EOF를 요청하고 Supervisor.poll_cleanup을 같은 owner로 관측한다. 정상 body-end 뒤에도 client/runtime 종료와 child/family 종료를 기다린다. DNS blocking 때문에 graceful close가 멈추면 기존 cleanup 예산 안에서 원 Job 강제 종료를 요청한다. operation cutoff 안에 기존 cleanup reserve를 두며 예산을 늘리지 않는다. deadline 초과도 종료로 간주하지 않고 Pending/Incomplete를 유지한다. raw EOF·IPC endpoint 정리·physical_closed·최종 receipt 저장을 별개 사실로 기록하고 필요한 모두가 관측된 뒤 host ACK를 반환한다.
7. **core/오류 우선순위**: 기존 owned_session의 source.poll_close→source.close_owned→partial.abort 순서를 보존한다. client Drop/unwind도 retaining core가 원 host/response/partial/guard를 회수한다. 첫 request/control/transport 오류 또는 원 panic이 primary이고 abort/IPC/강제 종료/receipt 오류는 secondary다. close 오류가 primary를 성공으로 바꾸지 않으며 physical_closed와 작업 성공을 혼동하지 않는다. CAS·publication·검증 정책은 공통 core에 유지한다.

## 정확 책임 경로 제안 — 아직 편집 배정 아님

기존 수정 후보는 root `Cargo.toml`, `src/application/download/mod.rs`, `src/infrastructure/download/mod.rs`, owned-process의 `src/infrastructure/mod.rs`, `src/infrastructure/windows/{mod.rs,launch.rs,handles.rs}`다. owned-process 담당이 purpose-specific IPC endpoint/명시 handle whitelist·partial-launch/수명 확장을 소유하고 Media가 parent SourceOpener/response adapter를 소유해야 한다. generic arbitrary handle 상속·별도 CreateProcess launcher는 만들지 않는다. 기존 Supervisor/cleanup/ProcessFamily 정책은 바꾸지 않는 것을 우선한다. 필요 변경 발견 시 해당 exact 경로를 추가 요청한다.

신규 최소 책임 후보:

- `packages/media-source-host/{Cargo.toml,src/lib.rs,src/domain.rs,src/main.rs,src/application.rs,src/infrastructure/http.rs}`: parent와 child가 공유하는 bounded wire 타입/codec, 얇은 조립, 한 response loop, child HTTP/runtime. worker-only feature로 reqwest/runtime 구현을 분리하고 실제 locked feature/dep graph를 먼저 검토한다.
- `src/infrastructure/download/source_process.rs`, `source_process/{admission.rs,stream.rs,closeout.rs}`: 부모 original capability/attempt·원 process owner·bounded stream·poll_close 연결. worker 권한과 제품 정책을 복제하지 않는다.
- `packages/owned-process/src/infrastructure/ipc.rs`, `windows/source_pipe.rs`: purpose-specific owned endpoint 계약과 Win32 pipe/overlapped 수명. 기존 whitelist와 Job spawn을 재사용한다.

private provider_session.rs의 BrowserOriginalRequest는 replay 권한을 현재 표현하지 않는다. 이 원 candidate의 별도 revision에서 typed replay admission 계약이 필요하며 source owner와 PM이 정확 배정해야 한다. 기존 HTTP opener77DA와 http_stream9CA5는 원문 보존하고 새 process adapter로 몰래 바꾸지 않는다. browser metadata7/등록과 SDK 제품 연결13은 선행 별도 인수다. 위 목록은 구현 초안의 책임 경계이며 전부 자동 배정됐다는 뜻이 아니다. 테스트 leaf는 protocol/IPC/parent source fixture 책임별로 추가 exact 목록을 제안한 뒤 작성한다.

## 이후 인수에 필요한 fixture와 실제 근거

현재 작성/컴파일/등록/실측0이다. future fixture는 각 source attempt와 포트를 OS 임의 배정으로 고립하고 teardown 뒤 재bind가 성공하는지 확인한다. 성공200, header 직후 실패, send 중 취소, 64KiB+1/분할 frame·상한+1, credit0, EOF/length mismatch, first error 뒤 close 실패, Drop/unwind, spawn 부분 실패·child crash, resolver 지연/종료 지연·강제 종료, receipt 실패를 원 owner로 검증한다. 원 source 무효화가 실제 child 취소와 같은 증거가 아님을 구별한다. synthetic DNS 지연 fixture는 실제 OS DNS 종료를 입증하지 못한다. 별도 배정된 실제 고정 provider/네트워크에서 child 종료·family/EOF/ACK·SDK/CLI 정상/오류/취소/저장까지 인수해야 지원을 표시한다. SDK13 전에는 SourceOpener 또는 이 문서를 제품 지원으로 표시하지 않는다.

modoo.today 검색의 replay/model benchmark 결과는 무관하여 채택하지 않았다. 실제 읽고 적용한 `modootoday/software-qa/test-fixture-and-mock-build` SKILL.md SHA256 `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`은 별도 필수 참조가 없었다. 정의된 원 port/응답/시간 예산에서 fixture를 구성하고 teardown/재획득과 synthetic가 입증하지 못하는 실제 동작을 분리하는 규칙을 적용했다. 성능은 not measured이며 시간/비율 개선을 예측하지 않는다. 기존 QA 관측/지원 분리 원칙과 Rust·소유권·중앙 자원 보류가 우선이다.
