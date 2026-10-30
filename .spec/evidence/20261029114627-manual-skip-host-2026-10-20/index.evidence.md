---
id: evidence:kinekino/20261029114627
title: 수동 skip host·SDK·CLI 결합 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 수동 skip host·SDK·CLI 결합 검증

2026-10-20. 증거 디렉터리 `runtime/verification/manual-skip-host-r3-r1/`. [소스 계약](../../player/application/manual-skip-host.md), [실행 계획](../../../plans/player/manual-skip-host-gate.md). PM은 최종18을 root **43e6d9276b912992d818661dd0d4d25eaa634b5c**로 커밋하고 index 비움/푸시0을 확인했다. 아래 결과는 고정 입력의 실제 시험이며 실제 native seek 또는 자동 스킵 완료를 뜻하지 않는다.

## 입력·수리·실행 파일

실행 root는 public-p3-parent-r1 snapshot, base eb62672+PNG+SDK surface이며 Meari494를 유지했다. producer2699는 섞지 않았다. source17-r3 203D85…E4B3의 신규9를 정확 추가하고 기존8은 shared6/core2 patch로만 합류했다. before17.json과 before/에 기존8 원문·신규9 absent를 보존했다. 수리 전 인수 입력과 baseline-drift=[]를 확인했고 원 mtime을 기록했다. 기존8은 frozen 비교에서 줄바꿈만 달라 git diff --ignore-space-at-eol 차이0이었다.

원 core-host20은112320ms/exit0였지만 host/tests.rs의 중복 host::* import 경고가 있었다. 원 실행 terminal·PID 회수 뒤 import만 제거한 r4 source17 5839C15B…D3597/import diff8E1E3645…6EBE를 만들었다. 단언·정책 변경은 없고 최종 core-host20도 재확인했다.

원 CLI4/37은122405ms/exit0였으나 새 test target에서 기존 helper의 progress/poll 미사용 경고2건이 드러났다. 원 실행을 보존하고 PM 읽기 인수 후 tests/manual_skip_cli.rs 호출기만 기존 start→progress/poll1회→finish/결과 일치·Debug 관측으로 바꿨다. source17-r5 **C6184933885BC3376728E35B314EF88D155D8A70A7847338A32E6A7112419048**, helper-observation.diff **2A26BC084E97A4BBF6042A91A83DFA84DE5135976EBE662B697A785E7FB5B5CB**. 새 timer/retry/blanket allow·shared helper 변경은 없다. 원 helper의 blocking recv/join 한계도 그대로다. 경고를 사전 수리했으므로 host gate에 발생하지 않은 strict101을 기록하지 않는다.

기존 subtitle_conversion fixture2곳의 SDK에는 PNG 포트가 없는데 Windows CLI는 stock PNG 포트를 주입함을 실행 전 읽기에서 확인했다. 전체 capability 동일 단언은 유지하고 두 SDK 생성자에 cfg Windows 주입만 추가했다. 원 snapshot bytes/hash를 subtitle-fixture-before/에 **수리 전에 실제로** 보존했다. 별도 manifest **7A477285863A775FF8C212665B72C4ACC92418099F5A80174392994CC24A70E8**, 파일SHA **30C5F62B3932DAA456DCC0D315A1699721757924EB9B05B6456DD3C8F85CBECE**, diff1F051C…E845다. 이는 사전 fixture 정합성 수리이며 관측하지 않은 시험 실패로 표현하지 않는다.

최종17+fixture1의 `integrated18.json` SHA **D08F697E7117D09477FCC74E05D9BFA87601463FFAC7C7A444AD35B9AF9F00F7**. 정확 시험 소스18은 final-source/에 보존했다. 기존 core r3 정책·PNG·Meari494·SDK builder와 원 source17 r3/r4/r5는 보존했다.

fresh build58,864ms/exit0 후 CLI pre/post와 artifact SHA는 모두 **A016D645A674D35C71D8D823B1178C927600C99A035F4133A0BBD0F0246035D5**,29,099,520bytes다. `artifact/player-cli.exe`는 dev/debug/default-feature 검증본이며 release 배포본이라고 표시하지 않는다. r4/r5 테스트 수리와 추가 fixture는 제품 CLI 코드를 바꾸지 않았다.

## 실제 결과와 분모

| 검사 | 실제 결과 | 원 receipt 시간 |
| --- | --- | ---: |
| 최종 core+registry |20 passed/0 failed/0 ignored,417 filtered |22496ms |
| 최종 r5 실제 endpoint CLI |4 passed, 실제37요청 |130260ms |
| 기존 PNG API/CLI |7 passed |12334ms |
| 자막 conversion/range |9+4 passed, 외부 corpus1 ignored |46438ms |
| capability parity |2 passed |66030ms |
| default all-target strict |0 |48832ms |
| native feature all-target strict |0 |56988ms |
| Cargo fmt check |0 |1642ms |

고유 active **46pass**, ignored1은 별도다. 원 r3 core20과 r4 CLI4/37을 최종 수에 다시 더하지 않는다. 전체 root 시험이나 실제 모델·native GUI 시험을 실행했다고 하지 않는다. PNG backend14는 이번 host 연결 변경에서 불필요하게 재실행하지 않았다.

