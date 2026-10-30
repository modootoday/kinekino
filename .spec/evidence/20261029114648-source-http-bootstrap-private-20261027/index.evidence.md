---
id: evidence:kinekino/20261029114648
title: Private HTTP bootstrap 후보 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Private HTTP bootstrap 후보 — 2026-10-27

후속 R25/R26의 parent process/IPC/partial-launch custody와 actual OS2·순수47 검증은 [부모 host 전용 기록](source-parent-host-custody-20261027.md)에 분리했다. 선택 input68이며 R22 고정 worker를 소비했다. root SourceOpener/D replay/SDK 연결은 미완료다.

## 후속 R23/R24 — 부모 wire 수신과 worker 상호 운용

root의 실제 SourceOpener/DownloadSession/retaining core와 private http-source-opener 원 후보를 읽었다. 기존 source.poll_close→source.close_owned→partial 정리 순서와 SDK의 오류/취소/CAS를 그대로 사용해야 하며 부모 HttpTransport 참조를 dummy로 채우지 않는다. D의 원 browser replay 권한이 없으므로 production factory/SourceOpener 등록은 계속 미지원이다. 이번에는 그 어댑터가 소비할 ParentStream wire receiver만 private 구현했다. root SDK 코드는 바꾸지 않았다.

ParentStream은 원 binding/Exchange와 URL/maximum을 보유하고 Metadata를 재검사한다. pending 최대256KiB/16chunk와 outstanding byte credit를 합산해 제한한다. 슬롯이 모두 차면 partial chunk를 조금 읽었다는 이유만으로 credit를 추가 발급하지 않는다. physical data read를 멈추는 can_receive와 독립 control/abort 경계를 제공한다. 부모 read는 Pending/Data(n)/End를 구별하며 명시 BodyEnd와 길이 일치·queue drain 전에는 EOF를 반환하지 않는다. transport EOF/worker Failure/잘못된 URL·길이를 sticky 실패로 기록한다. 이 객체 생성/metadata는 원 replay/current·media identity·trusted digest·physical ACK를 발급하지 않는다.

preimage Cargo/lib 및 상호 운용 시험 이전 worker tests를 `runtime/verification/source-parent-stream-preimage-r1`에 보존했다. R23(input inventory73E79B91)은956 strict·957 순수46 통과. 선택 R24 SOURCE.md SHA256 `D134C67B860375F98D92E98813322638C118A36BEFE05DB857219D3BC7190C74`, inventory `D7140E82DDAD0A985D19724DFDA951554AF61E8F3000C04662258238B798EE8E`,65files,lock BC3E5C43 유지.958 all-target/all-feature strict·959 메모리/순수47(기존43+parent3+상호운용1,OS3ignored)·961 fmt·962 content/identity 동일65 통과했다.960 normal parent-stream dependency tree는 url graph만 포함하며 reqwest/tokio/owned-process가 없다. 새 첫 실패 없음.

parent3은 Pending과 End 구별/소비 후 credit 공간·16chunk/256KiB 합산 상한·partial chunk 슬롯 유지·원 URL/최종 길이/worker Failure/미BodyEnd pipe EOF 거절을 확인했다. 상호운용1은 같은 parent Open/Metadata/Credit와 worker Response70000byte를 실제 codec/1111byte partial writer로 전달해 Data2개/연속 offset/원문 전체 일치/명시 BodyEnd/parent queue drain End를 확인했다. 실제 네트워크나 IPC 전송은 아니다. R22 OS3은 그 원 입력의 성공으로 보존하며 새 R24 OS 실행으로 승계하지 않는다.

다음 구현은 원 SourceFamily/partial-launch owner·first/secondary/원 diagnostic receipt를 보유하는 parent host와 launch-free/one-use opener, D의 원 replay issuer 및 SDK 연결이다. 기존 B/worker/root WIP와 소유권·AI/GUI·추론/다운로드/수동GUI/WARP/push 보류를 유지한다.

## 후속 R22 — 완성 bootstrap/미Open 실제 OS와 원인 진단

entry의 로컬 IO 정리 뒤 제한 진단에 first enum·secondary 개수·overflow를 출력한다. LoopError 이하가 모두 유한 enum이며 reqwest error/URL/header/원 panic 객체/미디어는 포함하지 않는다. stdout은 외부 관측일 뿐 부모 physical ACK가 아니다. 진단 IO 실패는 별도 실패26이며 성공으로 바꾸지 않는다. entry/tests preimage는 `runtime/verification/source-worker-diagnostic-preimage-r1`에 보존했다.

선택 input-r22 SOURCE.md SHA256 `7F5ABA1681BA08DC864CE79DB7EBC148976892C80F4DFFCA4BD65B1ED5CBECF6`, inventory `E93A2EB1B2E768D68412CA056DFF2D0CFC41E0F19F9B7025319BA8411CCE8D34`,62files,lock BC3E5C43 동일.949 strict·950 OS test/normal producer no-run·954 fmt·955 content/identity 동일62 통과. 이전 순수43은 R21 실행에 유지하며 이번에 재실행한 것으로 계상하지 않는다. OS3은951~953에서 새 pin으로 각각 실제 실행했다.

고정 fixed-r22 worker SHA `2BDCA67FB13578D0328C69BC6A6C901100A467C0854F645864D2290BCBCAFCC9`/6648832B, test SHA `0D44C19A16A3B2BBA962E88D39D70EE4BD3769271ADF4B37182A67D99E6DE666`/2076160B. operator index `source-worker-preopen-r22-index.json` SHA `E39AF15D3901726EB80CA9DA2169B9565091E4E7428B488EC590D886F1CE1E02`/558B는 proof=false다. 같은 독립 baseline driver7E7AD258/45s/cleanup5s를 사용했고 pins는 실행 후에도 동일하다.

