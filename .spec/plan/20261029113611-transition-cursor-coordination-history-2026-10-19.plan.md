---
id: plan:kinekino/20261029113611
title: 회차 전환·커서 조율 이력
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 회차 전환·커서 조율 이력

현재 상태는 [중앙 조율](current-coordination.md)을 따른다. 아래는 재배치 시점의 누적 관측 원문이며 최신 슬롯 상태를 대체하지 않는다.

# 현재 세션 조율 상태

2026-10-19 PM이 원로그·실행 receipt·영상 프레임을 직접 대조한 현재 상태다. 전체 제품 완료나 모델 품질 승인은 아니다. 과거 실행은 [옵션 조율 이력](native-options-coordination-history-2026-10-19.md), [앞선 체크포인트](session-checkpoints-2026-10-19.md), [전체 조율 이력](session-coordination.md)을 따른다.

## 담당과 현재 배정

| 담당 | 확인된 완료 | 현재·다음 작업 |
| --- | --- | --- |
| PM | 실제 native·옵션 인수와 소유 프로세스 종료 직접 확인 | 실행 슬롯·소스 동결·의존성·주요 완료 통합 보고 |
| SDK | 회차 전환·커서 실제 인수 통과, 커서 전체330pass/18ignored·strict/fmt·소유 종료 | heavy 명시 반환. root 등록 입력 유지하며 audio 공통계약 수리 미등록 저자원 준비 |
| Meari | Repeat 고정 Core56/Native20·실제 IAB28·strict/fmt 및 서버 정리 | 단일 heavy 배정. Native host port 고정 입력 type/focused·필수 회귀/strict 검증 |
| UI | 실제 R3·원본 Cards/Highlights 포함 전체27pass/0fail/0ignored·strict/fmt | factory/panel·부분 생성 복구·정리 근거/Busy retry 소스 준비(미실행), 커서/audio 계약 대조 |
| AI | historical import 일반12+부정 인수1·고정 CLI 및 제품 연결 차이 검토 | provenance·fresh CLI·최종 사람 품질 검토 준비. 새 추론/다운로드 보류 |

서로 다른 checkpoint 수치를 합산해 완성률로 표시하지 않는다. 수정 소스에 이전 통과를 소급하지 않는다.

## 최신 실제 인수

SDK 커서 최종 정본·source-final/pins 및 heavy/root 소비 freeze 명시 반환을 인수했다. PM도 owned-processes-after[]와 exactPID117504/111512/78888 부재를 확인했다. Meari에 Native host port 입력 고정→window feature type/focused4·기존 회귀→최소수리→필수fmt/test/strict·공개API 인계를 단일 슬롯으로 배정했다. SDK는 root 등록 compile 입력을 다음 UI 소비까지 유지하고 audio 공통계약을 미등록 소스로 준비한다. UI는 새factory/panel 게이트 준비를 계속하며 중복 heavy를 시작하지 않는다.

pointer actual-first는94311ms/exit0·1pass/0fail/0ignored/15filtered(본문94.26초)로 PM이 원로그/receipt를 대조했다. native-pointer-observations에서 실제 owned video pid78888·1920×1080의 캡션→진입→1500ms 대기→이동→아래 가장자리 모두 showing/handle/owned HWND true이며 원좌표 복원 표본도 true였다. EOF engine4/IPC/stdout/stderr·browser 종료true/jobs2/pendingnull·diagnosticcomplete도 직접 읽었다. standalone native 커서 수정 범위는 확인했으며 제품 fullscreen/overlay·청취·audio handoff 미해결과 별개다. 최종 pins·소유PID 부재·정본/명시 반환을 담당이 마무리한다.

pointer 고정 실행 준비에서 bins-final3113ms/0·harness-final6959ms/0·strict-final1692ms/0·fmt-gate10170ms/0을 PM이 receipt로 대조했다. helper는 창 캡션으로 나간 뒤 실제 영상 진입을 관측하도록 보강됐으며 기존 전체330 통과와 이후 helper 변경의 경계를 보존한다. 실제 커서 관측과 정리·최종 인수 전 해결로 표시하지 않는다. 배포 권장안은 mpv 동봉·고정 공급 manifest→Rust adapter 분리→libmpv 별도 검증으로 mpv-package-boundary.md에 제안 상태로 기록했다.

