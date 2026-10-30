---
id: evidence:kinekino/20261029114557
title: 로컬 review worker 실행 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 로컬 review worker 실행 검증

## 최신 판정 — r3 실제 시험 통과

아래 진행중/실패 설명은 각 역사 epoch의 상태다. 최신 r3는 SOURCE F7A32EFEE0AA82341B3A63687F4A3AAD58F32C6A2ABD34539D024C7300442C54, inventory A7FE68409362842B83ECCBFD3127B1ECFCC305D49789059F4490E643C0F7847E /2172파일이다. 254 제품/owned-process strict,fmt,255 actual target no-run,256입력고정,257 actual1시험,258전후2172입력 동일 모두 통과했다. 257 outer6077ms/actual5.28초,258원 identity/content 감사43135ms다.

정상 CLI를 실제 local worker의 시험 LocalHost/공통 SDK control에 연결했다. 두 CLI receipt는 accepted/completion=unobserved/engine_instance_id="1"을 그대로 보존했다. 독립 원 IPC 관측에서 요청17,000,000us→17.0초,25,500,000us→25.5초를 같은 instance에서 확인했다. 허용 관측오차100ms이며 frame-exact/tick proof가 아니다. 원 PID23372/creation134375049028265343, exit0/forced=false/physical_closed=true/Job Empty/ipc_exited=true/first_error=None다.

실 원본·tool·test/CLI producer·source receipt·nomination·B0 witness·부분 관측 파일의 전후 guard를 확인했다. producer witness211611F21DD7EE470693265A8B22730EF0BB18DE3E322DC2123CD329F6021294, 정상 CLI0B527048F1A03822A6F8F674FA07353F1C7ED8FFFD47F7917C13964DF07858BA다. 실제 tool F8BF7E8F…/원 media7CFC8B8C…/package46445F35…/item sprite-latin-17-technical/requestA518F587…는 이전 봉인 지명과 정확히 일치한다.

| 원 기록 | SHA256 |
|---|---|
| runtime/verification/review-worker-actual-datalab-r1/RETURN.json | 2629F0E2E59DFD93FF94985C088C611B7A33807FC252658619243DB8A932255A |
| allocation/WITNESS.json | 0B83E59FC6694D476E13F74B81C97309B5506CD9B055AA2AD520CCCCD50AF3BF |
| allocation/OBSERVATIONS.json | B118529741CA19A9349E2B9A1C4BC7243FCEA1EC2DD513B19DB2E13CDFA9C984 |
| allocation/evidence/INPUT.json | 070EA1A0E5804F8E2A48464B7E489368D661FA2FFB604C14F579121FD788C799 |
| allocation/evidence/PROCESS.json | 9560E5DE75ADBBE986AF064F1E10A8ACD9ABB98451C76D50D91C285FE389799E |

allocation은 runtime/build-cache/tmp/review-worker-29672-1793031300744710900이다. 원 stdout/stderr·INPUT/PROCESS와 사전 WITNESS/종료 전 OBSERVATIONS를 보존한다. worker에서 생성한 session/source·tool·Job·scope는 실제 종료까지 유지됐고 원 scope는 종료 전 해제하지 않는다.

현재 지원 근거는 trusted Rust assembly가 만든 시험 host의 공통 SDK control/정상 CLI seek와 headless clock·정상 정리다. 제품 public local-open/status/close 인수, frame-exact/Verified 저장·CLI, worker 부분 실패/취소/재시작 전체 행렬, AI9·GUI는 여전히 미완료다. RETURN의 verified_timeline/actual9_binding/gui_visual_acceptance/quality_approved=false를 유지한다. 이번 입력의 선택190/정상 bins 명시 재검증도 아직 미실행이다. stage/push 없음.

추가 보존: R1에서 실제로 달라졌던 run.rs와 actual test를 R2 보존본에서 역패치한 별도 사본으로 runtime/verification/review-worker-failure-r1-reconstructed에 보존했다. SHA F32FB64F6CD874444A605D8DFBE38B6AA47CF140DDA1F508BC4BDB779CA7D129 및5A84E11E4EC32DC074F43021DAC373CA0DD42FC052E7EAED759A85EA5E1B0ACD가 R1 원 TSV와 정확히 일치한다. 이는 동일 내용의 사후 재구성이며 원 physical FileID/실행 binary 사전 witness를 복구했다는 주장이 아니다.

후속 r2: SOURCE BBC0A3AB9340A32F07D331EADC844C1390F8E27FCA0FC5DA503DB940543646F3 / inventory A5F1800C… /2172입력. 252는 실제 CLI seek17/25.5초 경로 이후 정상 종료에서 실패했다. 원 PID23260/creation134375043007355784, 마지막 clock25.5/duration629.88771/1718x720, exit124/forced=true/family_empty/physical_closed/ipc_exited=true, 마지막 IPC error="invalid parameter"였다. 전체 시험은 FAIL이다. 253 전후2172입력동일 통과 후에만 수정했다.

원인은 send_legacy가 command 배열을 받아 IPC layer에서 envelope를 감싸는데 새 worker가 envelope를 한 번 더 감쌌기 때문이다. quit을 고정 배열로 전달하도록 수정한다. r2 raw evidence는 runtime/build-cache/tmp/review-worker-35920-1793030698890163800에 보존했고 수정 전 run.rs/test 및 실패 EXE2개·B0 witness를 runtime/verification/review-worker-failure-r2-preimage에 복사했다. 사본 verifier069F42FE…/CLI626FCB57… SHA는 원 WITNESS와 일치하며 물리 FileID는 다른 사본이다. 이전 R1의 witness 영속화 공백과 구별한다. 다음 시험은 CLI/clock 결과도 close 전에 OBSERVATIONS.json으로 먼저 영속화한다.