|로그/root|외부 worker 진단|원 worker/outer test|
|---|---|---|
|951 source-worker-no-open-os1-r22|first=PeerEof,secondary0,overflow=false|worker13440/exit23,outer21804/exact1pass|
|952 source-worker-eof-os1-r22|first=Ipc(Bootstrap(Truncated)),secondary0|worker8704/exit23,outer13860/exact1pass|
|953 source-worker-truncated-os1-r22|first=Ipc(Bootstrap(Truncated)),secondary0|worker33092/exit23,outer28016/exact1pass|

모든 inner/outer는 원 handle의 JobEmpty/nonforced, inner parent IPC local reaped/closed, 원 receipt 저장을 확인했다. outer test는 exit0/raw EOF/ownership complete다. 각 root에 outer 원 outcome/raw/receipt와 inner-observed copy를 두고 원 tmp도 유지했다. parent pipe control/data EOF=false는 그대로 남긴다. worker 진단은 자가보고이므로 독립 원 handle/receipt를 대체하지 않지만 R21의 특정 원인 미관측 범위를 보완한다. 세 시험 모두 Open을 보내지 않아 HTTP preparation에 도달하지 않는다. 네트워크 packet 계측·정상 HTTP/download 검증은 아니다. 전체 목표·제품 root190/bins·AI/GUI/원 소유권·WIP·보류는 유지한다.

## 후속 R20/R21 — 실제 worker pre-HTTP OS2

새 ignored integration 시험2를 작성했다. 완성 bootstrap/Open은 전혀 보내지 않고 EOF 또는3byte `MHR` 뒤 endpoint를 닫는다. parent는 원 spawn_source의 child/Job/IPC를 보유하고 원 handle로 child exit를 관측한 뒤 cleanup/receipt를 수행한다. action panic·launch/cleanup Incomplete carrier도 동일 owner로 보존한다. R20은939 strict·940 no-run 통과했으나 OS 실행 전 readiness 경합을 확인했다. endpoint를 너무 빨리 닫으면 child handle adoption 단계에서 실패할 수 있으므로 실제 loop EOF와 구별해야 했다.

entry에 고정24byte `source-worker-ipc-ready\n` 진단을 추가했다. inherited endpoint와 runtime 소유 후의 관측이며 request 허가/physical ACK가 아니다. 시험은 bounded 원 stdout record로 readiness를 기다린 뒤 입력을 보낸다. URL/header/error 본문은 출력하지 않는다. readiness 전 entry/tests preimage는 `runtime/verification/source-worker-os-preimage-r1`에 보존했다. patch 위치 불일치는 변경 없이 재읽어 적용했고 새 compile/test 실패는 없다.

선택 input-r21 SOURCE.md SHA256 `A7FFE737B66BF976EB5BB5B151900C9665EFD9ADACAE5DE743A3CC54A8035264`, inventory `4559DFFB29066C3FDBB089FBC494B21F9585C7EE91B514699F183FB71225CFAC`,62files,lock BC3E5C43 동일.941 strict·942 실제 OS test/normal binary producer no-run·945 fmt·946/948 content/identity 동일62·947 순수/메모리43/OS2ignored 통과. OS2는 별도943/944에서만 실행했다. 지정190/제품 정상bins 재실행이 아니다.

고정 `fixed-r21` worker SHA `D6682049519CDC8BB4E2416D947C79D3E3A34208157040FE498844C6974C42BF`/6639616B, OS test SHA `DA4C9CCD9469720C647CCF3E82ABD2DA4D9C46598D0339EB0B2EFA3E2466A4D3`/1100800B. 두 exact case는 기존 독립 baseline driver7E7AD258/924672B의45s/cleanup5s에서 각각 실행했다. baseline index `source-worker-preopen-r21-index.json` SHA `5603F1A3EA29BACEB80393639999042E07D699D33C91FC774D5B85D21357FED2`/580B는 proof=false operator index이며 Cargo/source/원 receipts를 대체하지 않는다. producer pins는 실행 후에도 동일하다.

|로그/근거 root|실제 결과|원 worker 사실|
|---|---|---|
|943 `source-worker-eof-os1-r21`|exact1pass,outer testPID26568/exit0/Empty/nonforced/raw EOF|workerPID36336/exit23/Empty/nonforced; readiness24B,stderr0B; parent local IPC reaped/closed|
|944 `source-worker-truncated-os1-r21`|exact1pass,outer testPID13396/exit0/Empty/nonforced/raw EOF|workerPID19664/exit23/Empty/nonforced; readiness24B,stderr0B; parent local IPC reaped/closed|

각 root의 outer outcome/receipt/raw와 inner-observed copy를 보존했고 원 tmp inner도 유지했다. parent ClosePoll control_eof/data_eof=false이며 local close를 peerEOF 관측으로 바꾸지 않는다. worker의 특정 LoopError variant를 독립 진단으로 읽은 것은 아니고 예상실패23과 원 child/Job/receipt 사실을 확인했다. HTTP 미준비는 완성 bootstrap/Open 부재의 고정 코드 경로에 근거하며 네트워크 packet 계측은 하지 않았다. 이 expected failure 시험의 통과를 정상 HTTP/download 성공으로 계상하지 않는다.