pointer full-first는237518ms/0·330pass/0fail/18ignored를 PM이 전체 로그로 대조했다. Meari Native host port는 독립 source/얇은4시험 등록·구문fmt까지 준비됐고 compile/OS검증0이다. 커서 실제 인수·핀·정리 반환 뒤 Meari type/focused/전체검사, 그 뒤 UI 신규factory/panel 소비를 준비한다. SDK audio 공통계약 수리는 미해결 결함으로 유지하고 미소비 입력 경계를 확인해 저자원 소스 준비와 검증 슬롯을 나눠 진행할 수 있다. 기존에 시작된 실행은 중단하지 않는다.

커서 새 checkpoint native-pointer-cli-r1/check-first는12114ms/exit0로 PM이 receipt/로그를 직접 확인했다. 공통 pointer_policy의 input-cursor=yes/cursor-autohide=no를 양 mpv launch가 소비하고 Rust Win32 포인터 관측 helper가 작성됐다. 아직 실제 커서 통과는 아니다. UI는 recoverable factory cleanup의 근거 enum과 pending ticket cancel→terminal 뒤 retry 소스 검사를 보강했으며 새 전체 실행 전이다. legacy 강제 mute 결함과 CLI volume0은 별도로 추적한다.

무음 설명 정정: SDK 소스 감사에서 legacy mpv/mod.rs의 고정 --mute=yes를 발견했다. CLI actual의 명시 volume0과 legacy GUI 강제 mute는 다른 원인이다. GUI state 기본volume80만으로 실제 소리가 난다고 가정하지 않으며 PM이 사용자에게 정정했다. 강제 mute도 결함으로 등록해 커서 수리 다음 순차 수정·명시 volume/mute 일치·검증을 SDK에 배정하고 UI에 기존 상태/저장 계약 대조를 요청했다. 설정 관측을 실제 청취 품질 성공으로 확대하지 않는다.

PM이 actual-r2의1pass/0fail/15filtered·103036ms/0, Next old1 종료→new2 및 Previous old2 종료→new3 commit receipt, EOF engine4/모든worker·browser 종료true/jobs2/pendingnull과 owned-processes-after-r2[]를 직접 읽었다. Previous PNG 실물을 열어 불타는 성의 실제 프레임을 확인했다. 고정 실행 인자의 input-cursor=no가 local/native 양쪽에 있고 실제 mpv list-options는 input-cursor 기본yes·cursor-autohide 기본1000ms로 관측됐다. 이것만으로 사용자 커서 결함의 원인/해결을 확정하지 않고 담당이 양쪽 경로 수리·실제 복귀 확인을 별도 checkpoint로 진행한다.

사용자가 영상 위 커서 동작을 명시적으로 수정 대상으로 확정했다. 정상 자동숨김으로 종결하지 않고 SDK의 실제 mpv launch/adapter 진단·수리와 UI의 pointerenter/move/controls 복귀·전체화면 유휴숨김 정책을 책임별 인계했다. 실제 r2 재생 성공과 커서 결함은 별도 checkpoint다. 원증거를 보존하고 중복 heavy 없이 순차 수리·필요 검사·최소 실제 커서 복귀 검증까지 이어간다. 검증 volume0 무음과 실제 청취 미검증은 별도이며 제품 기본 무음을 확정하지 않는다.

first 실제 실패의 마지막 play 관측은 instance3/pausedfalse/idlefalse/connectedtrue/errorfalse이나 seek 뒤10000000us 고정이었다. 담당은 seek 전 Cache2.5초/AV3초, 이후 Cache0/AV10초를 진단에서 확인했고 네트워크 원인은 미확정이다. PM은 failure-stop의 engine/IPC/stdout/stderr 종료true·diagnosticcomplete/무손실·browserworkertrue/jobscompleted2/pendingnull 및 forcedfalse/waittrue를 직접 읽었다. owner는 정확 소유PID 부재를 owned-processes-after-first.json에 보존했다. 같은 고정 pins·조건·기한으로 fresh actual-r2 재관측해 일회성과 지속 회귀를 구별하며 원인 확정/수리 성공으로 앞서 기록하지 않는다.

actual-first는104559ms/exit101·0pass/1fail/0ignored/15filtered로 종료했다. PM이 native_episode.rs186의 `Native application not observed for play` 원문을 확인했다. Next/Previous resolve/commit·새 영상 관측/PNG 부분 진행 보고와 전체 실제 gate 실패를 분리한다. 담당에게 control request/ack·직전 상태·instance/source·poll을 대조한 원인 진단, 소유 cleanup 증거 확인 뒤 필요 최소 수리와 새 고정 입력 재검증을 지시했다. 기한·완료조건·전체 PNG 검증을 임의 완화하지 않고 첫 실패 원증거를 유지한다. 단일 heavy 슬롯은 SDK가 계속 소유한다.

