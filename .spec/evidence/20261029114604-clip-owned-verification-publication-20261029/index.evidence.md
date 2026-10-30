---
id: evidence:kinekino/20261029114604
title: 소유한 검증 후보와 publication staging — R198
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 소유한 검증 후보와 publication staging — R198

## R208~R210 Core 소유 세션 통합

private overlay의 ClipRenderJobExecutor가 engine-free `prepare_owned`에서 원 identity/deadline·공유 프로젝트/소스/도구/폰트/host guard를 보유하는 세션을 반환한다. 원 native driver/source holder의 unwind 정리를 유지하고 R207 ledger의 동일 parent 관측으로 source/inventory/boundary/encode/verify process의 종료·family·receipt·실제 일반 파일 EOF를 Core `OwnerObservation`으로 전달한다. serialized worker 선언을 읽어 종료로 바꾸지 않는다.

원 렌더 helper를 legacy/owned가 공유한다. owned 세션은 원 candidate를 자신의 필드에 먼저 보관한 뒤 staging하므로 staging 오류/콜백 unwind에도 해제 carrier를 Core owner가 유지한다. 성공 후보는 media/encoder/decoded/subtitle/font/PNG/MP4·staged file/root guard와 원 host 설정 guard를 함께 가진다. Core가 기존 settlement·permit·단일 publication CAS를 수행한 뒤에만 candidate.publish가 manifest를 commit한다. candidate는 reserve_publication/begin_verification을 호출하지 않고 Core가 각 상태를 소유한다. candidate.verify는 원 deadline을 유지하며 늦은 취소로 이미 발행한 결과를 취소로 위장하지 않는다. commit 오류는 publication_may_exist, post-commit 검사 오류는 publication_committed 및 manifest pin을 보존한다. poll_release는 R202의 원 MP4/PNG 해제 carrier를 유지한다.

취소 시험용 Paused 어댑터도 원 owned session을 감싸 staging 이후/Core CAS 이전에만 정지한다. candidate를 wrapper 필드에 보관하고 오류/해제 시 같은 원 candidate를 유지한다. legacy 시험을 Core 성공으로 계상하지 않는다. native 중간 강제 취소·OS close 오류·전체 pre-verification cleanup·ACK-loss 및 GUI 품질 인수로 확대하지 않고 partial_cleanup_supported:false를 유지한다.

수정 전 `preimage-r207-core-clip/`에 원 jobs/host/publication/renderer 및 취소 fixture를 보존했다. R208은 fmt/snapshot/native strict(2850~2852) 종료0이고 실제 시험 전 입력이다. R209 2462파일 inventory SHA256 `1D5629204B60ACB4EABA677A100B41109A37590D54B2F545C46FB878CC1A2938`, SOURCE.md SHA256 `773A5F12AE95108F12EC35E1A859AAFAF22BF6C1B2886B19539D578459C10D8A`. 로그2853fmt/2854snapshot/2855native strict/2856default strict 종료0.2857 combined no-run은 player-cli의 LNK1318 LIMIT(12)로 실제 종료1이며 실패 stdout/receipt를 보존했다. 당시 C 여유357548032바이트와 실패 PDB를 별도로 보존했고 [캐시 공간 근거](../environment/datalab-cache-space-20261029.md)에 조치를 기록했다. 실행 시험 성공으로 계상하지 않는다. R210에서 SDKCLI의 pre-CAS Paused fixture도 원 owned session을 전달하도록 고쳤고2858fmt 종료0이다. 현재 정상 bins/전체190 최신 완료 입력은 R197이며 미검증 변경으로 대체하지 않는다. 기존 QA fixture/변경 검증 skill과 Rust/소유권·private overlay·보류를 유지한다.

### R210 실제 통합 검증

2462파일 inventory SHA256 `14BDAF1523081DA0B2624940B2846A0F3B883E4C17A274AF1A8FE54D47F2D24A`, SOURCE.md SHA256 `5986817067810C30BACC8C15EF71DA8388B8DCB80885F76BCAAFFF7DDE8D0A54`. 2858fmt/2859snapshot/2860native strict/2861default strict/2862lib+Jobs+unitCLI no-run/2866정상 bins/2874fmt check 종료0. 실패 PDB를 보존하고 공간을 확보한 뒤 no-run이 성공했지만 PDB 오류의 단일 원인을 분리 재현한 것은 아니다.