실제 정상 bootstrap/Open/control/credit/Data/BodyEnd 및 DNS/TLS/응답 실패·취소·panic/cleanup 지연·parent SourceOpener/D replay/SDK는 남는다. 기존 root 제품190/bins·AI품질0/9·GUI 미인수·원 WIP/소유권·추론/다운로드/수동GUI/WARP/push 보류 유지.

## 후속 R18/R19 — worker 실행 루프·정상 binary (실제 실행 전)

worker_loop/drive/outcome과 전용 media-source-worker entry를 작성했다. URL/header는 argv에 없고 inherited handle2·절대 uptime cutoff2만 받는다. unsafe handle 인수는 trusted spawn_source 원 handle1회 전달을 전제로 하며 CLI 입력만으로 원 browser replay 권한이 생기지 않는다. 부모 source factory/registry는 계속 미등록이다. 런타임은 current-thread Tokio이며 control event를 매 turn 먼저 처리하고, 원 send/Response를 보유한 채 HTTP future를 한 번씩 poll하며1ms sleep에서 runtime IO를 구동한다. source/Response 상태는 Pending 뒤에도 남고 detached reader를 생성하지 않는다. 이 scheduling의 실제 OS/네트워크 동작은 아직 미검증이다.

bootstrap/Open→send→Metadata/credit/Data→BodyEnd 또는 boundary Failure→같은 IPC poll_close를 연결했다. body 종료 뒤에도 부모가 실제 child/family exit·raw EOF·receipt를 확인해야 한다. operation cutoff 뒤 새 Failure 쓰기를 보장하지 않는다. 로컬 IO 정리가 end까지 끝나지 않으면 owner를 보유한 채 Incomplete를 반환하고 entry는 실패124로 process exit한다. local IO reap 성공을 Job ACK로 바꾸지 않는다. runtime Drop의 resolver 대기가 남을 수 있으며 부모 원 Job/cleanup 예산의 강제 종료가 필요할 수 있다. shutdown_timeout 반환을 종료 근거로 쓰지 않는다.

panic은 catch한 원 Box payload를 outcome ledger에 넣은 후 같은 owner close를 수행한다. 첫 오류와 별도 최대8개 중복 없는 secondary·overflow를 보존하고 panic payload/URL/error 본문을 로그로 출력하지 않는다. 새 순수1시험은 원 Order primary·원 panic 객체·8secondary/overflow·같은 IO 관측100회 중복 억제를 확인했다. 실제 worker panic/unwind/IO 정리를 검증한 것은 아니다.

preimage Cargo/lock/lib/session·쓰기 cutoff 보완 전 drive는 `runtime/verification/source-worker-loop-preimage-r1`에 있다. R18은928 offline lock·929 strict·930 시험43·931 정상binary·932 fmt·933 input61 동일 통과했다. R18 binary `fixed-r18/media-source-worker.exe` SHA `F5362CC8495CB408FD3468EA3755928AD77F4DFDCBF6DBDDFCF1DEEF7963C4DE`는 보존했다. 그 뒤 Open이 inherited cutoff를 줄일 수 있으므로 IPC write 전후에도 Session의 축소 cutoff를 재검사하도록 보완했다. patch 첫 위치 불일치는 파일 변경 없이 재읽어 적용했으며 실제 컴파일/시험 실패는 없다.

선택 input-r19 SOURCE.md SHA256 `685CA0D17E3263B07C554892A270035286AE8E91E6EC91CA1B1876821F4AEDAD`, inventory `5C5ECBE9613962128EA71543D7D83E8D53A7DCB34F0F4CD1804B7C189D1DB115`,61files,lock BC3E5C43 동일.934 strict·935 순수/메모리43(기존42+ledger1)·936 정상binary·937 fmt·938 content/identity 동일61 통과했다.936 Cargo artifact profile.test=false/fresh=true이며 같은 입력935에서 이미 생성한 normal artifact를 사용한다. 선택 고정 copy `runtime/build-cache/native/source-http-bootstrap-r1/fixed-r19/media-source-worker.exe` SHA256 `DFD749A877A281589C41E2C8E59634893B35D7E7B5728FAEA25D73E5DADBD58C`다.

새 binary를 실행하지 않았다. 실제 worker/원 inherited IPC·DNS/TLS/HTTP·timer fairness·cutoff 실측·원 child/family/receipt·부모 SourceOpener·D replay·제품SDK 인수는 남는다. 기존 B OS10이나 제품 DFB0 190/bins를 새 worker 성공으로 계상하지 않는다. AI/GUI·원 WIP/소유권·추론/다운로드/수동GUI/WARP/push 보류 유지.

## 후속 R16/R17 — 원 B inherited IPC 어댑터 조립

검증된 B 격리 문맥의 src/Cargo/lock만 이 후보 `packages/owned-process`로 복사했다. B 원 입력/고정 executable은 바꾸지 않았고927에서 원 input-r4 32files content/identity 동일을 확인했다. 복사 dependency는 원 B의 모든시험/AGENTS snapshot과 동일 분모라는 뜻이 아니다. optional worker-ipc가 Windows에서만 이 path dependency를 사용한다. media-source-host의 이전 Cargo/lock/lib 및 control 제한 전 receiver/adapter/tests preimage는 `runtime/verification/source-worker-ipc-preimage-r1`에 있다.

