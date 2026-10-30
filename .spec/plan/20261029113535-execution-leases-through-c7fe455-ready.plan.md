---
id: plan:kinekino/20261029113535
title: 현재 실행·편집 배정
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 현재 실행·편집 배정

2026-10-21 중앙 PM 소유. root `c7fe45510d05ba6297240354abc149ae46cc6f66`, Meari `22fe0d952f5d03d28147d12fcce5f1914fe26bbc`.

이 문서의 현재 표가 실행 권한의 정본이다. [직전 원문](history/execution-leases-through-9ffd855.md)(SHA256 `060315A1F0CA4CD1EC67D79DB1E954DC8F534980264194882B936AEF0F38F73B`)과 [이전 배정](history/execution-leases-through-5040f22.md)은 보존 이력이며 재실행 권한이 아니다. 결과는 [현재 조율](current-coordination.md), 전체 완료 조건은 [통합 게이트](full-product-integration-gates.md)를 따른다.

## 현재 담당

| 담당 | 현재 작업 | 권한 |
|---|---|---|
| 미디어·검증 | VAD 정상1/취소1 원반환 완료, 신규 media-vad P0 source10 구현 | source-only·직접fmt/회귀작성. 추론/재빌드/다운로드 추가 배정0 |
| SDK·CLI | 실제ready/EOF1 반환 인수 후 원 저장resolution의 native 재생·guarded 결과1 | **유일 heavy**. 기존 실제media exact1 새 checkpoint. 입력/기대치/예산 유지·자동재시도0 |
| 플레이어 UI | union12 동결, 공개 Viewer 실제 제품caller6 구현 | source-only·직접fmt/회귀작성·인계. Cargo/OS/미디어0 |
| Meari | union20r2 동결, 실제 loop 결속 caption/fullscreen port 구현 | source-only·직접fmt/회귀작성·인계. Cargo/package/OS0 |
| 중앙 PM | 독립 감사·원 실행 인수·후속 배정·정확 stage/commit | root/index/gitlink 단독. push0 |

네 담당은 사용자 직접 전환 뒤 get_goal active를 확인했다. 본문 편집 도구 제약과 과거 blocked는 [목표 복구 관측](goal-recovery.md)에 보존한다. 실행 슬롯 대기를 전체 blocked로 처리하지 않는다.

사용자의 active 재전환 지시는 네 담당 모두에게 다시 전달했다. 기존 전체 goal을 유지하며 현재 실행 계획을 갱신한다. objective 변경 도구가 없다는 이유로 미완성 goal을 complete 처리하거나 내부 DB를 수정하지 않는다.

## 최근 인계와 다음 읽기

- AI r2 정상 BIN은 실제 링크에서 `DMLCreateDevice1` 미해결로 실패했다. 원 terminal101과 source를 보존한다. r3는 build.rs 정확1에서 Windows/MSVC·명시 ORT_LIB_PATH 조건의 기존 DirectML import library 링크를 추가하며 나머지12는 불변이다. DLL·CPU EP·모델·전역 환경 변경은 없다. 같은 r3의 두 strict/fmt는 통과했으며 정확7과 정상 BIN은 별도 최종 확인 대상이다.
- UI r2 source8 `63E955DB7AE0B8C75396EB0B72005A194C4A82CB3ED6A7CB63C7458C49874297`은 실제 close surface+1과 wake Ok(false) 병합 성공을 반영한 시험 단언2 수리다. 실행 성공이 아니라 source 검토 결과다. 연구 담당이 변경1/불변7과 순수2를 독립 검토한다. 작성 중 r3의 trace 용량 검토는 별도 핀으로 인계하며 r2를 덮어쓰지 않는다.
- Meari CP01 source5 `3D3F897C58D7000CE6CC4DE5173A76233FCF281329A38E529E4AB675C979C952`은 opaque 값·owner·예산·transaction과 작성 회귀12다. 독립 읽기를 배정했으며 Cargo/타입/실행0이다. Q09와 union 검증은 후보이고 아직 합성·실행 권한이 아니다.

### 후속 검토 반영

