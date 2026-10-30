---
id: evidence:kinekino/20261029114047
title: Pending 문서와 공통 SDK headless 소비 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Pending 문서와 공통 SDK headless 소비 검증

2026-10-20 KST. PM 재개 인계의 UI consumer 기능 gate다. 실제 OS wake·키보드/IME·시각 일치·전체13화면 완료와 구별한다. [계약과 계획](../../../plans/player/presentation/typed-async-sdk-s1.md), [기존 bootstrap 검증](bootstrap-preparation-2026-10-20.md).

## 입력과 소유

원로그·각 실행 receipt·최초 수리 전 소스·candidate→final diff·최종16파일 사본은 `runtime/quality-review/ui-pending-document-r1/`에 있다. 원 후보15파일은 `runtime/coordination/ui-pending-document-s1/source-candidate-r1/`의 b16a1f599807d3d6d36c7e1335ce7190ccd76f2e5ce6bfd55ca6146969ce9e06 입력이다. 추가 파일은 proposals/mod.rs의 document_controller/pending_host/pending_controller/correction_panel 최소 export다. Native 의존 initial_panel/document_factory/window_bridge는 integration path module로 소비한다. SDK/API/Native/manifest/lock 변경은 없다.

SDK 최종 inventory CAF1353C6BF0E850CBF79A6C5192C67D5FF5CA896D0E0FE8AB51FA8FD6406325, Native 최종 8cf2ab2f6eb431a6138288f6475be73c7506fecb096cc3dde481d8df4fed4739 및 generic5 f469a872ece03c8c41ab4f2a534658a97cba0e94de2a7aa2227c7f05cdea8d2b의 실제 파일과 대조했다. UI 최종 inventory는 5b6515510b154805e5d8d199fdf778f3d439486a0fd6f7a15113def151d17a2f다. 최종 gate 뒤 모두 동일함을 재확인했다.

runner SHA256 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463, target `runtime/build-cache/sdk-product-parent-consumer`, affinity3/2CPU·BelowNormal16384·descendants Job, offline/locked/-j1·RUST_TEST_THREADS1을 유지했다. 명령 공통 인자는 `--manifest-path tools/player-ui-sdk-bridge/Cargo.toml --offline --locked -j 1`이다. GUI/브라우저/모델/다운로드/DPI/VM 실행0이다. 컴파일 초기의 낮은 CPU·긴 I/O 대기는 관측했으나 원인/AV 귀속을 확정하지 않았다. timeout으로 실행을 종료하거나 재시작하지 않았다.

## 실패 보존과 최소 수리

| 원로그 prefix | 실제 결과 | 원인과 수리 |
|---|---|---|
| compile-first | 763493ms/101 | factory budget 선언 범위를 shared preparation 앞으로 이동 |
| compile-r2 | 287591ms/101 | 시험 Port<I,O> 타입 명시·비교를 직렬화 값으로 변경·미사용 import 제거 |
| ack-repro | 17052ms/101·정확1fail | Core history 소거 후 close가 Stale로 ACK 재시도에 도달하지 못함 |
| ack-fixed | 9164ms/0·정확1pass | closing 설정→cached terminal poll/ACK→남은 pending 취소→quiescence |
| pending-document-first | 42716ms/101·4pass6fail | correction TSX exact prop 선언 누락 및 시험 관측 channel drop·알림/terminal 혼동 |
| pending-document-r2 | 18268ms/0·10pass | 선언 정합·관측 send 오류로 worker를 panic시키지 않음·3초 deadline/channel로 실제 준비 상태 확인 |
| full-first | 140907ms/101·22pass1fail | change_text 후 구세대 opaque selection 재사용. Stale assertion 유지 후 명시 index1 재선택 |
| full-r2 | 46058ms/0·83pass | 3task 실제 수정·원본 보존 의미 assertion 포함 |
| strict-first/r2/r3 | 32257/5037/15480ms·각101 | 큰 소유 enum 간접화·let-and-return 제거·시험 if 정리 |
| strict-r4 | 2431ms/0 | all-targets Clippy -D warnings |
| final-full | 233666ms/0·83pass/0fail/0ignore | strict 수리 뒤 최종16 source 전체 consumer 회귀 |
| fmt | 726ms/0 | consumer cargo fmt --check |

