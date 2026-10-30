---
id: evidence:kinekino/20261029114323
title: Native surface·수신 관측 combined 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native surface·수신 관측 combined 검증

상태: **2026-10-21 SDK gate 마감**. 전일 시작한 검증 snapshot의 제품 kinekino-player 전체 test를 최종 input30 r5에서 구성별 한 번씩 완료했다. root live/index/gitlink에는 적용하지 않았다. 실제 OS probe·GUI·미디어 및 phase2 제출/인가 완료를 뜻하지 않는다.

| 최종 구성 | 전체 pass | ignored | 실패 | 별도 strict |
| --- | ---: | ---: | ---: | --- |
| default | 523 | 24 | 0 | all-target0 |
| native-proposal-window | 627 | 29 | 0 | all-target0 |

두 구성의 중복 시험을 고유 합계로 더하지 않는다. workspace fmt check0이며 다른 workspace package·독립 tools 전체 검증을 대신하지 않는다. root87992b30ea474b4f71e25502d06568ae4d4f3672/Meari d669694a01a00f03f506980df127b8734b6a61d0와 이전 소비 root98cd를 구분한다. 최종 input30 SHA4CBABEBFAC60CA7221B688E7CF3CBCA65888A5D9EEF6CBAAB9439B7066534E67, raw 근거는 `runtime/quality-review/sdk-native-combined-r1/`다.

최종 input30·snapshot 보호1262·root 보호907 및 default 실제 binary 모두 delta0이다. native binary는 default 빌드 직전 delta0을 확인했다. receipt29는 전부 terminal(성공25/원실패101 네 개 보존), 기록63PID는 부재이고 owned[]다. fixed runner SHA도 원 EEA9DEE6…5488D로 일치한다. heavy/shared snapshot은 최종 r5를 보존한 채 PM에 명시 반환한다.

PM이 exact30을 root commit **a8efdb6b6b1f674ad9bce1bf06e88b78684b2b44**로 인수했다(index empty/push0). 실행 gate의 root 기준87992b와 중앙 인수 commit을 구분한다. 이후 heavy/shared snapshot은 UI 담당 소유이며 SDK 추가 Cargo0이다. 다음 guarded2 단계는 별도 source pin·범위 배정 전 설계/읽기만 진행한다.

단일 heavy 운영은 아래 기록한 parity/snapshot670ms 겹침 위반이 있으므로 전체 과정을 엄격 직렬로 주장하지 않는다. 원 시각·receipt는 보존했고 이후 handle 회수→terminal→다음 실행을 적용했다. 이는 기능 시험 결과와 별도 운영 제한이다.

다음 절은 원 실패와 진행 이력을 보존한 것이다. 당시의 “진행 중/미완료”는 해당 시점 상태이며 최종 상태는 위 표와 아래 default 마감을 따른다.

## 입력 보존과 overlay

원 combined29 5BAC21407D206230B5F712F0D08861C8E85FC268E9C27CCA86EFFEBE3C1B4A33 및 독립 원19/11은 보존한다. 이전 snapshot1270 manifest9FBC2FA0A4EC0C9C89BF9D8078DEE97B87CB5FC8D92EAB30AC52DDD58DB61720을 재확인해 preoverlay 차이0이었다. exact29는 기존7/absent22이며 before 원문과 metadata를 저장했다. 보호 범위 snapshot1263/root907은 별도 manifest에 보존했다.

기존 등록 patch를 additive로 적용했다. 관측 shared6의 최초 check는 CRLF/LF context 차이로 실패했고 `--ignore-space-change` check/apply 뒤 모든 기존7의 frozen source와 EOL 외 diff0을 확인해 원 frozen bytes로 정규화했다. 신규22를 복사해 최종29 SHA 차이0을 확인했다. root live/index/gitlink 변경0이다.

## 최초 compile 실패와 최소 r2

native-parent-surface 최초 receipt는 launcher93120/child107796, terminal101/20816ms다. 시험 실행0이며 컴파일 오류2는 owner의 preflight import 누락과 windows-core Error::from_win32 미존재다. fetch_update deprecated 경고2도 관측됐다. 원 로그/receipt를 덮지 않았다.

기존3 파일만 수리한 r2 source29 manifest는8BFF3E458895BA7437C146243950CD61F766ECA18420CEEDADFC5E7C13C065C3, repair diff7CBFC32C2C534E97712CA19BDF51AECCA7AB4C7779231D5659E0CE9F9DA0E590다. preflight import, 설치된 windows-result의 GetLastError 기반 from_thread, 동일 checked 조건 try_update만 변경했다. direct fmt0이며 실제 집중 회귀는 현재 실행 중이다.

후속 r2 실제 집중은 surface19/recovery12/receive8/defaultguard1=새40 및 기존 outer6/child7/router12=25, 고유65pass다. 실제 native libtest binary는153817600bytes/SHA5A9C74871669C98537007947F2305F3B0609C9FE72042A92A3EF094F601BD38D였다. native 전체 --no-run도 terminal0/408465ms로 끝났으며 이는 전체 suite 실행이 아니다.

