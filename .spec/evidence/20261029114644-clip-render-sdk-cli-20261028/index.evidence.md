---
id: evidence:kinekino/20261029114644
title: 숏폼 SDK·CLI 실제 연결 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 숏폼 SDK·CLI 실제 연결 검증

카드 회귀 후 최종 정리:2491 기본 bins 복원 종료0 뒤2492 원 custody도 종료0으로 확인했고,2493 custody를 단독 직렬 재실행해 종료0/2429파일 내용·identity 동일을 확인했다.2492 overlap은 위반 기록으로 보존하고 최종 사후 근거는2493이다. 복원 산출물은 fixed-r162/native-bins-restored에 별도 보존(CLI SHA F51179EF627E39FBE7B46EF727326D5BBFC9EEF6FA89966D9C5C7940825A6FEE,GUI SHA4A93991C7BA0A91CD41C7CA2DF6951F90432727DA8F393C459844F7C6AC4C5B2). 기존 실제 CLI 시험은 fixed-r162/native-bins의 A94B...를 실행했으므로 복원된 다른 bytes의 실제 성공으로 계상하지 않는다. 원 검토 기준선과 실행 증거를 그대로 보존한다. card feature registry 실제 회귀 항목은 이번에 확인됐고, 다음 제품 구현은 source probe SDKCLI 노출 및 관측부터 프로젝트 작성·렌더까지의 여정이다. VAD/card/clip 전체 동시 구성과 기타 남은 범위는 유지한다.

R162 card feature 회귀 결과:2488 strict/2489 no-run/2490 실제1시험 종료0(46.17초). 별도 fixed-r162/card에 exe/PDB를 보존했으며 card CLI SHA3E45878E5039028989FADE9DB4572F58B3644B160C4C1429A2959F940846FB39, Jobs 시험 SHA E895C43BEB070E3AED063708974101ED5AF5AAAC9763BF4D4CE7E5A0B021AC83. runtime/verification/source-sdk-r162-card-cli-r1/card-bundle에서19 CLI 요청·typed SDK·원 Jobs12완료·pixel_sha_equal=true·정상 EOF/pipe·sdk joined/cancelled0/interrupted0을 확인했다. 원 소유권·proposed template·품질 미승인 상태는 유지한다.

자원 배정 일탈 기록:2491 기본 bins 복원의 exec session50765가 terminal 전인데2492 custody session53146을 시작해 잠시 겹쳤다. 소스 편집은 없었지만 heavy1 직렬 배정 위반으로 기록한다.2491은 종료0/25.75초를 원 handle로 확인했고2492도 원 handle로 종료를 확인한다. 이후 custody를 단독 재실행해 최종 사후 근거로 사용한다.2492의 결과나 일탈을 숨기지 않는다.

2488 card-review-export all-targets strict 종료0(54.782초). 같은 R162 입력에서2489 card feature의 Jobs/CLI no-run을 exec session43223으로 실행 중이다. 다음 실제 회귀는 기존 card_bundle::configured_cli_card_bundle_and_typed_submission_share_durable_jobs_and_owned_eof이며 원7 CLI 성공을 이 카드 회귀로 대체하지 않는다. 새 card exe/PDB는 별도 fixed-r162/card에 고정하고 기존 default fixed-r162 산출물을 보존한다.

동일 R162 source를 유지한 추가 card feature 회귀를 시작했다. 2488 native-proposal-window,card-review-export all-targets strict는 exec session4081에서 실행 중이다. 원 handle 종료 확인 전 source 변경/추가 producer를 하지 않는다. 이 결과는 기본 feature gate와 별도 기록하며 아직 통과로 계상하지 않는다.