- PM가 r3 실제 raw를 읽어 VAD validation3·request1·invalid handle1·borrow doc2의 고유7 통과를 확인했다. doc 거부는 원 E0515/E0505다. 정상 BIN r3는 별도 원 runner85728/child34760에서 진행 중이며 이 7건을 추론 성공으로 세지 않는다.
- UI r2 실제8의 bytes/SHA와 변경1/불변7을 독립 검토했다. 두 정상경로 단언의 수리는 source 수준에서 확인했으며 타입·OS는 미검증이다. 후속 r3 trace 용량은 작성 중 별도 핀이다.
- Meari CP01 독립 감사에서 caller String/key의 len만 과금하고 큰 capacity를 그대로 보존하는 예산 우회를 발견했다. 같은5 범위 r2 수리·overcapacity 회귀를 배정했다. 원3D3F를 보존하며 수정 전 source를 검증 게이트에 넣지 않는다.
- SDK CLI help1 `E1BF11725F417880265E85F5AED3A4929161C3440928EAEE7FB2AFDD81AD6C39`은 실제 등록된 subtitles.convert/export_range 두 operation의 안내 수리다. PM가 정확1줄 diff를 확인했다. parent6와 교집합 없는 union7의 계획/핀 준비만 배정했으며 validation9는 별도다. 실제 정상 CLI help 검증은 후속 슬롯에서 수행한다.

### 현재 source-only 확장 배정

- UI r3 source8 `AB23227D84ACF5E2391A744BF1687810A6602D3588EE29F7263D166A8A1EEA3D`은 독립 감사에서 실제8 일치·probe1 변경/7불변을 확인했다. trace80과 순수 회귀의 70사건은 정상 시나리오 기준이며 모든 revocation notice를 포함한 보편적 최댓값은 아니다. 예상 regular69/ignored7은 실제 실행 수가 아니다. 이 핀은 동결한다.
- UI 후속은 별도 root9ffd 기반 overlay의 native-default 커서/컨트롤 제품 연결이다. 현재 경로를 먼저 조사하고 hover 소실·전체화면 idle 숨김/입력 복원·패널/메뉴 유지에 기존 공통 계약을 연결한다. 책임별 파일 목록을 기록해 source-only 구현/직접fmt/회귀 작성/핀 인계까지 진행한다. 새 SDK/Meari API 부족은 실제 서명으로 조율한다. source8와 중복 파일은 별도 delta로 관리하고 무단 합성하지 않는다. Cargo/OS/shared0이다.
- Meari CP01r2 source5 `141566FAA3D255624BA6928AAE5CB003C9E05A48752FE320F0FDE056D276C4DB`은 독립 읽기로 capacity 수리를 확인했다. 작성14/실행0이다. Q09r2와 union20 준비·정확 게이트 계획 및 CP02 연결 조사만 배정하며 Cargo/shared0을 유지한다.

SDK union7 후보 `2440683CCECD94A72B867CB9A2C7F5F1B6EFAF2E58D7D0D38C59A8FD12A9A870`의 계획을 PM가 읽었다. source6와 help1 원 핀/교집합0을 보존하고 순수1+9+11=21(신규6 포함), 정상 CLI help1 관측을 별도로 인수한다. AI build 및 별도 정상/취소 추론 원회수 후 다음 CPU-heavy 후보로 우선한다. 현재 실행 허가는 아니며 실제 준비 상태 관측1도 별도 배정이다. AI resource4와 SDK supervisor 변경은 같은 crate의 별도 경로이므로 당시 root/before/반환 source를 확인하고 합성 범위를 기록한다.

PM가 r3 정상 BIN의 원 build-finished success=true와 receipt terminal=true/exit0/145858ms를 확인했다. 원 runner85728/child34760의 현재 부재도 확인했으나 이 사실만으로 정상 cleanup이나 전체 인수를 만들지 않는다. artifact는 정상 profile.test=false·opt2·debug_assertions=true다. 담당의 최종 SHA/provenance/보호·원handle 회수 및 명시 반환을 기다린다. 실제 VAD 추론은 아직0이다.

## 새 배정: 고정 VAD normal1

build closeout의 원27receipt 전terminal(최종25zero/원실패101둘), 기록PID62 현재부재, artifact 차이0·root/shared1414/snapshot234/source13/입력16/index·기존ORT/DirectML 보호차이0 및 heavy/shared 반환을 인수했다. PM는 raw7·기대doc오류·정상artifact·preserve-build receipt/provenance228을 읽었다. provenance227 source는 명시 수집 범위이며 registry/OS DLL 전체 closure가 아니다.