248은 첫 실패 전후2172입력의 content/FileID/bytes/time 동일성을 통과했다. 그 뒤 원 HostScope Arc를 worker 전체 수명에 보유하도록 수정했다. 후속 시험은 source/test/CLI/tool/nomination의 B0 witness를 media 시작 전에 create-new WITNESS.json에 저장·guard한다. 249 후속 strict 및 fmt check 통과. 250 no-run 뒤 새 r2 입력을 별도 발행하며 첫 r1 실패를 덮어쓰지 않는다.

첫 실행: 246 SOURCE226234DDF7B264523915E9DD7ABC0C1A452DBC2B1EEB26404AC896AFA8ABF5D9 / inventoryBC5C9462… /2172입력을 고정했다. 247 actual은 IPC 연결 전에 실패했다. 원 PID26440/creation134375038315820852, exit124/forced=true/physical_closed=true/family_empty=true/ipc_exited=true다. evidence runtime/build-cache/tmp/review-worker-35008-1793030229688840000/evidence의 INPUT/PROCESS/빈 stdout·stderr를 보존했다. CLI seek는 아직 실행되지 않았다.

원인 근거: GuardedQueue::scoped는 HostScope를 Weak로만 보유한다. 새 worker가 constructor의 임시 Arc만 전달하고 원 scope를 유지하지 않아 ConnectionFence가 닫힌 상태로 판정됐고 ipc::open이 연결 없이 반환했다. 원 scope를 worker 수명에 유지하는 수정이 필요하다. 이 첫 실패를 tool 부재나 실제 미디어 성공으로 바꾸지 않는다. 248은 수정 전2172입력의 전후 동일성을 확인하는 별도 감사다.

247의 test/CLI witness는 실행 중 guard로 보유했지만 실패 전에 값이 영속화되지 않았다. 따라서 실패 binary의 사전/사후 SHA 증거를 완성됐다고 주장하지 않는다. 후속 시험은 media 시작 전에 create-new WITNESS.json을 기록하도록 보강한다.

242 prepare 실패 unit1 통과,243 최초 actual target no-run 통과. 이후 비정상 child exit의 원 오류 및 create-new PROCESS.json 저장을 보강했고244 최종 제품/owned-process all-target strict 및 fmt check 통과. 최종 actual target no-run은245에서 다시 수행한다. 미디어 성공은 아직0이다.

현재 host toolchain은 cargo1.99.1/AA5B8980BE3681899FDE765BA47E2ED361463AE9DD8EE7CD1963C18D23743A0F, rustc1.99.1/BA03E3BA03A73D9FEF19AB3659140F94B43615EC1D587FE9BEFC9AED3BD5AA72를 직접 지명했다. root lockB2E3D540…와 Meari lock23792484… 동일. source inventory 도구78DD39F4DBB8C47CD0242B43A7E766CC92CD1DB17087F640E2BB64B6D94672D9도 유지했다. root HEADd13cbfca…/Meari0844f6c2… 그대로이며 원 source375 manifest/실패 계보를 대체하지 않는다.

2026-10-27 중앙 PM. 현재 기존 WIP·소유권·원 Sprite bundle/actual9 판정과 C cache/heavy1/CPU2·보류를 보존했다. 실제 미디어 단계 전에 최신 source를 새 checkpoint로 고정한다.

## 최초 실패 및 정적 검사

236 check는 ApiError의 미구현 Display를 사용한 두 진단 문자열에서 컴파일 실패했다. 원 로그를 보존하고 Debug의 원 code/message/details를 보존하도록 수정하여237 all-target check 통과. 238은 새 worker의 collapsible_if lint2개,239는 새 테스트 helper의 private module dead-code2개 및 비교 allocation lint로 실패했다. 모두 보존한 뒤 수정했고240 strict 통과. 실제 create-new INPUT 기록 및 ownership metadata 보강 후241 제품/owned-process all-target strict도 통과했다. 이 단계에 미디어 프로세스 시작은0이다.

242는 prepare 실패가 child 생성 없이 원 오류/terminal 상태로 돌아오는 unit 시험이다. 실제 SOURCE/custody와 정상 CLI/engine 시험은 이후 명시 배정하며 아직 성공으로 계상하지 않는다.

## 주장 경계

headless 실제 playback/clock은 GUI rendering/색상/입력 인수가 아니다. CLI receipt는 enqueue만 입증하며 same-instance clock landing을 별도로 확인해야 한다. Sprite source와 Sintel actual9는 다른 source이며 원 품질 판정을 바꾸지 않는다. SOURCE396 선택190/정상 bins와 E710 publisher checkpoint는 역사 검증으로 유지하고 이번 변경 입력에 승계하지 않는다.

modoo.today에서 작업 도메인을 재탐색했다. 일반 process 질의의 ML 관련 결과는 부적합하여 채택하지 않았다. software-qa 필터에서는 기존에 본문/필수 참조를 읽고 적용한 modootoday/software-qa/code-change-review-verification이 다시 확인됐다. 해당 스킬의 실제 호출 경로/판정 근거 분리와 test-fixture-and-mock-build의 synthetic/actual 구분을 계속 적용한다. 기존 정확 출처·해시는 [이전 B0](review-playback-b0-20261026.md) 및 [case 근거](review-case-evidence-resolver-20261026.md)에 연결한다.