고정 lib/Jobs exe/PDB는 mirror `fixed-r210/native-core/`, lib SHA256 `D778D0AEB033F2479BC077BB261036CD712F7C038C48FCA27DF4D92974D44AE0`, Jobs SHA256 `D78C00084A4A48CF519B2F39E776BC3F62471D82D03110F6DDD9E612975750D1`. 2863 원 Core Jobs 자막 없는 렌더/발행/재사용/저장 재열기, 2864선택 자막, 2865Core CAS 전 취소·정상 재기동·수동 attempt2 각1 실제 시험 통과. root `runtime/verification/source-sdk-r210-jobs-plain-r1/`, `source-sdk-r210-jobs-subtitles-r1/`, `source-sdk-r210-jobs-retry-r1/`.

정상 exe/PDB는 mirror `fixed-r210/native-bins/`, CLI SHA256 `0F84968DB525A750D9A1B67C584F368B36E0542EB7E5542C05F417529BE70A77`, GUI SHA256 `A19816B0D8BDE6E64C1530E414B8D9D1A836DDAA57E21BA6C89F2DAB146DDB91`. GUI를 수동 실행하지 않았다. 2867선택 자막16CLI/typed·원Jobs2완료,2868기본15CLI/typed·원Jobs2완료,2869실제8CLI 취소 ACK/조회/재전송/수동 attempt2/재조회가 통과했다. root `runtime/verification/source-sdk-r210-cli-r1/`, `source-sdk-r210-cli-cancel-r1/`. 취소는 in-process SDK LocalHost fixture의 원 Core 경로이며 실제 ACK 유실을 주입한 것은 아니다. 원 caption은 manual fixture이고 사용자 AI 품질 승인이 아니다.

2870Product145/2871Core18/2872Native26/2873OS1의 각 group-return.txt가 execution_result=Ok/producer_integrity=Ok와 위 동일 SOURCE SHA를 보고했다. Meari3그룹은 보존된622/624 producer를 이 호스트에서 실제 재실행했으며 새 컴파일로 표시하지 않는다. 2875사후 custody 원 실행 종료0,2462파일 내용·식별 동일성을 확인했다. 전체 제작 인수0/5·AI 사용자 품질0/9·GUI 시각 검토 및 기존 보류는 유지한다. stage/commit/verify/release 전 구간 fault·인코딩 도중 Core 취소·Core outer deadline의 post-commit 진단·실제 ACK 유실/비정상 중단 복구는 이 성공 경로 인수에 포함하지 않는다.

## R199/R200 렌더 context 소유권

이하 Core 미등록 판정은 R199/R200 당시의 기록이다. 현재 clip 원 Jobs의 Core 소유 경로와 검증은 위 R210 범위를 따른다.

원 Jobs가 `RetainedClipRenderCandidate`에 verified bytes/proof와 media·encoder/preset·decoded raw/PTS·자막 문서/style/font·PNG working input을 함께 이전한다. Context는 source custody·프로젝트/문서/preset·decoded window·subtitle/overlay binding을 확인한 뒤 보유하며 기존 publication이 끝날 때까지 해제하지 않는다. borrowed boundary/overlay는 원 데이터를 이동해 소유 형태로 바꾼다. Core owned-session 등록은 아직 하지 않는다.

Source lease마다 비직렬화 Rust custody 식별을 부여하고, retained decoded 근거가 그 원 lease에 속하는지 확인한다. 같은 source pin을 재입수해도 별도 lease 식별로 대체할 수 없다. worker JSON·진단 manifest에서 이 식별이나 context를 재구성하지 않는다. 원 candidate context가 worker로 이동 가능한 Send 타입인지도 컴파일 계약으로 확인한다.

수정 전 `preimage-r198-render-context/` 보존. `input-r199`2457파일 inventory SHA256 `339475A539F7769C1D0709A48DC3C7C3BFA666F4DB6F7C4B5F1B09EAB52B5CE0`, SOURCE.md SHA256 `804AB8244B32A49ABC3190EDFF558F26BA6346E886417772C8411F6AB6BC7C75`. `2791`fmt/`2792`native strict 종료0. 추가 실제 동일pin/다른lease 거절 시험 전 원 boundary 시험은 `preimage-r199-source-custody-test/`에 보존했다.

`input-r200`2457파일 inventory SHA256 `A5308F207F95400AC574F1D0C42AF4D0D98CDF4770D3F51E9AE13808AFB9105B`, SOURCE.md SHA256 `7710B377723CA6A9246BD157587D61873DEDE10CD0448D46036F427CE23EDE4F`. `2793`fmt/`2794`native strict/`2795`default strict/`2796`lib no-run/`2803`fmt check 종료0. 고정 lib exe/PDB는 mirror `fixed-r200/native-core/`, exe SHA256 `4A0004E457CB77761BD85AA54805271A5D4298439E6A305959037D59F477FDF8`.