미디어 담당은 자원 진입 기준을 다시 확인한 뒤 같은 BIN/동일 실행 디렉터리에서 `bounded-vad normal FRESH_ABSOLUTE_RUN_ROOT`를 **1회** 실행한다. 기존 Silero HF revision/MIT/FP32/modelSHA·25초 WAV SHA, CPU EP/intra2/inter1·conservative2/affinity3/BelowNormal, 원 total30/cleanup5와 4GiB 관측 목표를 유지한다. 원 runner와 내부 Supervisor/Job/Deadline·worker 신원·결과782프레임·실제 loaded DLL5·최종 OS peak·원 finish를 확인한다. peak 조회는 cleanup 증거가 아니고 4GiB는 hard cap이 아니다. 원 raw/receipt/출력/입력·소스·binary pin을 보존한다. 오류나 timeout으로 자동 재시도·입력연장·예산증가·DLL교체·새download0. 취소1은 이 결과를 읽은 뒤 별도 배정한다. 실행 종료 뒤 문서보다 먼저 명시 슬롯 반환을 보고한다.

## 병렬 후속 source-only

- Meari union20 `AAFF150AA4BE55EDA8B4E89334DBB2971697C28B45AB0C39202E5266FEEF70EE`는 Q09r2+CP01r2의 독립426 사본이다. 예상 Rust25와 Cargo 목록명령3을 분리한 계획을 인계받았으며 실행0이다. 별도 CP02 overlay의 hook_values.rs/tests.rs 정확2 staged-read API 구현을 배정했다. 현재 후보/committed snapshot의 유효값 읽기·scope/수명/poison 보호와 회귀를 작성한다. StateProgram/HookSlot/sourceIR/queue/HMR 변경은 아직0이다.
- SDK는 기존 CLI provider jobs가 hidden WebView2 어댑터와 같은 host에 이미 연결됐음을 소스로 확인했다. 새 중복 host/GUI 조작을 추가하지 않는다. 독립 jobs.rs+신 episode_target_contract.rs exact2에서 보존 archive metadata/DOM을 실제 parser/strict target계약으로 읽는 bounded offline replay 회귀 작성을 배정한다. 유효 실제 입력 부재는 부족으로 인계하고 합성물을 actual로 바꾸거나 allowlist를 확장하지 않는다. union7/validation9와 무단 합성·네트워크·host·Cargo0이다.

UI chrome4 `8C1F48851802EF674418E1C9EEE33F36CC5609D4FDDE93469F8AEAA99DB0E07C`은 독립 source 감사에서 actual4/egui 계약/기존 ControlVisibility·입력 조건 보존을 확인했다. 이전 렌더 rect 소비의 현재프레임 resize/occlusion과 실제 child pointer는 미검증이다. source8r3와 별도 union12 준비·정확 계획을 배정하며 원 부모 핀은 보존한다. 69 예상에 새4/기존visibility1을 더할 때 중복을 제거하고 ignored OS2와 분리한다. Cargo/shared/OS0이다. 커서는 playback wire에 혼합하지 않는 좁은 window/surface 포트 후보를 읽기 조사하며 아직 새 포트 구현 권한은 없다.

## 새 배정: 같은 VAD cancel1

normal gate `runtime/verification/ai-vad-normal-r1`의 실제 raw/receipt/closeout을 PM가 읽었다. 25초/782scores·speech254·CPU2/1·load0.1425539s/inference0.2355821s/worker전체0.4747055s, outer3593ms다. 최대 working set42188800B와 Job peak27308032B는 다른 지표다. loaded DLL5 pin 일치, original62532/creation134370157095029049 child_exit0/Empty/forcedfalse/stdinEOF와 final Result 성공, outerterminal0·기록PID3부재·source234/root/shared/index/binary차이0·명시반환을 확인했다. 감지 정확도·GPU·제품 SDK 지원·품질 승인은 입증하지 않는다.