WorkerIpc는 원 SourceIpc::from_inherited만 인수하며 다른 launcher/handle duplication을 구현하지 않는다. unsafe 호출자는 spawn_source의 원 distinct handle2를 정확1회 전달해야 한다. bootstrap→control Frame을 bounded buffer에서 하나씩 꺼내고 coalesced remainder를 보유한다. control-only FrameReceiver는 header에서 Open/Credit/Abort만 허용해 Data 본문 할당 전에 거절한다. data write는 B가 원 OVERLAPPED buffer로 복사한 뒤 completion count를 Session에 넘긴다. read/write가 따로 poll되므로 대기 data write 중 control을 계속 처리할 수 있다. 실패 후에도 같은 B owner의 poll_close를 호출할 수 있다. EOF/ClosePoll은 원 Job/receipt ACK가 아니다.

R16은921 offline lock·922 strict 통과. control header 제한 보완 후 선택 input-r17 SOURCE.md SHA256 `3CD283F82E7C2023C64698992709BD7F19A34298AC0A625628BAC48457AC39A9`, inventory `20B1BC4C99B6C0ECE649EF25C39BEBEF4E4C5B4C0799CFCCB1037F4DAD89892E`,57files. 새 isolated lock `BC3E5C43CFD1B4198C3782957FF82B5132B92F4A627A53CD6F0A8768579A9C1E`는 원 product/B lock과 구별한다.923 all-target/all-feature strict·924 순수/메모리42(기존41+control-header1)·925 fmt·926 content/identity 동일57·927 원 B32 동일 통과. 새 first failure 없음.

새1시험은 선언64KiB Data를 control header60byte만으로 sticky Order 거절하고 Abort zero-body를 허용함을 확인했다. 새 WorkerIpc의 실제 inherited handle/OS IO·closed 상태·원 worker 프로세스는 아직 실행하지 않았다. 기존 B OS10 성공을 새 어댑터 성공으로 승계하지 않는다. B는 operation cutoff 후 새 쓰기를 금지하므로 그 뒤 Failure 전송은 보장하지 않고 poll_close로 넘긴다. worker executable/main loop/runtime scheduling·원 부모/D replay/SDK 연결은 미완료다. 원 제품190/정상bins·AI/GUI 및 기존 보류는 유지한다.

## 후속 R14/R15 — failure frame과 첫 오류 보존

WorkerSession::begin_failure는 첫 WorkerError를 보존하고 같은 send를 abort한다. 프레임 경계·원 end cutoff 이전에만 고정 짧은 UTF8 Failure를 한 번 queue한다. URL/header/reqwest error를 format하지 않는다. 이미 부분 송신 중인 frame·AwaitOpen/Abort/Ended/Failed·end cutoff 도달이면 새 Failure를 끼워 넣지 않고 false를 반환하며 원 writer/HTTP owner를 보존한다. 외부 owner는 원 pending IO cancel/reap·Job/receipt 정리를 계속 수행해야 한다. poisoned Exchange 때문에 Failure encode/accept가 실패하면 그 반환은 secondary이며 first_failure를 바꾸지 않는다. failure_written은 wire 완료만 의미하고 작업 성공/물리 ACK가 아니다.

preimage session/tests 및 첫 기능 실패의 failure.rs를 `runtime/verification/source-worker-failure-preimage-r1`에 보존했다. R14(input inventory BC07A64B)에서915 strict는 통과했지만916 새 failure boundary 시험이 실패했다. 송신 완료 뒤 offer(None) 조회에서 원 Http(Invalidated)를 다시 반환한 문제이며 나머지 library9는 통과했고 뒤 integration 시험은 미실행이다. Failure가 완전히 쓰인 경우 writable 경로의 terminal 조회를 허용해 수정했다. first_failure는 별도 보존한다.

선택 input-r15 SOURCE.md SHA256 `ABCFB4D045278907A00FC3E2A172E7801DCD1151F23506D8D934F241C1509B3A`, inventory `D28C87A24B1D8E829931F0944B55FAA3B2046426BB9FA602C475FD4C6AC16D4A`,29files,lock B7447F9D 유지.917 strict·918 순수/메모리41(기존38+failure3)·919 fmt·920 content/identity 동일29 통과.

failure3은 정상 boundary의 bounded Failure/sequence/partial write·다른 후속 오류/abort 후 first 보존, 부분 frame 중 거절·end cutoff 불연장, 원 binding 실패 뒤 poisoned Exchange의 secondary Order가 primary Binding을 대체하지 않음을 확인했다. 원본/WIP/소유권·추론/다운로드/수동GUI/WARP/push 보류 유지. 실제 Failure IPC/worker 실행·physical closeout·parent SourceOpener/D replay/SDK는 계속 미완료이며 제품190/bins·AI/GUI 인수로 확대하지 않는다.

## 후속 R11~R13 — single worker session 통합

WorkerSession이 bootstrap·원 send/Response·Exchange·대기 FrameWriter를 함께 보유한다. Open 후 bootstrap을 setup 전에 take하여 재시도를 막고, metadata→credit→Data→실제 response EOF의 BodyEnd를 원 sequence/offset/credit 규칙으로 연결한다. queue는 encoded frame 생성 후 Exchange를 진행한다. HTTP body read는 streaming/credit>0/대기 frame 없음/원 body buffer 소진일 때만 가능하며 caller current/attempt/cutoff check를 read 전후 실행한다. 대기 write 중에도 control lane Abort를 처리하고 원 send/프레임을 유지한다. failure 후에도 explicit abort로 같은 future 정리를 요청할 수 있으며 첫 오류를 보존한다. 이 abort/BodyEnd는 원 Job/IPC EOF/receipt 물리ACK가 아니다.