actual-first의 runner111248/고정 harness113296 실존과 terminal=false를 PM이 확인했다. exact case는 native_episode::actual_resolve_and_owned_native_lifetime이며 fresh job/profile·동일 combined host Next/Previous 인수다. 고정 CLI49a0404263a0b83396891a2316362da333aca40605e8bc78a789b8dd345d5e36·harness9a242b118431c37889f2098caed0c400335d035bd53df39dc83b6ade8f9c8732를 사용한다. 시작은 통과가 아니다. UI는 panel2+factory5·recoverable session 소스 준비와 per-file fmt까지만 완료했고 신규7개는 미실행이다. 새 factory의 부분 cleanup owner/원인·Busy retry·stale 검사는 다음 별도 gate로 인수한다.

SDK full-final은268717ms/exit0/terminaltrue이고 전체 suite 합계329pass/0fail/18ignored를 PM이 직접 대조했다. fmt-gate3898ms/0·jobs-harness-final11187ms/0도 확인했다. 고정 bin/harness·새 job/profile·실제 양방향 회차 검증은 다음 gate다. full-final 뒤 방향별 진단 문구/서식만 바뀐 경우에도 그 차이와 최종 source/harness 경계를 명시하도록 인계했다. 실제 재생 성공이나 전체 제품 완료를 아직 주장하지 않는다.

교체 fault r3는22949ms/0·10pass/0fail/0ignored(직접 실패9+bounded child probe1), strict-first23582ms/0·fmt1138ms/0을 PM이 receipt/원로그로 대조했다. full-final은 launcher81000/child70156가 실존하는 진행 실행이다. 최초 fault 실패는 실제 exit101이므로 SOT exit1 표기를 owner에게 정정 요청했다. UI factory 준비에서 기존 session constructor가 partial cleanup 실패를 잃는 gap을 발견해, root 미등록 UI 소유 document_session의 recoverable constructor·오류/부분소유 보존·명시 retry 소스 수리를 허용했다. permanent unresolved 표시만으로 기능 완료 처리하지 않으며 새 cargo/공유 SDK 변경 없이 별도 미검증 checkpoint로 준비한다.

replacement fault 최초 실행은6972ms/101로 ReplacedEpisode의 Debug 부족에 따른 테스트 컴파일 실패를 PM이 읽었다. 최소 수리 뒤 r2는28396ms/0·8pass/0fail/0ignored·227filtered였다. 자식 프로세스 probe1개와 직접 실패 분기7개이며 EOF 새 cleanup 오류의 code/details 보존과 재시도 소유를 포함한다. 실제 mpv 미디어 재생과 구별하고 이후 형식·주석 변경까지 반영한 최종 전체 입력 검사는 별도 gate다.

SDK full-first는278828ms/exit0/terminaltrue이며 PM이 전체 suite 로그에서319pass/0fail/18ignored와 owned_startup worker failure/recovery2·미시작 stream/관측EOF 구분·diagnostic complete 조건 검사의 ok를 확인했다. 이 실행은 replacement 전체 fault·실제 Next/Previous 성공을 대신하지 않는다. SDK는 old-close/new-launch/EOF cleanup 직접 fault 보강 뒤 최종 전체검사·strict/fmt/bin·실제 CLI 인수를 이어간다. 제품 host의 UI 대조에서 동기 reproject commit 대 Effects 후속commit, Closing권한 대 close_callbacks, batch take뒤 작은queue Capacity의 잔여소유 세 gap을 Meari에 인계했다. 독립 소스 준비는 계속하되 새 heavy는 SDK만 수행한다.

UI 최종 정본 original-review-sdk-2026-10-19.md와 명시 heavy/rootSDK/Meari/consumer freeze 반환을 인수했다. PM은 fmt288ms/0·input-comparison shared0/manifest0·소유생존[]와 Cards restored 실제 Unicode/원raw/refs2/None시간/doc3·receipt2·output3·automaticfalse를 직접 확인했다. SDK에 root 등록 동결해제와 다음 단일 heavy 슬롯을 배정해 준비20파일의 현재 baseline 대조→책임별 등록/type/focused12·fault→전체 검사/bin→동일 combined host 실제 Next/Previous 인접2회차·기존옵션·종료 검증까지 진행하도록 인계했다. UI/Meari는 제품 factory·공개 host port 계약의 저자원 준비를 이어가며 추가 heavy는 시작하지 않는다. 최종 제품 host/fresh CLI/project/provenance·사람 품질 승인은 미완료다.