같은 source/BIN/모델/25초 입력/CPU2/1·원 total30/cleanup5로 fresh run-root의 `bounded-vad cancel` **정확1회**를 배정한다. 실제 첫 chunk 처리→원request ready→parent 신호1→worker 관측의 단조 타임스탬프/원identity·cancelled1frame/성능비교 제외를 확인한다. 빠른 정상완료나 강제종료를 취소 성공으로 바꾸지 않는다. 원 오류를 보존하고 자동 재시도·입력/예산 연장·파일교체·다운로드0이다. 원 finish/receipt·peak·보호와 슬롯 반환 뒤 SDK union7 순서다.

## 새 현재 실행: SDK union7 gate

취소1의 raw/원request ready1077955187→signal1077955234→observed1077955296·원identity를 PM가 읽었다. 실제1frame cancelled/performance비교false, 원59336/creation134370158396451854 exit0/Empty/unforced/EOF, outerterminal0/3803ms·보호차이0/명시반환을 인수했다. 품질·제품SDK지원은 false다.

PM는 검증된 resource4만 원before3/신규부재1 및 최종 bytes를 대조해 root `e93a0b7`로 채택했다(114+/1-). tool9 부분공개·모델/캐시 채택0, 기존Meari문서2 보존·index비움. 이제 SDK 담당은 자원기준을 확인하고 기존 shared1414/기존native cache에 **채택된 root resource4를 baseline으로 먼저 명시 반영한 뒤 자기 union7**을 통합한다. 총경로11이며 원7·4 before/최종/비입력과 당시root/index를 별도로 고정한다. 같은 crate지만 경로교집합0이며 원source 핀을 덮어쓰지 않는다.

원 runner/conservative2/offline locked -j1/thread1로 default/native 및 관련 owned all-target strict·fmt, 기존고정21(신규6 포함), 동일최종 정상CLI build/pin/fixedcopy `--help` 1회 관측을 수행한다. actualready/미디어/validation9/replay2/GUI0이다. 첫오류 보존·수리별도핀, 원handle/receipt/family·binary/source/비입력/root/index 대조 뒤 명시 슬롯 반환을 먼저 보고한다. 수리가 resource4의 의미·권한을 바꾸면 별도 조율한다.

실제 gate는 `runtime/verification/native-ready-help-union7-r1`이다. PM는 shared integrated11 실제해시차이0·baseline1414/root1455·preflight RAM30.06GiB/VRAM5718MiB/C189.09GiB·원runnerEEA9를 읽었다. strictdefault 원runner64328/child99072의 nonterminal은 실행 진행 관측이며 성공/중단 판단이 아니다. 원handle을 계속 회수한다.

별도 replay2 `8704C1D0B167A77098CFB09C2B7D0DCA0F02F6C07BB6916F85C3C44B8CE77E1C`는 독립감사에서 actual2와 입력6의 bytes/SHA·원metadata세대/시간/nullstatus·실parser/strict target호출·출력URL비노출을 확인했다. 종료후 입력재읽기는 캡처2 범위로, resolution/보존16전체 불변을 대신하지 않는다. 작성pure2/ignored1은 아직실행0이며 SDK현재gate에합성하지않는다.

SDK default strict는 원receipt terminal0/65097ms와 원stderr의 owned-process/제품 all-target 확인까지 PM가 읽었다. native/시험/정상CLI 결과로 확대하지 않는다. 다음 후보의 실제 최종 입력도 PM가 독립 대조했다: UI union12 `83B9084EF81F17D418C80786D2F882DB6E6D9B853F6CA48476530C1BFC543EFD` actual12 해시차이0, Meari union20r2 `27B391829DFD67AE42AA5DF5628B24C6FB0B65A93AC322E9ED5713D65969A0E8` actual20 해시차이0이다. 이 대조는 컴파일/실행 또는 UI 전체 baseline426 검증이 아니다.

## 새 현재 실행: 실제 ready/EOF1

SDK gate closeout의 순수21·정상help1·양제품/owned strict/fmt·21receipt 전terminal0/기록PID42부재·보호1417/root1455/bundles382/index/binary차이0·원handle 전회수/명시반환을 인수했다. PM가 실제raw21명과help문장/receipt를 읽고 source7 원before·최종bytes를대조했다. 준비관측6은 `3659d0a`(760+/21-), help1은 `c7fe455`(1+/1-)로분리채택했다. 기존Meari문서2·원실패근거는보존했다.