preimage는 `runtime/verification/source-worker-session-preimage-r1`에 lib/Response/send 및 strict/explicit-abort 수정 전 session/tests를 보존했다. R11 input에서907 strict collapsible_if 첫 실패를 보존했고908은 미실행이다. 동작 동일 let-chain과 fixture의 역행 clock(다음credit3/직전data4)을 단조4로 수정한 R12는909 strict/910 시험38 통과했다. explicit abort를 추가한 선택 input-r13 SOURCE.md SHA256 `6819E46320933222FE8AA6ECB5D72EA728AA5FA57A1A12A0EA8A42595AA707A6`, inventory `AFD656F7D59C6D3334A259A866A8D7AAC094494ACC620F05AE06FC234495E2A5`,28files다. lock B7447F9D 유지.911 strict·912 메모리/순수38(기존35+session3)·913 fmt·914 전후 content/identity 동일28 통과.

session3은 실제 reqwest 타입의 in-memory70000byte 본문을 metadata→7회credit10000→Data offset/원문 전체 일치→추가credit 뒤 EOF→BodyEnd까지1111byte partial write 완료로 연결했다. Metadata 전송 대기/credit0/잔류 body 중 새 read 금지, 대기 metadata write 중 control Abort와 owner 보존, read post-check invalidation의 sticky 실패·explicit abort 뒤 first failure 보존을 확인했다. fixture send constructor는 cfg(test)이며 production future/GET는 poll하지 않았다.

미완료: 실행 worker entry/runtime/원 inherited IPC loop·현재 시각/절대 cutoff의 outer scheduling·취소/오류 Failure frame·진단/exit/물리 closeout·부모 SourceOpener/D replay 권한/SDK registry가 남는다. send/Response 및 framing을 단일 상태로 연결한 메모리 인수이며 실제 worker 프로세스/네트워크/다운로드나 원 제품190/bins·AI/GUI 완료로 계상하지 않는다. 기존 소유권/WIP/원 실패·보류 유지.

## 후속 R9/R10 — typed HTTP Metadata v1

[별도 payload 계약](../../contracts/source-host/http-metadata-v1-private.md)을 private 초안으로 고정했다. URL/선택적 length/type을 bounded binary로 전달하며 unknown/zero length를 구별한다. no trusted digest·no authority를 유지하고 부모는 원 URL/maximum을 다시 검사한다. HttpResponseOwner::wire_metadata는 validated 상태에서만 내보내며 metadata feature 소비자는 HTTP runtime을 자동 활성화하지 않는다. preimage Cargo/lib/request/Response와 첫 strict 전 metadata.rs는 `runtime/verification/source-http-metadata-preimage-r1`에 보존했다.

R9 inventory E4C4465D에서900 strict collapsible_if 첫 실패를 보존했고901은 실행하지 않았다. 동일 의미 let-chain 수정 후 선택 input-r10 SOURCE.md SHA256 `A97D64EF57A8543EFAFF143532E26CDBD8C7ADBD87780E385B6F5C6A6E9E1251`, inventory `E91F8F2941D93429EFAF4513E0B3AF4C16D915DCE25A10093C951A5CB1258CF1`,26files다. lock B7447F9D 유지.902 all-target/all-feature strict·903 순수/메모리35(기존32+metadata3)·905 fmt·906 content/identity 동일26 통과.904 normal dependency tree의 metadata-only 구성은 url graph만 포함하며 reqwest/tokio/rustls는 없다.

metadata3은 MSH1 Metadata 안 roundtrip·None/Some0/Some123 구별, 모든 truncation/unknown version/flags/reserved/불일치 length/trailing 거절, 비허용 URL/type/부모 원URL·최대크기 불일치 거절을 확인했다. 기존 Response 시험도 wire export를 실제 확인한다. 실제 metadata IPC/worker loop·parent current/replay·SDK·네트워크 검증은 아니며 원 제품190/bins·AI/GUI 인수와 구별한다.

## 후속 R8 — MSH1 incremental receiver/partial writer

R7 codec/lib preimage는 `runtime/verification/source-frame-transport-preimage-r1`에 보존했다. 기존 codec의 magic/version/flag/kind/declared-length 검사를 header_fields로 추출해 blocking decoder와 incremental receiver가 공유한다. wire 형식은 바꾸지 않았다. FrameReceiver는 header60byte 검사 후 최대 허용 payload만 복사하고 frame 하나와 consumed를 반환해 이어진 frame을 호출자에 남긴다. zero-body BodyEnd도 파싱하며 transport boundary EOF를 BodyEnd로 승격하지 않는다. parse/partial EOF 실패는 sticky다. binding/sequence/credit/current/cutoff는 후속 원 Exchange/owner 검사를 계속 요구한다.

FrameWriter는 전체 bounded encoded frame을 소유하고 lane maximum(1..64KiB)에 맞춰 offer한다. pending offer 동안 재제공을 거절하고 실제 completion count만큼 offset을 진행한다. partial completion 후 나머지 suffix를 다시 제공한다. zero/초과/unpaired completion은 sticky Io이며 성공으로 표시하지 않는다. B SourceLane::start_write는 자기 pending allocation으로 복사하므로 그 원 operation의 completion/close를 별도로 보유해야 한다. 메모리 frame writer가 원 OVERLAPPED lifetime/물리ACK를 대체하지 않는다. full frame staging+decoded payload 중복 할당을 포함할 수 있어 hard RAM/zero-copy 지원으로 표시하지 않는다.