UI 원본 Cards/Highlights 실제 SDK 선택2개는 native82923ms/0·2pass/0fail/0ignored·본문1.13초로 PM이 원로그를 대조했다. 이어 full-test122962ms/0의 --no-fail-fast 전체11개 suite는 합계27pass/0fail/0ignored로 확인했다. 기존 실제 R3 NeedsRepair·SDK/원본 intake를 포함하며 이전 map 실패가 같은 원문에서 해소됐다. 첫 선택/전체의 compile·기능 수리는0이다. strict Clippy12107ms/0도 receipt에서 확인했고 fmt·최종 pins·정본·소유 종료 및 명시 반환은 담당이 마무리한다. 실제 제품 host/fresh CLI/project/provenance·사람 품질 승인은 여전히 별도 잔여다.

UI 실제 원본 SDK2개 gate가 runtime/quality-review/player-ui-original-review-sdk-r1/gate-r1/native.receipt.json에 시작됐다. 최초 terminal=false·launcher102096/child68780의 실존을 PM이 Get-Process로 확인했다. 실제 SDK api/mod.rs pin55216af45998dc09adfee69e7da97942bcc7643732b87a9e0ac6b3e46c36e4f9와 Meari source-after10ec036…가 인계값과 일치한다. 옛 api.rs pin 경고는 api-mod-corrected-pin.json으로 정정하고 전체 src inventory에는 처음부터 실제 mod.rs가 포함됐음을 담당이 확인했다. Meari browser/source_callbacks.rs 최신 marker는 UI 시작 전 변경이며 r11/browser-test-freeze-audit.json에 snapshot항목/currentSHA 일치를 보존했다. 진행 실행의 재시작·공유입력 수리는 하지 않는다.

Meari Repeat checkpoint는 실제 IAB28pass/0fail/0ignored/0filtered·0.58초까지 인수했다. PM은 result.dom.txt·console warning1/error0·cleanup remaining[]/port6596 listen0/tab_closed와 exactPID109316/113300 부재를 직접 대조했다. interactive server.receipt terminal=false/Ctrl-C exec1은 그대로 보존하며 테스트 실패로 바꾸거나 종료0으로 덮지 않는다. 담당의 heavy 명시 반환·전체 src 등록/미등록 입력 동결을 인수하고 SDK 명시 root freeze와 함께 UI에 다음 단일 슬롯을 배정했다. UI는 original_review_sdk 선택 인수→전체 --no-fail-fast→strict/fmt를 진행한다. 최종 Meari source-before/after pin10ec036aae146ae7ee4fe2b409e0f9d9b04f9eb680b4fa69bfdd25ff20dac3d5, lock 불변 보고를 따르며 workspace strict23258ms/0은 r10 입력(이후 WASM test unused_mut 최소수리)을 구분한다. OS 창/전체 접근성·혼합 slots·복합key·전체Q01–Q09·제품 host pump/GL은 이 인수 범위 밖의 잔여다.

UI는 tests/original_review_sdk.rs에 실제 PlayerSdk·ProposalFiles 기반 Cards/Highlights top-level2개를 작성했고 PM이 imports·저장소 위임·Timeout 주입·Stale/자동false 검사와 두 test 진입점을 직접 읽었다. 아직 미컴파일/미실행이며 실제 통과 수는 확정하지 않는다. 원본 소비 계획의 준비 상태를 갱신했고 신규 test도 다음 입력 동결 대상이다. Meari r11 browser-build는44777ms/0으로 종료한 것을 PM이 대조했으며 담당이 IAB 인수 준비를 진행한다. build 성공은 Web 동작 성공이 아니다. SDK에 UI 소비 기간 root 등록 compile input 동결의 명시 확인을 요청했다.

r11 Native는 terminal=true·108872ms/0이며5개 suite 합계20pass/0fail/0ignored를 PM이 원로그로 확인했다. 같은 checkpoint WASM strict5297ms/0·fmt1446ms/0도 직접 대조했다. browser-build는 최초 확인 terminal=false로 실제 Web 성공 전이며 담당이 기존 프로세스로 계속 실행한다. SDK startup/diagnostic 후보는 미시작 collector와 EOF 관측을 구분하는 focused12개를 포함한 준비20파일로 보고받았으나 아직 등록·실행하지 않았다. UI 다음 슬롯과 root 동결은 유지한다.

