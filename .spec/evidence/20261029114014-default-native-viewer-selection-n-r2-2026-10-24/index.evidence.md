---
id: evidence:kinekino/20261029114014
title: Selector r2 실제 N1 정상 종료 반환
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Selector r2 실제 N1 정상 종료 반환

PM이 READY F4CB482D/원 CONTROL ED44로 N1을 배정했다. 새 H8AA040B2와 normal playerCF6BCAA2/CLI7E16F9D8 기존 사본을 사용했고 원 rootabb63/libraryf85/lock8F·carry1456661D1A6를 유지했다. Cargo/공유 source overlay/추가 Close/Run retry/Abort/kill/C0이다. 새 독립 A/B는 동일 SHA/다른 FI265c1·265c2/actual volume4061598778이며 A만1회, no_args/projectnull/argv0/case original-1/30s·cleanup5s/Runwait33·IO3/host diag1 상속이다.

## 실제 결과와 관측 경계

`runtime/verification/default-native-viewer-selection-n-r2/N/07-run.receipt.json`은 terminal0/7732ms, 원 시험1pass/0fail이다. 같은 H host92800/outer22216/session88822에서 State pre1→Run1→State post1을 수행했다. 원 request는 run-35328/bodySHA2d8fe83230ba74f24d8619d79a0b4acdf955780a60f834e1a06b5b15d6a40228다.

선택 HWND92804648/PID45652/생성 FILETIME134372875830016070/thread59780/class Window Class/title Meari Native/visible·top-level true/parent0·owner0/style382664704/ex-style262416을 실제 관측했다. parent witness3은 written3/failed0이며 원 close method/PostMessage ok, callback_received는 parent에서 여전히null이다. 게시 수락만으로 callback 수신을 표시하지 않았다.

별도 원 child stderr21줄/2354bytes는 같은 window의 CloseRequested op3/s5(n10), drive 진입/점유/복귀(n11~13), Destroyed op4/s5(n18) 및 해당 drive 복귀(n21)를 기록했다. 이 pinned Win32 backend의 WindowId=HWND source mapping으로 parent 선택과 child renderer window를 상관시켰다. child-local clock을 parent와 직접 등치하지 않았다. 기록된 loss/io는0이며 quota/IO 누락 가능성과 계측 효과를 일반적으로 없앴다고 주장하지 않는다.

원 State post는 normal_success=true/physical_completion=true, fresh observation.physical_closed=true다. 원 family-receipt는 child_exit0/family Empty/force_requestedfalse/stdin_eof_requestedtrue이며 같은 PID/creation이다. State 후 응답 SHA `AC2DE75E90DEF4A328E22821AB8B526ACAF8E8566ED6CDE408A3748C83F82B89`, child stderr SHA `FB19110B5F2B8103F022924C600B9A68FCD6AD8C00A345A26E585020C92FDBE1`로 보존했다. stdout.raw는0bytes다.

## 정상 host 정리·감사

원 closed를 확인한 뒤 승인된 ReleaseClosed와 FinishHost를 각각1회 수행했다(383ms/293ms/각 terminal0). released_original_summary에 같은 정상 receipt를 보존하고 original_owner=null을 확인했다. host outer는 terminal0/102318ms로 끝나 session88822를 회수했다. 이 host 서비스 수명은 case 예산7732ms와 구분한다. 비정상 Abort나 PID kill은 사용하지 않았다.

receipts6 모두 terminal/exit0, 원 evidence16 및 runner log12는 exclusive open/end read=-1로 닫힌 EOF를 확인했다. 기록 PID13 모두 부재/writer0/pending0/live session0이다. 원 고정 H/player/CLI/A/B/config6의 fresh SHA·bytes·mtime·FI·attrs 변화0이다. C2/캐시 보호는 CPU 반환 EB7F7147을 재사용하며 전수 S/T 재감사·중복 backup0, known2·alias FI 예외/all_cache_protected=false를 유지한다.

이 결과는 **계측된 no-args N의 viewer 선택·CloseRequested/Destroyed·원 exit0/Empty/EOF와 정상 host 정리** 범위다. parent renderer_frame_observed=false를 첫 frame/시각 인수로 바꾸지 않으며 no-args의 미발행 SDK telemetry나 전체 기능·전체 shutdown을 완료로 표시하지 않는다. 원 비계측 실패2F742932와 이전 진단 실패D1ACD4BF는 그대로 보존한다. 이번 pass로 두 실패의 원인을 소급 확정하지 않는다. configured C는 별도 입장·배정 전0이다.

이번 세션에서 읽은 modoo.today code-change-review-verification9f73865f/필수 verdict-table9d9619a8·fixturea393f15f의 원 소유·관측/추론·성공 범위 분리를 적용했다. 정확 본문/필수 참조 SHA는 selector HANDOFF/parent witness 후계 정본에 유지한다. 새 비Rust 검증 프레임워크0이다.