native all-target strict 최초101은 probe host의 큰 Err 두 곳(result_large_err)이고 원 로그를 보존했다. probe_host.rs 한 개만 Box 저장으로 수리한 r3 source29는E482DDD2D138485295E245C62D1554E52DCF2052DE491AF7EB418F54DF07C1C2/repair C26734723E599448E7F65F80F158E21388096B69B624D58323B1BA1CF3710DEB다. 원 input/partial owner를 보존하며 inert owner 저장이 OS보다 먼저인 순서는 같다. start_probe의 공개 Rust Err는 Box<ReservationFailure<NativeSurfaceInput>>로 바뀌었으므로 후속 소비자의 exact 타입 표시는 이에 맞춰야 한다. 이 source29 안에는 외부 start_probe 호출이 없다. generic 회귀는 concrete Box actual OS 증거가 아니다.

r3 native all-target strict0/33093ms, default all-target strict0/44387ms, workspace fmt check0을 확인했다. 최종 r3의 전체 suite/ignored 목록 및 실제 실행은 진행 중이며 65집중을 r3 전체 테스트 통과로 바꿔 기록하지 않는다.

최종 r3 native 실제 binary list는 terminal0/410447ms, ignored list는 terminal0/4208ms다. 목록상 총656/ignored29이며 lib551에는ignored6이 포함된다. GUI/WebView/network/실미디어/외부 verifier/actual-nine/corpus 요구 시험이 ignored로 분리된 것을 source 이유와 대조했다. 기본 suite는 CLI/LocalHost IPC·temp·자체 subprocess와 FFmpeg 기능 탐색을 포함하며 순수 시험이라고 부르지 않는다. 자체 held_file_child/verifier_fault_child의 명시 재호출은 보류 actual GUI와 구별한다. 테스트 환경 override는 발견되지 않았다. blanket --ignored 실행은 하지 않는다. native 전체 기본 실행은 목록 인수 후 진행 중이다.

native full r3 최초 실행은 terminal101/50516ms, lib544pass/1fail/6ignored에서 중단됐다. 기존 api/tests.rs::unsupported_commands_never_report_success가 모든 malformed{}를 Unsupported로 일괄 기대한 것이 원인이다. 원 로그에는 command 진단이 없었으므로 정확 실패 명령을 실관측했다고 하지 않는다.

PM 승인 exact1로 output-review3 기대와 command 진단을 추가한 fixture r1/input30 r4(67A93D8E44F925532BC4868D38CEB036D85AB122CF4D7EAE69AAA42AD05AD24A) 집중은 terminal101/52002ms·0pass/1fail이었다. 여기서 production.export_snapshot가 실제 InvalidArgument인 것을 처음 command 진단으로 관측했다. 앞서 output-review를 첫 원인으로 좁힌 소스 순서 추론은 정확하지 않았으며 이 실제 관측으로 교정한다. 원 fixture/두 실패 로그는 보존한다.

원 before API test1 SHA F9B6684F611AE7657C93BC6186BA03989E29F005154425BF63BEEDD0CDC68EA0와 원 snapshot 보호1263은 그대로 두고 active 보호1262/source1을 분리했다. root source/index는 변경하지 않았다. 현재 추가 코드는 freeze, 새 heavy0이며 전체 COMMANDS validation-before-unavailable 읽기 표 `runtime/coordination/sdk-api-unavailable-fixture-r2/dispatch-review.md`와 PM 독립 감사를 합류한다. snapshot1+outputreview3의 정확 기대 외 오류를 임의 허용하지 않는다. 유효 Unsupported는 기존 parity typed3/snapshot API 통합이 담당하며 제품 오류 우선순위는 수정하지 않는다. 최종 전체 default/native pass는 아직 없다.

PM 독립 전체 dispatch 감사는 COMMANDS79−discovery3=76, InvalidArgument 명시4/Unsupported72로 일치했다. 모든 command의 ok/data/code 진단을 끝까지 수집한 최종 fixture r3 EF881C706C0764A43F6D54CEB02F59BB3A312380E89C5FE75FEAE7E49473C645와 원 product29를 통합한 최종 input30 r5는4CBABEBFAC60CA7221B688E7CF3CBCA65888A5D9EEF6CBAAB9439B7066534E67이다. 이전 r4 및 원 실패 둘을 보존한다.

최종 fixture 집중1은 76행의 정확 actual/expected·okfalse/dataNone을 출력해 pass/51877ms다. 기존 parity2pass/44141ms, snapshot API7pass/670ms다. 단 아래 운영 위반 때문에 이 둘의 독립 실행을 엄격 직렬 증거로 사용하지 않는다. r5 native strict0/7450ms, default strict0/8673ms, fmt0이며 이후 source 변경0이다.