고정 r11 Core는 core.receipt.json의 terminal=true·exit0·84707ms와 stdout 전체11개 suite 합계56pass/0fail/0ignored를 PM이 직접 대조했다. 중첩 template의 outer capture/inner shadowing과 생성 Rust 실행을 포함한다. 같은 입력 Native·WASM strict·실제 Web은 아직 별도 gate다. UI 원본 소비 계획(original-review-sdk-gate.md)의 기술 fixture·기존 SDK 재사용·전체 --no-fail-fast·동결 범위를 검토했고, 슬롯 대기 중 UI 소유 top-level 실제 SDK fixture/typed props 조립을 계속하도록 배정했다. SDK에는 root green 등록 입력 유지와 미등록 후속 준비를 재확인했다.

Repeat 후속에서 PM은 r10 WASM strict의 browser/source_callbacks.rs368 unused_mut 단일 실패를 원로그로 확인했다. 담당은 최소 수리 후 r11 입력 snapshot을 고정하고 Core 검증을 실행 중이라고 보고했다. 앞선 r4 Core16·44379ms/0과 Native20·128819ms/0, r10 workspace strict22870ms/0은 해당 입력의 결과이며 이후 namespace·중첩 lexical capture 및 새 Web 테스트 변경의 성공으로 소급하지 않는다. Web 원본 마운트·동적 key/상태/포커스·삭제 이벤트·실패 시 기존 화면 유지 테스트는 작성 상태와 실제 브라우저 실행을 구별한다. 최종 Core/Native/WASM·실제 Web·fmt·pins·소유 프로세스 정리와 명시 반환 전까지 heavy 슬롯은 Meari가 유지한다. UI에는 다음 소비 인수·동결 범위의 저자원 준비를 재전달했다.

details/summary Native 어댑터 수리 뒤 같은 원본 마운트는 keyed-repeat-r2/native-original62144ms/0·1pass0.47초로 통과했다. PM이 로그를 직접 확인했다. 두 refs·외부 Actions 콜백과 disclosure 인수이며 동적 재정렬/listener·실제 Web·최종 전체 strict를 대신하지 않는다.

원본 Native 마운트 첫 gate는 keyed-repeat-r1/native-original에서126867ms/101·1failed로 종료했다. PM이 source_callbacks.rs273의 `native widget implementation unavailable: details` 원문을 대조했다. 파서 성공과 구별하며 Meari가 원본 details/summary를 보존한 Native 어댑터·토글·키보드·접근성 경로를 연결한다. 제품 태그 삭제/flatten으로 우회하지 않는다.

Repeat 파서 checkpoint에서 Core27pass/0fail/0ignored·75061ms/0과 동일 원본 Cards/Highlights 각3모듈 positive1pass·9069ms/0을 PM이 receipt와 결과로 대조했다. 기존 map538..609 파서 차단은 해소됐다. 이 단계는 원본 파서/IR 성공이며 dynamic DOM/listener·Native/Web 전체 지원은 아직 미완료다. 원본 Native 마운트·두 refs·외부 Actions 콜백 인수는 별도 실행 중이다.

Repeat 소스 첫 workspace all-targets check는 keyed-repeat-2026-10-19-r1/check.receipt.json의40621ms/0으로 PM이 대조했다. 원본 유지 Repeat/Local IR·TSX map/key/span·Rust 생성/재파싱·bound projection과 key marker가 등록됐고 Core 시험을 시작했다. Native/Web 동적 DOM·listener 재결합과 원본 실제 인수는 아직 미완료다. 타입 검사 성공을 반복 화면 지원 완료로 표시하지 않는다.

Array kernel은 Core47/Native18·workspace strict30973ms/0·WASM strict14501ms/0·최종fmt933ms/0으로 인수했다. 최초fmt943ms/1의 한 줄 형식 차이만 수리했으며 원diff를 보존했다. [실행 정본](../../../packages/meari/.spec/evidence/verification/array-kernel-2026-10-19.md)의 source-after pin3a2295a3c7d478842314b1ecb7700f9bc01e0d632784ccfc41694fcbb375361e를 따르며 원본 map positive는11460ms/101 실패다. PM이 두 graph의 proposal-source.tsx538..609 진단을 직접 읽었다.