선택 input-r8 SOURCE.md SHA256 `B2CDEC1D6136276430D322941EDCA7C7CE1A4C4417008857CFDCA6E4C376633C`, inventory `4FFC8003DEF70FCBFA29100AE5FA915DDB0E6FE68671761BE638CC93EFFBE6F8`,23files. lock B7447F9D 유지.895 all-target/all-feature strict·896 메모리/순수32(기존29+transport3)·897 기본 feature strict·898 fmt·899 content/identity 동일23 통과. 새 first failure 없음.

transport3은 header포함65596byte frame의1111byte partial write 완료 후 원 serialized bytes 일치,1/59/60/61/8192/65536byte 수신 조각 roundtrip, small-frame 모든 split·연속 BodyEnd와 boundary EOF, 초과 선언 조기 거절/모든 중도EOF/잘못된 completion의 sticky 실패를 검증했다. 실제 Windows named pipe·partial WriteFile 발생을 관측한 것은 아니다. typed Metadata payload와 worker loop/parent adapter·원 D replay 권한은 계속 미완료다. root 제품 checkpoint190/정상bins 및 AI/GUI 완료로 합산하지 않는다.

## 후속 R7 — 원 URL/HTML 거절 공백 수정

기존 `media-download-provider-r1/src/infrastructure/download/http_stream.rs`를 다시 읽어 source/status/url/encoding/range/type/length 검사와 비교했다. R6에서 원 URL 일치 및 HTML 거절이 누락돼 있음을 확인했다. R6의 성공은 그 당시 시험 범위이며 이 두 검사를 입증하지 않는다. private preimage4는 `runtime/verification/source-http-identity-preimage-r1`에 보존했다.

HttpResponseOwner가 필수 expected_url을 원 send owner에서 받아 exact Url 비교 후 metadata를 처리한다. 200이라도 다른 URL은 sticky Url 실패이며 HTML/XHTML type은 ContentType 실패다. Content-Type은 단일/256byte 이하만 허용하고, type이 없거나 다른 type이라는 사실만으로 실제 미디어 적합성을 승인하지 않는다. probe/parent 정책이 계속 필요하다. bootstrap은 Absent referer와 URL만 표현하며 Empty referer를 Absent로 바꾸지 않는다. Empty/auth/cookie 등 미표현 요청은 D/parent에서 Unsupported로 남겨야 한다.

선택 input-r7 SOURCE.md SHA256 `3E36A37ED4AAB2230D75FC56609D9579FFC8EF2B76FC8B444D2C9595152A0303`, inventory `8F26C716E50A5D6B3E68749167E9DEA3BEB2DD9B9B5B11BECD6E2DFB4E84EA9E`,19files, lock은 B7447F9D 동일.891 all-target/all-feature strict·892 메모리/순수29(기존28+URL/HTML1)·893 fmt·894 content/identity 동일19 통과. fixture response도 원 https URL을 명시하도록 수정했다. 실제 HTTP/redirect/권한 발급·SDK/OS 인수를 뜻하지 않는다.

다음 공백: Metadata payload는 기존 A가 bounded opaque bytes만 고정해 typed HTTP payload가 아직 미고정이다. 부모/child를 서로 다른 직렬화로 조립하지 않도록 별도 versioned payload의 길이·kind·length/trusted-digest 부재를 먼저 문서화해야 한다. IPC B data segment64KiB와 MSH1 header60+Data64KiB는 크기가 다르므로 한 frame을 한 segment로 가정하지 않고 bounded incremental frame reader/writer와 partial completion을 연결해야 한다. 원 B/제품 WIP·소유권·기존 보류는 유지한다.

## 후속 R6 — 원 send future 소유·전후 current checkpoint

R5 Cargo/lock/lib preimage를 `runtime/verification/source-http-send-preimage-r1`에 보존했다. http-worker가 request feature도 명시 활성화한다. HttpSendOwner::prepare는 GET request/client를 조립하고 unpolled send future를 보관한다. redirect none/retry never/no_proxy/자동 referer false/모든 자동 압축 off/Accept-Encoding identity/idle pool0/HTTP1/HTTPS only로 작성했다. local reqwest0.12.28 공식 source client.rs·retry.rs의 해당 builder 메서드를 읽었다. client builder 실행/설정의 실제 network 동작을 검증한 것은 아니며 이번 시험에서 prepare나 execute는 호출하지 않았다.

poll_open은 caller의 원 current/attempt/absolute-cutoff 검사 전후에서 future를 poll한다. 응답을 받은 즉시 owner.response에 넣고 post-check/metadata 검증을 수행한다. guard 실패는 sticky이며 원 future/응답을 같은 owner에 보존한다. abort는 pending future를 내려놓되 기존 response와 first failure를 보존한다. transport guard는 replay issuer가 아니며 원 D의 일회 권한을 소비하지 않은 호출자를 허가하지 않는다. 실제 생산 constructor/registry 연결은 없고 synthetic future constructor는 private다.

선택 input-r6 SOURCE.md SHA256 `099F82C455F825C8E2BF41407E27B9436B81D3E411F4365C7F391996A4D89DE5`, inventory `27D31AA9C846B428C7BA9D8AB833F5C5D8E5CDC319D9D2C51C9C3F489B82BEFA`,19files. lock은 R5의 B7447F9D 그대로다.886 offline lock·887 all-target/all-feature strict·888 시험28(기존24+send4)·889 fmt·890 content/identity 동일19 통과. 새 first failure 없음.