SDK 담당은 자원기준을 다시 확인한 뒤 gate `runtime/verification/native-ready-only-r1`에서 정확 ignored `ready_observation::actual_original_native_ready_and_eof_observation`을 **1회** 실행한다. accepted integration DEF882AA42D6336D2D2DC3D49EEDC553AC832E45266F64F0D6DBBEC88B9D3D0F/5826560B 및 같은최종 정상CLI F7D7D9AB2A3A935997AC663AF1BA36C6BC1743BE4777E4ADF077AFA0327C7310/30148096B·원/fixed/Fixture-copy pin을 고정한다. CLI feature는 native-proposal-window다. copied catalog16 `native-export-actual-media-input-r1/catalog`와 원 mpvF8BF/121051136B를 유지한다. 원 total90/cleanup10/ready10/stop10·parent summary2048 bound를 유지하고 ready→즉시stdinEOF만 확인한다. endpoint request/미디어/mpv process/네트워크0이다.

원 ready오류·phase timing/Unknown·child/family/EOF/pipeSDKjoin·poison/종료증거를 보존한다. cutoff를 늘리거나 warmup/자동재시도·input/기대치변조0이다. same handle·fresh 원 evidence·root/index/source/CLI/catalog/mpv/binary·기록PID 신원을 대조하고 반환한다. ready성공을 playback/ACK/landing/실영상 성공으로 승격하지 않는다. 이전 실제media 실패는 별도 이력으로 유지한다.

## 새 현재 실행: 원 stored media1

ready-only1의 raw/summary349B/360906us·원24088/creation134370169653159767 exit0/Empty/unforced/EOF·stop명시null/pipeSDKjoined/idleunpoisoned·3PID부재·root1457/snapshot1417/bundles501/입력16 양쪽·binary/index 보호0·명시반환을 인수했다. 이전 ready 지연은 재현되지 않았으나 원인 확정·영구 해결을 주장하지 않는다.

SDK 담당은 자원기준을 다시 확인하고 fresh `runtime/verification/native-export-actual-media-r4`에서 accepted SDK DEF882AA 및 정상CLI F7D7D9AB의 기존 ignored `actual_export::actual_original_guarded_export_ack_landing_and_consumption` **1회**를 배정받는다. 원 actual-media-r3 환경의 copied catalog16/저장resolution bodySHA79478334a5886c30f80371c2450ed18c67e0f7e9503c7b3c79e72e11ec8e31a0/artifact5c23ef31004a11da985766fd89f3a9e0da7950fd327d5cdeadfabae0fd012aa3·mpvF8BF/121051136B·target5000000us/tolerance250000us·원 total90/cleanup10와 각 phase 예산을 그대로 유지한다. accepted 원/fixed/Fixture CLI와 원/fixed integration을 다시 고정한다.

이 실행은 명시 원 playback.open→원 native observation/guard→submit/ACK/landing/consume→EOF/제품joins·원mpv/host family 종료의 실제 단계 관측이다. 원URL/입력/guard/schema/target/허용정책을 바꾸거나 새resolution으로 치환·warmup·자동재시도하지 않는다. 만료/네트워크/미디어/ACK/cleanup 오류와 원 owner/receipt는 별도로 보존한다. 임의셸/브라우저조작/사용자앱 종료0·manualGUI검토0이며 이 CLI 기능 시험을 세카테고리/전체광고차단/장시간/시각 성공으로 승격하지 않는다. 보호와 원handle 회수·반환 뒤 다음 경량 P0 계약 gate를 배정한다.

## 최근 독립 source 감사

UI 제품caller6 r1 `9738F8B0C2A80D6A59E6101E1299467201B433E6DB17E3F3B94E2E1DFC1AAA6B`에서 결과 진단의 stdout/stderr 쓰기 실패가 panic하여 원owner를 유지한 비정상 exit 경계 전에 unwind할 수 있는 문제를 발견했다. 같은6 범위 r2 fallible writer 수리·BrokenPipe 회귀를 배정하고 원r1을 보존한다. 기존 NativeProposal 동일 위험은 별도 잔여이며 범위 밖 코드를 무단 수정하지 않는다. 이 source는 수리 전 실행 인수하지 않는다.