Meari의 public 등록 입력 동결·heavy 반환과 SDK의 옵션r3 root 입력 동결을 명시 인수한 뒤 UI에 단일 heavy 슬롯을 배정했다. UI는 실제 R3 document_repair_sdk와 회귀·strict/fmt를 수행한다. 원본 map 미지원의 전체 실패와 scalar repair 성공은 구별하며 ignore/flatten하지 않는다.

[SDK 옵션 정본](../../sots/verification/application/native-options-cli-2026-10-19.md)의 r3는 fmt3149ms/0·strict18246ms/0·전체234247ms/0·bins1493ms/0·actual39963ms/0(1pass38.96초)이다. 실제1.25배속 진행→1.0 복원, audio/video 끄기·복원, 잘못된 종류/ID와 이전 catalog/instance 거부, 기존4controls·close/reopen·EOF를 통과했다. 자막 트랙은 없었으므로 양성 선택 성공으로 기록하지 않는다.

PM은 복원PNG1920×1080/7377161B/SHA ed874ad3642a4d19dfda134bad6d6f2e1f1df0f4725048fc87d2c75bd8d95922 실물을 열어 영상 장면을 확인했다. close1/EOF2의 engine/IPC/stdout/stderr 종료 true·diagnostic complete와 소유 프로세스 부재를 확인했다. r1/r2의4초 게시 실패와 원staging은 보존했다. compression1/filter0은 원픽셀·전체PNG/CRC검증·동기화·덮어쓰기 방지·기한을 유지한다. 필터 단독 가속량은 미측정이다.

선택 회차 인수이며 실제 청취·전체 카테고리/회차·장시간·정확PTS/색상·전체GUI·사람 품질 승인은 별도다. 영상에 포함된 번인 문자와 플레이어 자막 합성 여부를 구별한다.

## 다음 소비 순서

SDK 실제 회차전환의 최종 반환 뒤에는 Meari Native host port의 타입/얇은 소비자·실제 창 수직 검증, UI 신규 panel/factory·recoverable constructor 인수, root 제품 등록/fresh CLI/project/provenance 통합 순서를 준비한다. 아직 후속 슬롯 배정 완료를 뜻하지 않으며 담당의 실제 소스 준비와 공유입력 영향을 대조해 확정한다. Meari native_host/mod.rs·types.rs와 UI factory/constructor 소스는 이전 통과에 포함되지 않는 미검증 변경이다.

현재는 Meari Repeat 및 UI 원본 Card/Highlight SDK 소비·명시 반환을 완료하고 SDK 회차전환 준비20파일의 등록/type·focused12·전체 검사·고정 bin·실제 양방향 인접2회차 단계로 넘어갔다. SDK가 단일 heavy 슬롯을 소유하며 UI/Meari는 제품 factory/공개 host port를 문서로 준비한다. 제품 등록·fresh CLI/project/provenance 잔여 계약은 이후 인계에서 대조한다. 아래 앞선 시작/실행 문단은 해당 시점의 이력이며 최신 슬롯 배정을 대체하지 않는다.

최신 UI 인수는 [실제 R3 문서 수동 수정 정본](../../sots/verification/presentation/document-repair-sdk-2026-10-19.md)으로 확정했다. PM은 최종 정본·영수증·소스 경계·shared/root 전후pins차이0 보고와 heavy/freeze 명시 반환을 인수했다. Meari public/src 동결을 해제하고 Repeat IR→양 frontend→공통 projection→Native/Web 구조 갱신·key identity·기존 callback 안전 재결합과 동일 원본2graph 지원에 다음 단일 슬롯을 배정했다. SDK 회차 전환은 미등록 저자원 준비로 유지한다. 아래 앞선 UI 시작/실행 기록은 이력이다.

UI 소비 결과는 실제 R3 포함24pass와 원본 map intake1fail/0ignored다. Result helper 최소수리 뒤 full102339ms/101에서20pass 후 Card/Highlight의 proposal-source.tsx538..609 실패에 도달했다. failfast로 누락된 sdk_files4개를2616ms/0으로 별도 통과했고 strict29664ms/0·fmt246ms/0도 PM이 receipt와 로그로 확인했다. 전체 green이 아니며 최종 정본·pins·명시 반환을 인수한 뒤 Repeat 본체로 교대한다.