최종 host-cli-r5/에는 원 endpoint·request/response/exit·CLI SHA JSON37개와 --nocapture 원 Debug가 남는다. 기존 r4 host-cli/37개도 보존하지만 중복 성공수로 세지 않는다. PNG 실제 호출11개 원 bytes/입출력 증거, 자막 range call19개와 source/output digest, capability79개 schema·unsupported72·malformed3·discovery1(155 CLI 호출)을 별도 보존했다. 37은 신규 host 시험의 요청 수이며 gate 전체 CLI 수로 확대하지 않는다. 합성 seek counter는 Rust assert의0/1 검증이며 호출 JSON에 별도 계측 counter를 기록했다고 주장하지 않는다.

## 실제 endpoint와 합성 경계

같은 LocalHost IPC에 주입한 장수명 SDK/registry를 여러 실제 CLI 프로세스가 사용했다. open/edit/confirm/policy/inspect/skip·TooLateOrUnobserved 취소·close와 재-open guard, evaluate/override·stale revision 후 canonical snapshot 불변, 새 pass, typed SDK 결과 동일을 확인했다. unknown 포트에서도 Auto 설정은 편집하지만 skip Unsupported/submit0·schema/capability가 일치했다.

재시작 시험은 **한 test process 안의 실제 LocalHost 교체와 고정 fixture generation 두 개**다. `--serve-native` host subprocess나 skip OS GUID factory를 실제 재시작·실행한 증거가 아니다. 포트는 합성이며 실제 mpv seek/landing·in-flight 철회·OS seekability/관측 freshness·자동 host timer·durable 저장/복구·AI 검출은 여전히 미지원/미검증이다. native 조립은 seek_submission=false 및 Unknown 차단을 유지한다. 모든 builder의 skip owner 이전은 읽기 확인이며 PNG builder 회귀가 그것을 전부 직접 계측한 시험은 아니다.

## 보호·원 실행 회수와 PID 재사용 정정

fixed EEA9 runner 직접 conservative2/offline/locked/-j1/testthreads1/기존 native cache를 사용했다. 전 receipt의 admission accepted·mask3/논리CPU2/BelowNormal16384/descendant Job을 확인했다. root live/index는 이 세션이 변경하지 않았다.

최종 source18/보호615 차이0이며 원 protected-before614는 덮지 않았다. 기존 helper1은 원614에 없어 별도 helper-protected.json으로 더했고 snapshot helper의 Git blob acf52f54…f4fc가 eb62672 기준과 같음을 확인했다. 새 자막 fixture는 원615에 없으므로 가짜로1을 차감하지 않았다. raw helper/fixture 원문과 정책은 보존했다.

원 receipt11 모두 terminal0이다. runner/child22 + 원 CLI37 + 최종 CLI37 + PNG CLI11 = 관측 PID107개를 기록했다. parity/자막의 별도 CLI PID가 모두 이107에 포함됐다고 말하지 않는다. 각 harness의 원 child wait와 pass 결과를 따로 근거로 삼는다.

최초 PID 숫자 재조회에서 이전 CLI79112 자리에 새 conhost가 보였는데, 이 분류 전 owned[]라고 요약한 보고는 정정했다. 원 nonempty 조회는 pid-query-initial-raw.json에 그대로 보존했다. 원79112는 host-cli.stderr:42의 CLI exit0/24ms이며 원 로그 종료시각은 UTC12:56:22.9918693(KST21:56:22)다. 새 conhost는 KST22:15:23.9701839에 생성된 다른 이미지로, 원 CLI가 남은 것이 아니라 PID 재사용이다. 원 CLI의 정확한 creation/parent는 helper에 없고 새 conhost parent도 CIM 조회 전에 소멸해 미관측으로 남겼다. 부모를 발명하거나 PID만으로 종료하지 않았다.

`pid-reuse-79112.json`에 비교/한계를 기록했다. UTC13:16:40의 현재 CIM=[]·전체 PID 재조회 final-pid-requery.json=[]를 확인한 뒤 **그 실제 새 결과로** final-owned.json=[]를 갱신했다. 이전 nonempty 파일과 관측시점을 지우지 않았다. 사용자 앱/다른 프로세스 kill0이다. progress 문자열은 완료·timeout 보장으로 확대하지 않았다.

heavy/snapshot은 PM에 명시 반환했다. PM은 최종 해시·PID 구분을 별도로 확인해 root exact18 commit43e6d9276b912992d818661dd0d4d25eaa634b5c를 수행했고, 이후 UI capture/shell이 소유하므로 추가 시험·snapshot/소스 수정은 하지 않는다. 이 단계는 보고 정본만 마감한다.

스킬 적용: modoo.today requirement-to-testable-stories의 경계·동일 owner/다른 host·stale 요청 분모를 실제 fixture로 확인했다. code-change-review-verification의 소스/실행/미검증 분리, 경고·fixture 정합성 수리 별도 핀, 최초 관측 보존과 PID identity 재분류를 적용했다. Rust/SRP 및 중앙 자원·소유권·추론/download 보류를 유지했다.