AI P0 신규10 `B79C4C47616BB4432A2C079452D36C0FF6EEA04F88E418354B4AEAB98674D880`의 독립 읽기를 배정했다. 작성fixture9는 실제 모델 추론이나 제품 지원이 아니다. 후속 tests1 range/padding 보강은 원r1을 보존한 별도 핀으로 받는다.

P0 최종r3 `F7E04F06574147495B7A2DF99E0C5ADBAEB789109455B1C684C3EA71496E1E8C`은 독립감사에서 actual10·revision별delta·선택range/PCM/padding·Partial의 profile/input context 보존과 Complete 비승격을 확인했다. 작성11/실행0이다. S=`runtime/verification/media-vad-p0-r3/source`에source10+sibling9+workspace/AGENTS/초기lock3=22 조립을 인계받았다. 기존D tool target재사용은 후보이며 Cargo/lock생성/실행은 아직미배정이다. media-contracts 자체license 파일부재/전체transitive closure는 배포 미인수로남긴다.

자원 연구의 저장된 과거 `runtime/verification/resource-topology-r1/topology.stdout.json`은 mask3가 같은 물리코어 SMT쌍이고 mask5는 다른코어 후보임을 기록했다(candidate미적용). 현재OS 새관측·가속 실측·profile구현은없다. 현재runner conservative2/EEA9·원liveprocess 설정은 바꾸지 않는다. 이후 별도물리분산profile을검토하더라도현재허용affinity 교집합·2logical/BelowNormal·원Job/회수·반응성 확인 뒤 사용해야 한다.

## 제품 연결의 병렬 구현 배정

- UI: 조사한 product Viewer caller 정확6을 source12 기반 별도 overlay에서 구현한다. 기존 main client/LocalHost/NativeExecution을 공개 prepare/run/finish로 한번 이동하며 실패 owner/endpoint를 보존한다. NativeProposal을 일반 Viewer에 유용하거나 두번째SDK/shutdown·자동fallback을 만들지 않는다. mod.rs 교집합1은 개별 before/after·precedence로 관리한다. 명시 실험 선택 경로이며 미검증 기본flip은 하지 않는다. 실제 media·caption/transport·13화면/Meari기본 전환은 전체 잔여로 유지한다.
- Meari: source412에서 minimize/maximize/fullscreen의 현재 공개 command가 없는 실제 gap을 확인했다. 별도22fe0d overlay에서 원loop/UI thread·WindowStamp/Open수명 결속 typed window action port와 snapshot/capability를 구현한다. 시작 전 정확파일을 기록하고 값·application계약·nativeadapter를 분리한다. 원close/guards/childfirst는 유지하고 rawHWND/owner getter 공개·전역창조작0, 접수와 실제OS반영을 구분한다. 기존 union20r2와 CP02는 보존하며 추가 Cargo/OS0이다.
- AI: 제품 jobs에 cached VAD를 연결할 최소 독립 Rust 패키지 승격/public소스closure·고정 model profile IO/provenance/attempt/cancel/immutableoutput 계약의 구체 구현 범위를 조사한다. 기존 tool9 부분공개·머신 경로hardcode·schema만 추가한 지원 주장은 하지 않는다. 추가 모델/추론/다운로드/빌드0이다.

AI 조사 후 P0 신규 `packages/media-vad` 정확10의 source-only 구현을 배정했다: Cargo.toml/lib.rs, domain mod/profile/input/result4, application mod/ports/analyze3, tests/contract.rs. 공통media-contracts 오류/값을 재사용하고 bounded 실artifact bytes/hash·입력/결과·partial비성공을 검증한다. domain에OSpath/argv/ORT0, rootCargo/registry/기존공통jobs변경0·추가Cargo/추론/download0이다. Job execution identity와 Cancelling 뒤 executor오류 보존 계약은 SDK와 별도 합의하며 실제 제품지원false다. 품질manifest/실험provenance/실행profile을 서로 다른 책임으로 구분한다.