UI 실제 R3 Native1은142641ms/0·실행0.89초로 통과했다. PM은 선택 DocumentTextRef·restored.json의 Unicode·raw/import_error 보존·수정revision2→채택output3·automatic_application_enabled=false를 대조했다. 첫 full은19332ms/101의 E0599로 원본 map 실행 전 컴파일 실패했다. support/typed_review_case.rs의 module_projection Result에 ok_or를 호출한 한 줄을 최소수리해 retry-result/full-test를 실행 중이며 아직 전체 성공이 아니다. source 트리의 미등록 keyed_reconciliation 추가는 실소비 등록 입력과 구별해 인계했고 이후 Meari/src 전체 추가 변경도 고정했다.

현재 UI 소비 실행은 runtime/quality-review/player-ui-document-repair-r1/gate-r1/native.receipt.json에 기록한다. snapshot/owned-pins-before/shared-pins-before를 보존했고 최초 PM 조회 launcher86200/child26956·terminal=false다. SDK/Meari 등록 입력 동결을 유지하며 새 GUI/IAB 없이 실제 Native 문서·공통 SDK를 검증한다. 시작은 통과가 아니다.

1. SDK 회차 전환을 책임별 제품 모듈로 등록하고 실제 CLI의 Next/Previous·소유 종료·기존 옵션 회귀까지 검증한다. 현재 준비 코드의 타입/기능 성공은 아직 미확정이다.
2. 제품 factory와 Meari 공개 host port를 같은 SDK/registry/문서 수명에 맞춰 통합하고 root 등록·실제 창 연결을 검증한다. 앞선 headless SDK27개 성공을 제품 창 완료로 확대하지 않는다.
3. fresh CLI/project/provenance 연결과 누락/손상 오류 처리를 인수한 뒤 모델 품질 검토를 준비한다. GUI 전체 시각 인수·모델 품질 승인·나머지 전체 기능 범위는 별도 잔여로 유지한다.

UI 소비는 tools/player-ui-sdk-bridge·제품 plugin·presentation adapter/TSX·assembly·root SDK와 Meari path 입력을 고정해야 한다. 현재 root workspace는 Meari를 exclude하고 web_ui_adapter를 등록하지 않으므로 SDK 빌드와 미등록 UI 준비를 구별한다. 복사된 snapshot 전체의 전기간 불변을 주장하지 않는다.

## 기능별 정본·잔여 계약

SDK가 replacement 실제 경로에 private runtime seam과 직접 fault 테스트를 추가하는 소스를 PM이 읽었다. unsupported preflight/old close diagnostic/new launch/startup recovery/EOF cleanup 등 선언은 작성 상태이며 아직 실행 성공이 아니다. EOF 뒤 next.close Err 분기가 owner는 보존하지만 원 cleanup ApiError를 버리는 것을 확인해, 원 code/details 보존과 해당 fault/retry assertion을 담당에게 최소 보강 요청했다. 제품 구현은 SDK만 수정하며 최종 실행 receipt로 확정한다.

PM이 current root Cargo members/exclude 및 meari 직접의존 부재를 다시 확인했다. UI27 소비 반환 뒤에도 Meari 전체 소스를 동결할 필요는 없어 독립 packages/meari의 Native host port1~3·얇은 소비자 Rust 테스트 소스 작업을 해제했다. root/SDK/UI 공유입력·의존/lock 변경과 새 cargo/GUI/브라우저는 허용하지 않으며 단일 heavy는 SDK가 유지한다. r11/UI27 고정 snapshot·pins·원증거는 보존하고 새 소스는 별도 미검증 checkpoint다. UI도 새 panel preparation과 실제 공개 서명만 맞추며 제품 등록은 후속 통합 단계로 유지한다.

읽기 전용 연구로 replacement의 preflight/old-close failure/new-launch failure/launch 중 EOF/new cleanup failure 분기는 구현과 직접 fault 시험이 별개임을 확인해 SDK에 인계했다. owned_startup의 두 선언은 초기화층 시험이며 교체 전체 흐름·retain_startup_failure의 active 소유 연결 성공으로 확대하지 않는다. full-first 기존 실행을 유지하고 종료 뒤 누락된 직접 fault 검증을 같은 예약 checkpoint 안에서 보강한다. 연구는 파일수정/빌드/브라우저0이었다. focused-first12 통과도 신규 transition8+기존4로 나눠 기록하며 초기화/EOF 시험 전체 통과로 오인하지 않는다.