`2797`은 후보 Send 컴파일 계약1시험 통과이며 실제 스레드 인수로 표시하지 않는다. `2798-source-sdk-r200-source-custody` 실제1시험에서 기존 decoded PTS/stream/clock 경계와 retained 원 source 식별, 같은 pin의 재입수 lease가 원 소유자로 인정되지 않음을 확인했다. root `runtime/verification/source-sdk-r200-source-custody-r1/`의 source/frame/boundaries/verification/custody-verification을 보존한다.

`2799` 원 Jobs 선택 자막 렌더·발행·재사용·재열기, `2800` 자막 없는 원 Jobs, `2801` 취소·정상 재기동·수동 attempt2/부분 증거 보존 각1 실제회귀도 통과했다. 취소 전 원 PNG/MP4 입력2개가 유지되고 종료 뒤 부재함을 확인했다. root `runtime/verification/source-sdk-r200-jobs-subtitles-r1/`, `source-sdk-r200-jobs-plain-r1/`, `source-sdk-r200-jobs-retry-r1/`. `2802-source-sdk-r200-custody` 종료0,2457파일 content/identity 동일 확인. Core owned-session/permit 연결·실제 ACK 유실/중단 복구와 신규 SDKCLI/정상bins/190 통합은 미완료이며 통합 검토 빌드는R197을 유지한다.

`OwnedVerifiedClipCandidate`가 원 EncodedClipCandidate와 원 verification proof/임시 입력 파일 guard를 함께 소유한다. Clone/Deserialize 경로를 제공하지 않으며 기존 borrowed verifier API는 유지한다. `verify_owned`는 같은 verifier가 만든 원 proof를 이동하고, 읽기 view의 수명은 소유자에 묶는다. 원 Jobs도 이 소유 객체를 사용한다.

publication은 검증된 파일3개·manifest bytes/SHA·writer/root/directory/file guard를 소유하는 준비 객체로 분리했다. 현재 stage와 commit은 publication 내부에만 있고, 기존 publish가 원 control.reserve_publication 뒤 한 번만 commit한다. private commit은 재시도를 거절한다. Core owned 후보가 기존 gate를 우회해 commit할 새 API나 가짜 ExecutionControl은 만들지 않았다. 취소 전 부분 파일 보존·commit 후 late cancel/timeout 재열기 정책은 유지한다.

- 수정 전 mirror `runtime/verification/preimage-r197-owned-clip-proof/`에 verification/renderer/tests/publication/jobs를 보존했다.
- `input-r198`2456파일 inventory SHA256 `2AF72D3D63A28D80125DD89D2BE453D1E42C65035F832A484E24D83375CBF838`, SOURCE.md SHA256 `A5E48775C7B6D338F02209B5F8122280C6FF9AD888C9333A0977C7737C2CFC27`.
- 로그 루트 `runtime/verification/source375-datalab-b0-r1-logs/`. `2783`fmt/`2784`native strict/`2785`default strict/`2786`lib no-run 종료0. 고정 lib exe/PDB는 mirror `fixed-r198/native-core/`, exe SHA256 `4411D29965EC510FB90BF1445F6EC36EE4AEA8383EDB53A03790A4FE0FE4012D`.
- `2787-source-sdk-r198-owned-proof` 실제1시험 통과. 읽기 view drop 후 원 proof 입력이 유지되고 owner drop 후 부재함을 확인했다. 기존 MP4 metadata/full decode·파일 손상 거절·재열기, 원 reservation1회·commit 전 취소와 commit 뒤 late cancel/timeout 결과 보존도 통과했다. root `runtime/verification/source-sdk-r198-owned-proof-r1/`에 원 실제 candidate/artifact/probe/검증 증거를 보존한다.
- `2788` 실제 원 Jobs 선택 자막/발행/동일key재사용/재열기, `2789` 취소·정상 재기동·수동 attempt2/입력·부분 증거 보존 각1회귀도 통과했다. root `runtime/verification/source-sdk-r198-jobs-subtitles-r1/`, `source-sdk-r198-jobs-retry-r1/`. `2790-source-sdk-r198-custody` 종료0,2456파일 content/identity 동일 확인.
- 정상bins/190 최신 통합은R197이며 Core owned-attempt의 후보/permit/publish 연결·전체 정리/중단/ACK 인수는 미완료다. staging의 독립된 Core 호출 경로를 공개하거나 전체 owned executor로 등록하지 않았다.
- 기존 QA fixture/변경 검증 skill 적용을 이어받아 원 proof와 JSON 진단을 구분하고 실제 미디어·원 gate 근거로 검증한다. ownership/보류/private overlay 및 사용자 품질 승인 경계를 유지한다.