SDK의 별도 jobs P0 exact5(source-only)를 배정했다: media-jobs application 새execution_identity와 mod/ports/worker/tests다. ExecutionControl defaultNone 호환과 원startCAS Runningrecord의 private/noDeserialize JobAttemptIdentityV1 읽기만 추가한다. job/attempt/inputSHA/sourceblobSHA/optional submissionrequestSHA를 구분하며 APIrequestID·publication권한·cleanup권한은 만들지 않는다. 원cancel/state/workerstop 정책은 바꾸지 않는다. P1은 executorErr 뿐 아니라 stop_owned의 unpublished ACK·retry허용·hostlease와 Retained 원owner를 함께 보존해야 하므로 별도 설계다. actualmedia gate에 무단합성하거나 추가Cargo를 실행하지 않는다.

SDK gate의 실제 원 metrics1/state_machine9 raw 통과를 PM가 읽었다. 이는 고정21 중10이며 21 전체 성공은 아직 아니다. SDK integration 빌드는 원runner110148/child83192에서 진행 관측했다. Windows/fontique/parley/blitz compile 관측을 계기로 연구 담당에게 SDK 기능 검증의 필요한 typedNative 의존과 UI 조립 feature 결합을 읽기 조사하도록 배정했다. 신규 분리 구현/feature 변경·coverage 축소 권한은 없으며 현재 gate를 중단하거나 재시작하지 않는다.

후속 PM raw에서 metrics1+state9+SDK exact11의 실제 고유21 통과를 확인했다(신규6 포함). 제품 default/native 및 owned strict·fmt는 원receipt0이다. 실제 Native feature명은 native-proposal-window다. SDK no-run은251502ms/exit0, 최종integration binary는 DEF882AA42D6336D2D2DC3D49EEDC553AC832E45266F64F0D6DBBEC88B9D3D0F다. 정상CLI explicit build는 별도 원runner107140/child23444로 진행 중이며 help/실ready/미디어 성공은 아직 아니다.

## 현재 편집 범위

- **AI**: `runtime/verification/ai-vad-gate-r1/source`(S)에 source229+조립4 사본233, 원 actual manifest `A880556453A9174B30CF77D90ADC6FCE42FFE55566013D975BFFD5D2E7ED28AA`. 원 source13 `388DE737…C5637D`와 r2를 보존한다. tool lock은 Cargo가 local owned-process entry/의존1만 추가했고 S root lock은 별도 신규 산출물이다. Windows filtered metadata·두 strict/fmt·순수4/invalid1/doc2는 실제 통과, 정상 BIN은 진행 중이다. 실추론은 별도 배정한다.
- **SDK**: `runtime/development/native-ready-record-observation-r2` source6는 supervisor·신규 supervisor/record_observation.rs·state_machine·기존 jobs/owned_process helper·native_skip_cli top 모듈·신규 ready_observation.rs다. 원 parent4 `BB041ED9…E1C3`는 불변이며 독립 읽기로 인수했다. 기본 Clock/cutoff/offset/child poll/오류-owner를 유지하고 loop I/O0·overflow Unknown·반환 뒤 bounded summary1회다. ready-only ignored1은 원 host 준비/즉시 EOF만 계획했고 실행0이다.
- **SDK 별도 핀**: validation9 r2 `11A4D104D939CD4DD812616D329C5AE7504C48CA70DA08C358B36B6FD8DC1241`는 원371E86 대비 채택된 진단71줄만 추가/원8 불변으로 읽기 인수했다. 응답손실 실제 복구는 미검증이다. parent6와 무단 합성하지 않는다.
- **UI8**: `src/assembly/meari/` 아래 mod.rs, 신규 viewer_native_probe.rs, player_shell_host.rs, quarantine_surface/frame_delivery/native_backend.rs, viewer_host/mod.rs, viewer_native_launch/no_media/tests.rs, 신규 tests/end_to_end.rs 및 tests/end_to_end/supervision.rs. 전부 test-only 관측/시험 조립이다. 실제 public prepare/run·proof Observed·owned Completion·원 guards/close intent/wake·child0→inner close 순서를 검증하도록 작성한다. 시작 전 fullstamp 변경 거절은 별도 사례다. 관측 오류도 원 cleanup을 생략하지 않는다.
- **Meari Q09**: `runtime/development/meari-package-boundary-r1`, 최종15 `5A31ADDBC22D6AA08DDDE2A73E35B625A2BEC848440746857B6FA373297E107F` 읽기 인수. manifest3/builtin+localfixture2/원 법적동봉9/Rust시험1. publish=false·원 version·법적 원문을 유지한다. 일반2/Native1/ignored 준비·판독2와 실제 Cargo목록3 계획은 아직 미실행. tarball/verify/독립 packaged tests·전체 법률 검토는 잔여다.
- **Meari Q02 CP01**: 별도22fe0d 기반 overlay의 새 domain/hook_value.rs, 새 application/hook_values.rs, 기존 domain.rs/application.rs 연결, 새 application/hook_values/tests.rs 정확5. 최소 Rust owner/opaque값·명시 same_value·예산/수명/transaction API다. 기존 StateValue Eq/serde·HookSlot/queue/HMR/frontend/lib.rs를 바꾸지 않는다. React hook 연결은 CP02 필수 잔여이며 모델 회귀를 hook 지원으로 세지 않는다.