send4는 pre-poll invalidation에서 poll0/original future 보유·abort drop1, Pending 전후 검사2회·abort 후 재시작 금지, Ready response post-check/metadata 거절에서 원 response 보존, post-check panic 뒤 response 보존·정상 ready 단1회 경로를 확인했다. drop counter는 synthetic future 사실이며 실제 resolver 종료/remote 전송 취소/Job empty/IPC EOF/receipt가 아니다. reqwest error의 URL은 밖으로 전달하지 않는다. future 내부의 send/resolver/runtime 실제 소유·취소·deadline·프로세스 closeout 및 원 replay/SDK 연결은 계속 미검증이다. 원 보류 및 root WIP 유지.

## 후속 R4/R5 — 실제 reqwest Response 소유 어댑터

후속 실행883은 기존21+Response3=24시험 통과,884 fmt·885 전후 content/identity 동일17 통과다. Response3은 실제 reqwest 타입의 in-memory body를 사용한 시험이며 live HTTP가 아니다. 검증 전 read 거절·chunk drain 전 다음 read 거절·6byte 분할 후 EOF, status/encoding/length/max의 sticky metadata 실패, 본문 길이 부족·unknown-length 최대 초과에서 body_end 거절을 확인했다. 다운로드/실제 provider 지원으로 계상하지 않는다.

private preimage `runtime/verification/source-http-response-preimage-r1`에 R3 Cargo/lock/lib와 첫 strict 이전 http_response.rs를 보존했다. optional http-worker에 reqwest=0.12.28(default-features=false/rustls-tls), 시험에 http=1.5.0·tokio=1.53.3(rt)를 offline 추가했다. reqwest의 로컬 공식 crate source `async_impl/response.rs`에서 chunk·headers·From<http::Response<T>> 계약을 읽어 적용했다. 새 sender/client/worker main은 작성하지 않았으며 네트워크 실행0이다.

HttpResponseOwner는 Response를 먼저 보유한 뒤 status200·Content-Range 부재·identity encoding·단일 숫자 Content-Length와 maximum을 검사한다. 실패는 sticky이며 원 Response는 같은 owner에 남는다. pending buffer가 있으면 chunk를 추가 poll하지 않는다. 실제 response EOF와 길이 일치·buffer drain 전에는 body_ended=false다. chunk가 비어 있어도 호출자 control checkpoint로 즉시 돌아간다. upstream count와 expected/max를 검사하고 기존 worker buffer에 귀속한다. reqwest의 전송·헤더 수신 전 상한, DNS/취소/IPC/Job 물리정리, 원 replay/current/cutoff 검사는 이 어댑터의 입증 범위 밖이다.

R4 source SHA `AED84E1474BB8CE429AF45E9269CB9C9DF4F28A5D7F80639F5469C129DF31A26`에서 strict881 needless_return 첫 실패를 보존하고 동일 동작 tail expression으로 수정했다. 선택 R5 SOURCE.md `736194C11187B7866E45A76F745957B0DDEDCF508FB9D71E87EC0B7956A5D56F`, inventory `80765E59B834961956A3ED77FBB28EF80C2074367A06F438DE20CB58FB3EFE7E`,17files. isolated lock `B7447F9DEB46DB7D2AF51D770480A41B0E7EA2EC4D67A1955B2EEEAB87A3871C`는 root graph와 다르다. strict882 통과,시험883 결과는 아래 후속 실행 기록에 둔다. 제품 root/factory/source375 canonical 승격 없음.

## 후속 R3 — worker pending response buffer

private optional `worker-buffer` feature에 bytes=1.12.1을 고정했다. WorkerBuffer는 upstream chunk 한 개를 소유하고 잔량을 모두 완료하기 전 다음 install을 거절한다. offer는 현재 credit와64KiB 중 작은 크기의 Bytes slice를 반환하고 중복 offer를 거절한다. complete는 원 IPC frame 전체 길이와 일치해야 하며 partial WriteFile completion마다 호출하는 API가 아니다. pending Bytes+offset을 유지하고 slice가 owner Drop 후에도 실제 allocation을 보유함을 시험했다. credit 차감·원 Exchange/current/cutoff 판단은 후속 worker 책임이다. OS pending-write owner를 이 버퍼로 대체하지 않는다.

R3 input-r3/SOURCE.md SHA256 `F060E409E7351DE89015B89EE961D38CEB6F368AB2EA2E35EEB726C9D4049E86`, inventory `23C69B614986E016BB0E12C33031E237F8D27BA6D0ACEBB3C7F50E651A275CC0`,15files. 로그874 offline lock,875 all-target/all-feature strict,877 순수21(기존18+buffer3),878 fmt,879 content/identity 동일15 통과.876은 PM 명령의 toolchain 경로 `pc-windows` 오타로 child_spawn os3/실제 시험 미시작이며 보존했다.877에서 올바른 경로로 실행했다.

buffer3은 credit0 정지·10000byte씩 전체 원문 일치·중복 offer/불완전 completion/덮어쓰기 거절, 최대 credit에서도64KiB·zero-copy pointer·owner Drop 후 slice 수명, 빈/1MiB초과 chunk 원소유권 반환을 확인했다.1MiB는 candidate upstream-frame 허용 상한이며 upstream allocator/HTTP/TLS/OS를 포함한 hard RAM 보장이 아니다. 네트워크/다운로드/실제 IPC는 실행0, product factory 등록0이다. 이 구현만으로 실제 worker가 다음 response frame을 poll하지 않음을 입증하지 않으며 후속 통합 시험에 남긴다.

## 후속 R2 — bounded incremental receiver

private R1 preimage는 `runtime/verification/source-http-bootstrap-reader-preimage-r1/{request,codec}.rs`에 보존했다. R2 `input-r2/SOURCE.md` SHA256 `E2C5BBA3DA7557519B5AE3E0B663D4DFCB6C862A534A44A8CBE547104D8C7899`, inventory `A5B1C661A8B81F9A919FC657D4EC922A15F08F14BD384CAA2FE4550C31F991C1`,13files다. lock은 R1 그대로다.

