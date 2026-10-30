---
id: evidence:kinekino/20261029114516
title: 명시 카드 CLI host와 typed SDK 제출 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 명시 카드 CLI host와 typed SDK 제출 검증

최종2016 custody 종료0: current source content/identity unchanged files=2357. R111 고정 입력과 실제 CLI/지정190/정상bins 이후 입력이 동일하다.

## 최신 R111 보강 및 통합 gate

R111은 CLI 시험만 보강했으며 제품 코드는 R110과 같다. 기존 wrong-frame 요청은 이미 성공한 출력 경로를 재사용했지만 실제 응답 message는 frame pin 불일치였음을 확인했다. 이후 미래 회귀에서 출력 충돌이 frame 검사 누락을 가리지 않도록 새 경로로 변경하고, 최상위 manifest SHA/bytes·idempotency key 누락의 새 경로 거절/미생성도 명시 검사했다. 원 R110 통과 근거와 preimage-r110-card-cli-evidence를 보존한다.

R111 SOURCE.md SHA256 F86D2D003A4F3A82D3AEDE04C4D82510238794D8995D4E59F6046AF57ADBA2EB, inventory966D920CB8CBFA3CD613A7331F5F843E79A18D31FC2D9554260CBC2EE8BDF225/2357. input-r111에 고정했다. 2001fmt0/2002snapshot0/2003 카드 strict0/2004 카드 no-run0/2005 실제CLI1통과(7.79초)/2006default strict0/2007 일반 Native no-run0/2008fmtcheck0/2009 일반 host 입력4통과를 확인했다. R111 카드 CLI9요청은 아래 R110의8요청에 missing-key를 추가한 것이며 typed 제출/SDK polling은 별도다. stop은 동일하게 completed3/cancelled0/interrupted0·pipe/sdk joined·dispatch completed/poisoned=false이고 원 process는 exit0/nonforced/family Empty다.

fixed-r111 카드 CLI SHA256 AB7CC4EAF9D19A8DD99F2090957A88AE7B3436097D6C87B9D2D4CC7F72F4C2B6, jobs-9b29c59c4e3f5fb0.exe F7ACCBA8BE4EEF609C569E18E2DF8B34841E520B10BE21F7C5A9746FC1AAEB80. PDB도 보존했다. R110 CLI6897...와 바이트가 같다고 가정하지 않고 실제 R111 producer를 다시 고정해 runtime/verification/source-sdk-r111-card-cli-r1/card-bundle에서 실행했다.

2010 Product145/2011 Core18/2012 Native26/2013 OS1 모두 이번에 실제 실행·종료0이다. source-sdk-r111-exact190-r1 각 group-return.txt의 execution_result=Ok(())/producer_integrity=Ok(())와 같은 R111 SOURCE SHA를 확인했다. Product는2007 새 JSON manifest, Meari는 기존622/624 producer를 재사용해 실제 실행했다. Meari를 새로 컴파일했다고 주장하지 않는다. 과거 D 명의 원190 plan은 보존하고 실제 cache/실행은 C 경로다.

2014 정상 native-proposal-window player/player-cli 명시 build 종료0. fixed-r111/native-bins의 GUI SHA256 4A7874CEC4320478F7C2B1E484AB6CF4F6067321BD800AD90E8FB695A846558C, CLI5CEB231214DE5B97B835D817CA03D8AC2CB9D6ECC775EC41800BC5DC4D344E21와 PDB를 보존했다. 기본 debug 경로도 정상 구성으로 복원했다. 2015 실제 일반 CLI에 카드 host opt-in을 주었을 때 child exit3/code Unsupported/renderer 없는 빌드라는 메시지로 거절했고 지정 Jobs root는 비어 있었다. 작업 host를 조용히 시작하거나 renderer를 설치하지 않았다.

위 성공은 명시 카드 구성의 선택 CLI 여정·현재 지정190·정상 bins 근거다. 전체 CLI/GUI·실제9 품질·전체 카드 요구 인수로 확대하지 않는다. 현재 구조에서 operation별 상세 JSON schema, 부분 retry/cleanup/ACK 복구·다른 template/canvas·GUI는 계속 미완료다.

2026-10-28 R110 중앙 PM. [CLI 연결 계약](../../player/application/production/card-bundle-cli.md), [원 Jobs 발행/취소](card-bundle-jobs-20261028.md). --serve-jobs의 명시 --card-project-root opt-in에서 기존 자막2작업과 production.export_cards를 하나의 registry에 등록했다. SDK는 같은 Arc<CardProjectStore>를 공유하며 object-safe submit_card_bundle이 기존 jobs.submit을 호출한다. 직접 production.export_cards RPC는 Unsupported를 유지한다.