SDK check-first는33933ms/exit0/terminaltrue로 PM이 receipt를 직접 확인했다. 전체 target 타입 검사 성공이며 focused·실제 회차 재생 성공과 구별한다. focused-first도39252ms/exit0이지만 정확한 실행 테스트 수는 전체 로그에서 확정한다.

SDK 등록 뒤 최초 fmt-first-receipt는1977ms/exit1이며 runner/client/mod 등의 형식 diff를 PM이 원문으로 확인했다. 타입/기능 실패로 단정하지 않고 최소 format 수리·다음 type/focused gate를 담당이 진행한다. UI는 Meari host 문서4개를 읽고 shared/proposal-window-host-port.md에 차이표를 인계했다. PM은 Meari에 close 허용/정리 pending·실패/강제OS destroy의 typed outcome과 각 cleanup 단계 retry/receipt를 구체화하도록 요청했다. 현재 후보 서명·전체 accessor·얇은 소비자·OS 검증은 미구현이며 이 계약 준비가 새 SDK mutation이나 제품 창 성공을 뜻하지 않는다.

SDK 회차 전환 registration.patch는 git apply --check 후 적용한 완료 marker를 PM이 확인했다. 타입/실제 기능 통과는 아직 미확정이다. 다음 제품 host 계약 준비는 UI의 shared/proposal-window-host-port.md와 Meari packages/meari/.spec/sots/integration/native-host-port.md·plans/native-host-port.md를 상호 인계했다. 후보는 !Send UI-thread 문서 owner·기존 winit driver·guard 밖 after-dispatch batch이며 기존 downcast 경로 전체를 같은 owner accessor로 연결해야 한다. 단순 Rc wrapper·eframe GL 공유를 이미 지원하는 것으로 가정하지 않는다. 현재 공개 API 후보 구현0, 새 SDK mutation 없이 기존 API를 재사용하는 계약 준비이며 SDK 실제 전환 gate의 범위를 바꾸지 않는다.

제품 등록의 실제 차이는 [조립 인계](presentation/proposal-product-registration.md) 상단을 따른다. 현재 제품 host는 eframe Glow이며 독립 Native 문서 검증은 default-features=false다. 실제 창/event-loop와 Native 문서 callback host 연결, assembly·presentation 부모 등록·root 의존, 도구 전용 manifest/context의 제품 공통 책임 이관은 아직 미구현이다. UI 인수 통과를 player.exe 등록 완료로 확대하지 않는다. SDK가 저자원으로 인계받아 후속 통합 checkpoint를 준비한다.

- [Bool 실제 근거](../../../packages/meari/.spec/evidence/verification/bool-props-2026-10-19.md), [Array checkpoint](../../../packages/meari/.spec/plans/array-kernel-checkpoint.md): 원본 OR 해소, 다음 sourceRefs:string[] 진단과 Repeat 미완료.
- [UI Native23](../../sots/verification/presentation/native-review-sdk-2026-10-19.md), [유형별 후속](presentation/review-type-native-follow-up.md), [제품 등록](presentation/proposal-product-registration.md): 합성 Cue 성공과 실제 DocumentCorrection/NeedsRepair 후속 구별.
- [AI provenance](ai-provenance-integration.md): 기존 API 재사용·원자 저장·fresh CLI·project binding. 원candidate 품질 결정을 수정 출력에 자동 전이하지 않는다. baseline/evidence 검증은 human gold/출력 품질 승인이 아니다.
- [회차·native 후속](native-resolution-handoff.md), [옵션 계획](native-options.md): 전체 제품 범위 유지.

## 자원·소유권·보류

[자원 예산](resource-budget.md): 저자원 소스 작업은 병렬, build/test/browser/실제 실행은 한 소유 프로세스 계열만. 두 logical CPU·BelowNormal·offline/locked·cargo -j1·단일 테스트 thread·기존 cache를 사용한다. 명시 반환과 입력 동결 인수 후 교대하며 timeout만으로 정상 실행을 재시작하지 않는다.

root Cargo/API/SDK는 SDK, 제품 presentation/TSX/plugin은 UI, packages/meari는 Meari, AI source/evidence는 AI가 수정한다. PM은 제품 코드를 대신 구현하지 않고 연구 서브에이전트는 읽기 전용이다. 새 AI 추론/다운로드·Pages·VM·DPI·새 장시간 시험은 보류하고 ChatGPT/Terminal 등 사용자 앱은 종료하지 않는다. 전체 goal과 실행 성공·품질 승인 구분을 유지한다.