BootstrapReceiver는 고정8KiB backing storage로 header52byte를 먼저 수신·검사하고, 허용 길이를 확인한 뒤 payload를 복사한다. push는 consumed count와 완성된 요청 하나를 반환하며 coalesced 다음 MSH1 frame은 호출자가 보유한다. 실패는 sticky, 완료 후 재전달은 거절한다. EOF 중도 수신은 Truncated이며 BodyEnd가 아니다. URL 정규화/decoder의 추가 할당·TLS/OS 메모리까지8KiB 상한이라는 뜻은 아니다. 민감 bytes는 Debug로 노출하지 않지만 zeroization 보장은 구현하지 않았다.

로그870 all-targets/all-features strict,871 순수18(기존12+bootstrap3+receiver3),872 fmt,873 전후 content/identity 동일13을 통과했다. 새3은 모든 split의 다음 frame 보존/정확1회 전달,1byte 수신/모든 중도EOF의 terminal 상태, invalid header의 sticky 거절을 검증한다. 실제 IPC·worker transport read·deadline/current/spent·HTTP는 미검증이며 이 의무는 호출자/후속 worker에 남는다. 제품 root·registry 변경 없음,190/정상bins 재실행으로 계상하지 않는다.

## 범위와 선택 입력

`runtime/development/source-http-bootstrap-r1/packages/media-source-host`만 수정했다. 제품 root의 media-source-host 및 source375 canonical 채택은 수행하지 않았다. 기존 MSH1 v1 Open의 16byte cutoff payload를 바꾸지 않고 별도 MHR1 v1 요청 preamble을 작성했다. 이 후보는 C worker 구현의 선행 구성요소이며 HTTP source worker/부모 어댑터/D 브라우저 replay 권한 구현이 아니다.

private `runtime/verification/input-r1/SOURCE.md` SHA256 `692A8491642CD9EA7535960F6115449CC99D2487BA8751FB6AE0ABC7F5ACF1B7`, inventory `F50A265F8BE70D71F3238B2B67002C362F88325D9124DFB09E409F057ABEC2AC`, 11files. SOURCE.md의 도구 생성 제목 Current canonical source snapshot은 이 private root의 스냅샷 라벨일 뿐 제품 canonical 승격이 아니다.

isolated Cargo.lock `FD870160D1E816C85E43EBCB6B318B43A2A325C3BCAB60E4A46B0CD38A89BE42`; url=2.5.8은 고정했지만 transitive는 offline local cache에서 새로 해석했다(ICU2.3 포함). root lock과 동일 graph 검증으로 계상하지 않는다. 네트워크 설치/다운로드 없음. C native/source-http-bootstrap-r1 target, 공통 tmp, stable MSVC1.99.1, resource-runner conservative2/CPU2, heavy1 유지.

## 계약

52byte header와 UTF8 target/referer의 합계 8KiB 이하, 32byte binding·nonzero maximum_bytes를 전달한다. HTTPS/host만 허용하고 실제 username/password·fragment를 거절한다. query는 민감 정보일 수 있어 request에 Debug/Clone/Serialize를 제공하지 않는다. 오류는 URL 값을 포함하지 않는다. 생성/수신 성공은 전송 권한이 아니며 D의 원 브라우저 소유자가 별도 일회 권한을 발급해야 한다. 제품 최대 크기 정책을 codec에 복제하지 않는다.

## 실제 실행

로그는 `runtime/verification/source375-datalab-b0-r1-logs` 아래다.

| 로그 | 결과 | 입증 범위 |
|---|---|---|
|863|fmt 첫 실패 보존|새 후보 형식 차이|
|864|fmt 통과|형식 수정 후|
|865|offline lock 생성 통과|private graph만|
|866|all-targets/all-features clippy -D warnings 통과|타입 포함 strict|
|867|cargo test all-features 통과|기존 codec12 + 새 bootstrap3, 순수15|
|868|기본 feature all-targets clippy 통과|optional request 미활성 구성|
|869|content/identity 동일11|고정 입력 검증 전후 동일|

새3시험은 민감 query/참조자 roundtrip과 MSH1 구분, 모든 truncation·초과/길이 선언·version/reserved 거절, URL 형태·credentials·fragment·zero limit 거절을 확인했다. 실제 OS IPC/HTTP·DNS·TLS·리다이렉트·취소·전체 메모리 상한은 입증하지 않는다.

## 다음 구현·보류

증분 bounded preamble read와 worker 수명/Response 소유, 부모의 기존 DownloadJobExecutor 연결, D의 원 세션/세대/current/spent 권한이 남는다. URL/200/GET/쿠키 부재로 권한을 추론하지 않는다. factory는 등록하지 않았고 제품 지원으로 광고하지 않는다. 원 B raw13/후계32·C2·root WIP/소유권·첫 실패·추론/다운로드/수동GUI/WARP/push 보류를 유지한다. 새15를 지정190 또는 전체CLI 인수 분모에 더하지 않는다.

기존에 읽은 modoo.today `software-qa/test-fixture-and-mock-build`(본문 A393F15F…F100), `code-change-review-verification`(9F73865F…A9), `postcondition/replay`(61843099…8B5)를 적용했다. 순수 fixture와 실제 네트워크를 구분하고 원 실패·전후 입력·정확한 검증 범위를 기록했다. 타 언어 구현은 도입하지 않았다.