Core/SDK 결함으로 시험 채널 오류를 보고하지 않았다. 취소 알림은 terminal 완료 증명이 아니므로 실제 SDK helper의 preparation state를 확인한다. arbitrary sleep·무한 poll·실패 assertion 완화·SDK 업무 재실행은 없다. factory take_unbound_parent는 공개 retry_unbind 후 local session cleanup과 attachment 해제가 확인된 경우에만 원 parent를 한 번 회수하는 공개 복구 계약이며 initial_panel.recover_for_retry가 실제 사용한다. warning 억제용 allow나 복구 API 삭제는 없다.

## 검증한 의미와 한계

ACK 회귀는 controlled provider에서 Core 수용→ACK 실패→17개 다른 완료(limit8의2배 초과)→원 ticket history 소거를 실제 만든다. cached 결과로 ACK 성공1, SDK edit 호출1, 원 receipt 보존, 반복 close 추가 ACK0, 정상 registration shutdown을 확인한다. 재수용을 시도하면 Stale인 상태에서 성공하므로 Core 중복 수용이 없다. 실제 pending/foreign ticket을 강제 제거하지 않는다. 실제 worker join은 아래 공통 SDK 경로에서 별도로 확인한다.

실제3task는 공통 PlayerSdk+ProposalFiles 하나와 같은 client의 ExecutionOwner/AsyncProposalRegistration Native5를 사용한다. 원본 TSX ModuleProgram/InteractiveDocument 입력→callback queue→guard 종료 뒤 SDK edit→inspect→명시 accept→inspect→부모 close/join을 확인한다. runtime worker 호출 순서는 inspect/edit/inspect/accept/inspect이고 모두 main thread 밖이다. fixture create는 준비 단계의 기술 입력이며 runtime counter와 구별한다.

각3task는 명시 index1의 correction text/cards front/highlight title이 실제 입력 `명시 수정 日本語`인지 확인한다. index0와 선택 항목의 다른 필드·source_refs 순서, 전체 source cue ID/text/PTS·source pins·generated task/raw가 보존되는지 개별 assertion으로 확인한다. acceptance output이 저장된 draft와 일치하고 automatic_application_enabled=false인지 확인한다. revision 증가만으로 입력 적용을 판단하지 않는다.

초기7시험은 ACK 전 mount 금지, ACK 실패 시 같은 immutable raw/typed/prepared Arc 유지, canonical NotFound/queued Cancelled 명시 인수, 선택 불일치 성공을 실패 ACK로 바꿀 수 없음, 반복 부분 module/parent 실패 후 동일 worker/preparation으로 재조립·inspect 재실행0을 유지한다. 기존 Immediate/bootstrap 회귀도 유지한다.

현재 제품 런처는 이 adapter를 아직 등록하지 않는다. source-only21 후보는 신규 ACK 회귀를 더해22개가 됐고 기존61과 합쳐 consumer83이다. root 전체 cargo test나 실제 GUI를 실행했다고 확대하지 않는다. 역사적 원본/AI 품질·OS wake·입력·시각·전체화면 검증은 남아 있다. SDK worker_finished와 join 완료 사이 경쟁은 명시 close retry 계약으로 보존하며 후속 typed close/OS gate에서 다룬다.

## 최종 인수 상태

마지막 소유 간접화 후 final-full은233666ms/exit0·83pass/0fail/0ignore다. strict-r4는2431ms/exit0, fmt는726ms/exit0다. 동일 최종 source16 해시를 재확인했고 SDK/Native/generic5와 root·consumer·plugin manifest/lock 불변을 확인했다. final-input-observation.txt, final-manifest-observation.json, final-process-observation.json에 보존했다. 마지막 프로세스 관측에서 cargo/rustc/resource-runner 및 전용 consumer target 실행은 모두[]였다. 강제 종료·사용자 player 종료는 없다. 현재 UI gate의 단일 heavy 슬롯을 PM에 명시 반환한다. 후속 SDK typed close/OS gate가 입력을 바꾸면 이83 결과를 새 입력의 통과로 승격하지 않는다.