## 최종 native 전체 결과

최종 native list/ignored list를 다시 읽었고 ignored stdout SHA338A42A5ACA3609D0C9CDDDCFD6EFFD06F1FBE1E5971E26270D6F29C4B2A6CF0는 이전 인수 목록과 동일하다. 최종 native 전체는 **627pass/29ignored/0fail**, receipt launcher9392/child24424/terminal0/166493ms다. lib545pass/ignored6과 각 integration을 모두 포함한다. 0-test harness는 통과 개수로 더하지 않았고 집중65와도 중복 합산하지 않는다. source1 이외 제품 정책 변경은 없다.

실행 native CLI는29099008bytes/SHA81FB35383BA3A147A69A679318216373EEC923BA76211F807359E95222861FB5, native libtest는153819648bytes/SHAC30B4D3B055B7902DDA1982467E052DE99CCB823BDD0A9FC77ECA9E1838E94DC다. actual list의 binary identity 전부를 default 빌드 전 대조해 delta0이었다. default 전체/최종 보호/소유권 반환은 아직 남아 있다.

## 단일 heavy 운영 위반 기록

parity tool이 session5166을 반환한 상태에서 다음 snapshot7 시험을 시작해670ms 겹쳤다. 원 receipt를 지우거나 재실행으로 덮지 않았다. UTC receipt 생성/종료 metadata는 parity14:43:45.5476524→14:44:29.6969625, snapshot14:43:55.7705645→14:43:56.4492244다. source `overlap-audit.json`에 원 PID/receipt와 함께 기록했다. compile 겹침은 없지만 단일 heavy 운영 위반이다.

parity 원 handle 최종 exit0·receipt terminal을 회수한 뒤에만 후속 strict를 시작했다(native strict 시작14:44:43.8358771UTC). 기록4PID 부재와 해당 parent 후손0을 확인했다. 이후 모든 새 실행은 이전 handle 회수와 terminal receipt를 확인하고 진행한다. snapshot7 자체 결과를 보존하되 엄격 직렬 운영 증거로 주장하지 않는다. 최종 native 전체 suite의 직렬 실행은 계획된 필수 전체 인수이며 이 운영 위반을 숨기는 재실행이 아니다.

## Default 전체와 최종 대조

default actual list는 terminal0/368746ms이며 총547/ignored24를 확인했다. ignored list도 terminal0/6106ms로 source 외부요건과 대조한 후 blanket ignored 없이 실행했다. 최종 default 전체는 **523pass/24ignored/0fail**, launcher117892/child30448/terminal0/156736ms다. 원 handle58001도 exit0으로 회수했다. default lib는441pass/ignored5다.

default CLI는29115392bytes/SHA4518A7A94C09503026FD567F001974D82923DD07F59D791F68025730D8FF8AC7, libtest는93213696bytes/SHA50B4309EFCCEFEE6EDAC99B70E63E373955F08C7F3D20C055B03D7DAE5A722C8다. 구성별 top-level CLI 경로는 같은 cache 위치이므로 native CLI는 default 빌드 전, default CLI는 해당 실행 시점과 최종 시점의 identity를 각각 기록했다. native/default test binary 자체는 구성별 다른 artifact다.

`final-input30-differences.json`, `final-snapshot-protected-differences.json`, `final-root-protected-differences.json`, `default-binary-final-differences-r5.json`은 모두[]다. `final-receipts.json`과 `recorded-owned-pids.json`의29 receipt/63PID 및 `final-owned.json`[]를 남겼다. 이는 기록한 소유 범위의 종료 증거이며 사용자 앱을 종료하지 않았다. 현재 snapshot 최종30을 PM 인수용으로 보존하고 추가 Cargo를 시작하지 않는다.

## 인수 구분

새 surface31/관측9는 작성 수이고 아직 전체 통과 수가 아니다. default/native 제품 전체 test는 최종 고정 입력에서 목록/ignored 요구 검토 뒤 각각 한 번 수행한다. 기존 outer/child/router 및 status/control·host18/UIcapture-shell12 보호를 유지한다. 원 posted age는 그대로이고 native 진단은 인가false·guardNone·submit/resultUnsupported다.

실제 callback/OS partial fault·Destroy·parent loss, engine unique lease/whole outer proxy와 강제 host Drop는 미검증 제품 통합 잔여다. 후속 skip 소비의 공용 us/status 투영·submission receipt·ticket 재조회/소비·결과 읽기4항은 이번 gate scope 밖이며 종료 뒤 단일 계약 설계로 조율한다.

## 실행·종료 상태

고정 runner EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D, conservative2/affinity3/BelowNormal16384/descendant job, offline locked/-j1/singlethread를 사용한다. 모든 handle terminal·실행 binary·source/보호 SHA·소유 PID 부재 확인 및 명시 반환은 아직 남아 있다. 이 문서는 최종 완료 보고가 아니다.