R110 SOURCE.md SHA256 4B61C956B840C69B82FDEBC3C0ABDC757398B210EBF17A133BCC38BA463F3C40, inventory64FB4B5C3983A62A748BC9E6163107D042785319B248C8CB97F27875B83C9760/2357. mirror/runtime/verification/input-r110에 고정했고 원문은 preimage-r109-card-cli-host에 보존했다. root WIP·독립 Meari·원 실제9·기존 VAD/프레임 소유권을 유지한다. 기본 host는 opt-in 없음/카드 구성 없음, 미구현 VAD+카드 혼합은 명시 Unsupported다. source375/canonical 적용으로 승격하지 않는다.

로그 루트 runtime/verification/source375-datalab-b0-r1-logs.1992/1993fmt0,1994snapshot0,1995 card-review-export all-target strict0,1996 card CLI/jobs no-run0(1분50초),1997 host 입력4시험통과/35filtered,1998 실제 명시CLI1통과/67filtered(7.81초),1999 default all-target strict0. 이번 단계의 첫 실행 실패는 없으며 이전 R108 첫 gate 순서 실패는 원 증거에 보존했다. transitive blitz-paint deprecated 경고는 별도다. 원 live handle terminal 확인 후 다음 무거운 작업을 시작하며 CPU2/BelowNormal/heavy1·C-only/cache/tmp·locked/offline/-j1을 유지했다.

## 실제 명시 CLI

카드 구성 고정 exe/PDB는 runtime/build-cache/native/source-sdk-integration-r1/fixed-r110에 있다. CLI SHA256 68971B25F65BF4433B1DFB8D9441D0426F78E52AAAD2BABD4CC4EBDB37F1C1A4, jobs-9b29c59c4e3f5fb0.exe 7C416403D1518470B725B73B4CDBA49FED52CEAEE453C604DAEDABA42769EF03, CLI unit player_cli-14ed01a06c012af8.exe DF59E476143F4BE1F8044792765D45F8B187CB2AED76C6B8712498D3FD57D4F7.

runtime/verification/source-sdk-r110-card-cli-r1/card-bundle에 복사된 동일 SHA CLI와 원 host/요청/stop stderr/stdout·prepared/Jobs/source blob·최종 두 카드 묶음·card-summary.json을 보존했다. 기존 Fixture/OwnedHost/try_call_until/try_terminal_until/ScenarioFailure harness를 재사용했다. 새 프로세스 상태 기계/재시도/GUI driver를 만들지 않았다. original100초/cleanup10초 안의 ready/request/stop cap을 사용하며 timeout 관측을 완료로 바꾸지 않는다.

명시 --serve-jobs --job-root store --card-project-root projects로 원 host를 시작했다. ready의 operations는 subtitles.convert,subtitles.export_range,production.export_cards 세 개와 정확히 일치했다. 실제8개 CLI 요청은 capabilities/inspect-project/submit-card/get-card/replay-card/wrong-frame/direct-export-unsupported/legacy-subtitle이다. 별도 typed SDK 제출과 원 SDK endpoint polling은8개의 CLI 요청 수에 합산하지 않는다.

CLI 제출 카드1과 같은 원 endpoint의 typed SDK 제출 카드1이 각각 Completed로 끝났고, 두 manifest의 project pin·PNG SHA가 같았다. 서로 다른 job/attempt/descriptor를 동일하게 위장하지 않았다. manifest 각 파일의 bytes/SHA를 실제 읽어 확인했다. 동일 요청 재조회는 created=false/같은 job ID다. 틀린 frame pin은 RevisionConflict/exit5, 직접 export RPC는 Unsupported/exit3이며 이를 실행 성공으로 세지 않는다. 같은 registry에서 원 subtitles.convert도 실제 실행해 WebVTT를 확인했다. 자막 range는 등록 확인이며 이 시험에서 실제 실행한 것은 아니다.

stdin EOF 뒤 원 host는 강제 종료 없이 child_exit0/family Empty/stdin_eof_requested=true를 관측했다. stop은 completed3/cancelled0/interrupted0, pipe/sdk joined·dispatch completed/poisoned=false다. 이를 모델·GUI·광고차단/실제9 사용자 품질 인수로 확대하지 않는다. 기본 카탈로그103·별도execution host10과 직접 export Unsupported는 유지한다.

이 출력은 고정 제안 템플릿의 합성 카드 review_artifact다. 품질/템플릿 승인false·원 실제9 NeedsRepair/사용자 승인0/9를 유지한다. 추가 템플릿/빈 캔버스/역할별 디자인·부분 성공 재사용/실패만 retry·staging cleanup·발행 후 ACK 복구·disk/permission/crash·VAD 혼합·GUI/시각 인수는 미완료다. WARP/추론/다운로드/수동 GUI/push를 실행하지 않았다. 설치/전역 설정 변경 없음.

스킬은 기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용했다. 명시 구성·실제 CLI/typed SDK·원 host EOF/원 worker receipt·기술 결과/사용자 품질을 분리했다.