R162 통합 gate 완료: 2483 Product145/2484 Core18/2485 Native26/2486 OS1의 각 group-return에서 execution_result와 producer_integrity가 모두 Ok(())이며 동일 R162 SOURCE pin임을 확인했다. 2487 사후 custody도 종료0,2429파일 내용·identity 동일이다. Meari는 기존 고정 producer 산출물의 실제 재실행이며 새 컴파일로 계상하지 않는다. 기본 feature에서 실제 CLI/typed2렌더·기존 host4시험·strict/no-run/fmt·정상2bins·전체190·사후 입력 검증이 완료됐다. card-review-export feature registry 회귀와 제품 source probe API는 다음 미완료 범위다. 아래 진행 기록은 당시 관측으로 보존한다.

2026-10-28 중앙 PM, R162 private overlay. 계약은 [원 Jobs 연결](../../player/application/production/clip-original-jobs.md), 원 Jobs 자체의 R159 근거는 [별도 기록](clip-original-jobs-20261028.md)이다. 원 소유권/WIP·첫 실패·보류·source375의 비정본 상태를 유지한다.

R161 첫 실제 실패는 2474와 runtime/verification/source-sdk-r161-cli-r1에 보존했다. 실제 source_expired 종료8을 시험이5로 기대해 typed SDK 제출 전에 멈췄고, 기대 종료값만 수정했다. preimage-r161-cli-exit에 원 시험을 보존했다. 실패를 전체 CLI 성공으로 계상하지 않는다.

R162 입력:2429파일/inventory6500BAC55E9F117BEB59B1867370B2E90B2C6C26FEC10DB045BE05BDD8010994, SOURCE.md SHA4B8425CB131339899D0E198D901B71086F3829E7F508DF82A1B13EC342CB1708.2475fmt/2476snapshot/2477기본all-targets strict/2478no-run/2479정상2bins 종료0. 실행파일/PDB는 fixed-r162/native-core와 native-bins에 보존했다. CLI SHA A94BBD12B82D52807E5222679450F475D050783219FD97A3E496535CE2046480, GUI SHA670085F18AE692E684589072FBB2827C290091AB63449C6AA1625D8E8416C79B. R161과 같은 제품 bytes이지만 R162 입력에서 빌드 재확인을 수행했다.

2480 실제1시험/18.04초/종료0: runtime/verification/source-sdk-r162-cli-r1/clip-render. capability/schema/저장 프로젝트 inspect/jobs.submit/get/replay/wrong-project-pin의7 CLI 요청과 별도 typed submit_clip_render를 확인했다. 원 job-4dd90b33245747479448e70b1fc6c958와 typed job-96fc421e5e904cd5b939200d16534b1d가 각각 실제 렌더 Completed가 됐다. 첫 manifest SHA41602993c68f94ba04e0754c920a4488d733e177cc28279a0fabdc375e9eb060, 원 job 신원과 파일 SHA를 대조했다. 동일 key는 원 job을 재사용하고 다른 프로젝트 pin은 source_expired로 거절했다. 정상 EOF 후 원 family Empty/비강제 종료/pipe·sdk joined/completed_total2/cancelled0/interrupted0을 확인했다. clip-summary.json과 logs/stop.stdout에 근거가 있다.

2481 기존 host 구성4시험과2482fmt check 통과.2483 Product145 회귀 진행 중이며 나머지 gate·최종 custody는 아직 R162 완료로 계상하지 않는다.

이번 검증은 기존 저장 프로젝트와 기존 실제 원 미디어를 소비한다. source probe의 제품 SDKCLI 노출과 사용자의 프로젝트 작성 전체 여정은 아직 미완료다. card-review-export feature의 변경된 registry 구성 실제 회귀, VAD/card/clip 전체 동시 구성, 전체 실패/취소/attempt2/재시작/ACK, 자막/구간crop/preset catalog·색/회전/HDR/EOF·GUI/AI 사용자 품질은 남는다. direct production.render_short RPC는 Unsupported이며 jobs.submit 경로만 지원한다. CLI/typed 성공은 사용자 품질 승인과 GUI 시각 인수가 아니다.

기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)와 code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 fixture/원 Jobs/CLI·typed/기능·품질 근거를 분리했다.