범위 밖 파일·feature·새 API 권한이 필수면 실제 서명/소유권 근거로 추가 배정을 받는다. 작성 중 source를 불변 최종 핀으로 취급하지 않는다.

## 자원과 공유 보호

[하드웨어 배분](ai/resource-allocation-2026-10-21.md)을 따른다. 원 runner SHA `EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D`, conservative2/affinity3/2논리CPU/BelowNormal/-j1/thread1을 유지한다. 새 작업 진입 기준은 가용 RAM16GiB·VRAM3GiB·C12GiB다. 사용자 ChatGPT/Terminal·다른 앱/프로세스·WSL/VM을 종료하지 않는다.

AI는 D 기존 tool target·고정 static ORT library·실행별 offline 환경을 사용한다. 새 C cache/모델 다운로드/전역 환경/RUSTFLAGS 변경0. SDK/UI는 기존 public-p3-parent-r1/native cache를 사용한다. 공유 UI snapshot은 SDK 진단 반환 기준1414에 integrated1 마지막 우선이다. AI S와 섞지 않는다.

root·shared snapshot·Meari412와 기존 문서2·원 before/새 파일 부재·확정 source/bundle/binary/입력·index를 보호한다. 활성 독립 overlay는 원본 manifest와 최종 핀을 구별하며 작성 중 변경을 공유 오염으로 판정하거나 되돌리지 않는다. runtime/cache 전체 재귀 검색·전체 소스/모델 복사를 하지 않는다.

## 다음 검증과 종료 기준

1. AI build-only의 정상 BIN/provenance·원 receipt/handle·보호/신원 대조를 마친 뒤 즉시 슬롯을 반환한다. 정상1/취소1 추론은 새 배정에서 단일 worker로 수행한다. 작성7/실제7과 추론2를 합쳐 추론9로 세지 않는다.
2. SDK parent6는 양 strict/fmt·state_machine 등 고정 순수21의 실제 고유명·최종 artifact를 먼저 인수한다. 실제 ready-only1은 이후 정상 CLI/동일 source pin을 고정한 별도 배정이다. 실제 media 재시도/child milestone/timeout 증가는 현재 권한이 아니다.
3. UI8·Meari Q09/CP01은 source 인수 뒤 별도 단일 슬롯에서 검증한다. OS/시각/DPI/영상/React frontend 지원을 순수 회귀로 대체하지 않는다.

각 원 exec가 terminal인지와 receipt를 회수하고 동일 핸들을 관측한다. timeout만으로 재시작하지 않는다. 원 오류·forced/미관측 child exit·Incomplete owner를 보존한다. PID 부재만으로 정상 종료를 만들지 않는다. 원 family/worker·종료 신원·source/비입력/mtime·binary·index 대조 후 문서보다 먼저 heavy/shared를 명시 반환한다.

현재 검증된 root 단계는 UI V2b61 `736de48`와 단독 mpv 읽기 진단2 `9ffd855`다. 읽기315784us는 원 ready 실패 원인 확정이 아니다. 실제 세 카테고리→provider→native 재생/ACK/landing·실제 Viewer·제품 AI 연결/사람 품질 승인은 미완료다. Pages·VM·DPI·새 장시간 시험·수동 GUI 검토 보류는 유지한다.
