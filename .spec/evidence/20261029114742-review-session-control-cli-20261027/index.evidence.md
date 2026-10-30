---
id: evidence:kinekino/20261029114742
title: SessionRef 결합 제어 실제 CLI — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SessionRef 결합 제어 실제 CLI — 2026-10-27

후속 verify-current는2188파일 content/identity unchanged, 종료0으로 확인했다. 원 SOURCE/실제 결과/실패는 보존했다.

중앙 PM. 원 source375 private overlay·원 실패·소유권·WIP를 보존한다. 새 입력은 `runtime/verification/source375-review-session-control-input-r1`: SOURCE.md SHA256 `20886037BC0B3F8109D12898012B72071F09C998340E98F35C8D0866298D41DA`, inventory `06611A49D4DDF086C52E0008B7E1FF91B97C12B5E6F79F53FCD17586553015EE`, 2188파일이다.

공개 reviews.playback.control은 case/association revision/host scope/session ordinal/engine instance에 결합한다. 원 proof·custody를 IPC 발행 전에 재확인한다. Play는 관측 clock이 retained coverage 안에 있을 때만 허용하고, Seek는 양 끝점을 포함한다. 범위를 벗어난 재생 관측은 동일 worker의 Pause를 발행하며 500ms 내 일시정지 관측이 없으면 원 owner 정리로 진행한다. polling 정책이며 frame/sample 정확도 보장이 아니다. 제어 receipt는 accepted/unobserved이고 관측 상태와 분리한다.

로그는 runtime/verification/source375-datalab-b0-r1-logs 아래다. 353 첫 compile 실패(boolean schema move)는 원 로그와 preimage에 보존했다. 수정 후 354 정책/세션14, 355 strict, 356 worker9, 357 no-run, 358 actual1/CLI35, 359 discovery97이 종료0으로 통과했다. cargo fmt --check도 통과했다. 전체190/명시 정상bins는 이 입력에서 아직 재실행하지 않았다. 이전 FECF 통합 성공을 승계하지 않는다.

실제 결과 `runtime/verification/review-session-control-cli-actual-datalab-r1/RETURN.json` SHA256 `293ED786B08314438B9F9D0D5B00CF149DBE196FC4D62E2CE25F496DD49AA267`. 고정 Sprite/fixed mpv의 headless CPU 실행에서 Play/Pause/Seek/volume/mute, foreign session/instance·범위 밖·종료 후 거절, close/replay/Unverified child 미생성을 확인했다. 경계 paused clock은25500000us, 이번 표본 overshoot0us였다. 일반적인 hard-bound 보장이 아니다. 실제 CLI SHA256 `75D4FC2B694369DFA16D1A8589937B45CEADCCA604740EE02D3FBEA42E696E2C`는 시험 구성 산출물이며 명시 native 정상bins를 대체하지 않는다.

기본 CLI discovery는 runtime/build-cache/tmp/cli-discovery-26828에97명령/schema 일치를 기록했다. adapter 없는 기본 구성의 review playback4는 Unsupported다. 제품 기본 host/fresh scope·재시작 행렬·frame 실제 관측·전체 CLI·AI/GUI 인수는 남는다. actual9 연결=false, quality_approved=false. 수동 GUI·추론·다운로드·WARP·push 보류 유지.

기존 modoo.today software-qa/test-fixture-and-mock-build 및 code-change-review-verification을 적용해 합성/실제·수락/관측·기술/사용자 품질을 분리했다. 스킬 본문/참조 SHA 및 출처는 선행 review-worker-command-gate-20261027.md의 인수 기록을 따른다.
