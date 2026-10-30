---
id: plan:kinekino/20261029113732
title: 현재 세션 조율
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 현재 세션 조율

2026-10-20 KST. 중앙 PM 소유. 전체 목표 진행 중이며 기능 실행·제품 통합·실제 OS 동작·시각 일치·사용자 품질 승인을 구분한다.

## 담당과 실행 소유권

| 담당 | 현재 작업 | 경계 |
|---|---|---|
| PM | 근거 인수·의존성·단계 커밋·자원 조율 | root index 소유, push 없음 |
| Meari | source403 producer 검증 마감·소비자 호환성 계획 | 추가 실행 없음, root gitlink494 유지 |
| SDK | hidden Win32 probe r3·recovery r2 | 읽기 인수, Cargo/OS 미검증 |
| 자막 | 스킵 host/session SDK·CLI r3 exact17 | 현재 heavy/shared snapshot 배정 |
| UI | 실제 프레임 capture→PNG crop exact6 | 읽기 검토 완료, 실행 대기 |

공유 제품 snapshot은 PNG와 스킵 core r3를 포함하며 현재 host17-r3 통합 담당에 배정했다. Meari producer2699는 아직 overlay하지 않고 root gitlink494를 유지한다.

## 인수·커밋한 체크포인트

| 범위 | 실제 증거 | root commit |
|---|---|---|
| 품질 SDK wire | CLI10·독립96프로세스, 저장·호환·strict/fmt | 9c9846c |
| 공통 품질 제품 프레임 | ProductWindow4 포함27pass, root/bridge strict/fmt | b0c19f1 |
| NativeProposal 런처 | active2·기존ignored1, strict/fmt·player.exe 빌드; GUI 실행 아님 | ffc2187 |
| 자막 구간 package | 실제12pass·CLIplaceholder ignored1·strict/fmt | eaeb454 |
| SDK child + Meari494 | 35pass·root default/native 및 bridge strict/fmt; OS 미검증 | def8a2e |
| resource-runner r3 | 34pass·strict/fmt/build·실제2/4CPU Job·오류 거부 | 39025a9 |
| 자막 API/CLI | root 고유15pass·추가plugin3 별도, 최초 parity101 보존 | a6c6003 |
| 영상 정리 순서 | 정책6+기존child7/router12·native strict/fmt | a611f79 |
| PNG SDK/CLI | 신규21+기존18=39pass·양strict/fmt·실제CLI11회 원증거; ignored7 별도 | 4fa9396 |
| OP/ED 스킵 core | 수정 후15+기존control/status7=22pass·양strict/fmt | eb62672 |

최초 실패·수리 핀·잘못된0test 필터는 덮어쓰지 않는다. 반복 실행한 같은 시험을 통과 수에 중복 합산하지 않는다. root index는 비어 있고 무관한 미추적 파일은 stage하지 않는다.

## 현재 Meari gate

원 c21의 paint7 및 native 기본7은 실제 통과했다. 첫 window 실행은 trait import 누락으로 compile101/시험0이었다. import만 고친 r2(e913876)는 compile 후 실제16개 중3pass/13fail/103filtered, exit101/83756ms였다. 13개는 공통 TSX fixture의 무인자 함수가 현재 변환기의 typed Props 규격과 맞지 않아 실패했다. 원 로그와 핀은 보존하며 후속 strict/fmt는 실행하지 않았다. 원 launcher16624/child74308 부재 및 heavy 반환을 담당이 기록했다.

현재 입력은 독립 checkout runtime/development/meari-surface-validation-r3, pin2699a114342738c48a3a6a308b7daad179027098, source403 manifest SHA8D45A4D47BFBBC5FC7428A70B7DF40E7B84C52D83ECCC9194A4EB7BB7140F942다. PM이 실제 diff와 manifest 해시를 확인했다. fixture 한 줄만 typed Props로 변경하며 video 구조·정책·기대값은 유지한다. 무인자 함수 미지원은 별도 제한이다.

PM은 r3 window16 재시험에 새 build4 입장 성공 시 -j2를 명시 배정했다. 거부 시 child0 원receipt 보존 후 보고하며 자동 fallback은 없다. 통과하면 기존 checked_scene/doc/default·window·paint strict/fmt를 conservative2/-j1로 계속한다. 실제 핸들 terminal과 정리·반환 전 다음 heavy는 시작하지 않는다. root gitlink49443cf는 유지한다. OS focus Unknown과 실제 영상 배치 미지원은 해소되지 않았다.
## 다음 입력과 인수 조건

1. 스킵 host17: C5E8E4DF6F1B0B13DC4AFC2E13853599F9DC8333BF4D46D8D26CE3588B063AE6. registry/session owner·endpoint 수명·기본Unavailable·same binding one-pass 보존·PNG와skip owner의 builder 보존을 소스 검토했다. registry5+CLI3은 작성 수이며 실제 실행 전이다. 실제 native seek·host tick·durable 복구는 미지원이다. [계획](manual-skip-host-integration-scope.md).
2. capture→crop6: 497E2F6AB480258866EE1A20EBF08013EB497371457E0A85561EF65C1A65B792. 동일 검증 PNG bytes의 optional SHA를 capture receipt에 추가하고 기존 두 CLI 명령을 연결한다. legacy None·PTS·publication·deadline 및 기존 PNG 시험을 보존한다. 실제 기존 Sintel과 FFmpeg/FFprobe/CLI 해시를 고정한 연속 검증은 대기 중이다. [계획](snapshot-export.md).
3. SDK input/probe: 원 parent lease·nonClone slot·full window/child·fresh frame을 검증한다. 생성 단계의 구조 검사와 실제 접근 검증을 구분하고 실패 시 원 owner를 반환한다. hidden probe는 표시·route·media open 권한이 없으며 실제 OS callback/Destroy는 검증 전이다. [계약](native-video-surface-contract.md).
4. 최종 기본 합성: HWND는 호환 후보이고 parent CPU paint만으로 영상 위 전체 컨트롤 UX를 보증하지 않는다. 현재 배포 libmpv의 실제 API·ABI와 OpenGL/D3D11/공통 compositor 후보를 비교한다. master 문서만으로 제품 지원을 확정하지 않는다. [비교 계획](native-video-composition-comparison.md).

## 자원·검증 규칙

한 번에 heavy 하나. 기본 offline/locked/-j1/testthreads1, BelowNormal·2 logical CPU. 고정 실행기 runtime/quality-review/resource-runner-build4-r3/final-bin/resource-runner.exe, SHA EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D. 기존 canonical binary는 보존한다.

명시 배정된 새 build4 명령만 독립 부모에서 실행하며 CPU 두1초구간<50%·가용RAM>=16GiB·허용CPU>=4 입장 성공 때 -j2를 허용한다. 최초 SDK build4 요청의 CPU74.21875% 거부와 이후 명시2CPU 재배정은 원근거로 보존한다. 실행 만료를 종료로 간주하지 않고 같은 원 handle과 프로세스를 추적한다. terminal·owned 정리·명시 반환 후 다음 세션에 배정한다.

ChatGPT/Terminal·사용자 앱은 종료하지 않는다. Pages·VM·DPI·새 장시간 재생·AI 다운로드/추론·수동 GUI 시각 검토는 보류다. 실제 기능은 CLI/Rust로 검증하고 headless 성공을 시각 일치로 표시하지 않는다.

## 근거와 이전 기록

- runtime/quality-review/png-export-integrated-r1: source17/보호865·594 차이0, 원7receipt, PID25 부재, CLI원bytes·33worker joined·고정binary.
- runtime/verification/manual-skip-core-r2: source11/보호609 차이0, 원7receipt·14PID 부재, r3 실제15 재확인, 최초strict101 보존.
- [상세 인수 이력](coordination-history-2026-10-20-png-and-skip-core.md), [이전 SDK·runner 이력](coordination-history-2026-10-20-sdk-and-runner.md).

기능 정본은 .spec/sots, 결정은 .spec/adrs, 남은 실행은 .spec/plans, 원 관측은 각 gate 경로에 둔다. 이 문서는 현재 배정과 인수 조건만 요약하며 전체 제품·모델 품질 완료를 선언하지 않는다.

## Meari 최초 compile 실패와 새 입력

c21의 paint7과 native 기본7은 통과했다. window build4는 실제두1초CPU<16%·가용RAM>24GB 입장/affinity15·4logical·BelowNormal 뒤 Cargo compile101/26574ms였다. 원인은 engine/surface_slots.rs의 TElement trait import1 누락이며 시험은0개 실행이다. 원3receipt·6PID부재·source403불변과명시반환을보존했다.

새 r2 runtime/development/meari-surface-validation-r2 @e913876ec62a81bf7d7e8697765a2894b8c4111b, manifest02BF95876FF9EDCB888B9DAAC996AB5AFCEDCCB1570814E2D6CC1777C578961E는 import1줄만수리했다. PMdiff확인뒤 window16새명령에build4/-j2를재배정했고 admission거부는원child0보존후정지한다. 성공뒤나머지checkedscene/doc/기본windowpaintstrict/fmt는conservative2/-j1로명시배정했다. 실행중입력변경/자동fallback은없다. 이전c21의7+7행위와r2후속시험의범위를구분한다.

SDK Win32 probe r2 source11 D58120E9024DA783DE3C5F0114978ABEE0A30267558CC1983BD7E7DFC27F8DEE는 읽기전용 unsafe/OS계약 검토2개에배정했다. 실제rawArc callback/부분실패/원thread·generation/cleanup·metadata권한 경계를검토하며 Cargo·OS실행은아직없다.

## SDK Win32 r3 독립 검토

source11-r3 SHA D9B11EADCC2EE7666D0774BCDC8145A2DA776ACE7DD49F3E30B053FAE2E07E02를 별도 읽기 전용 검토에 배정했다. SetWindowLongPtrW LastError·설치 실패 raw Arc 회수·제거 실패 보존·재진입·원 WM_NCDESTROY 및 class 해제 재시도를 검토한다. 작성된 fault seam은 아직 실행 전이며 Cargo/OS/GUI 실행은 없다. detach 실패 후 자동 회수 및 Drop 실패 시 원 owner 영구 보존은 완료로 표시하지 않는다. root 반영 전 독립 검토와 실제 검증이 필요하다.
r3 window 실행은 fresh build4 입장 성공으로 시작했다. launcher101420/child96900, affinity15·4logical·BelowNormal, 가용RAM 약26.8GB이며 PM이 두 원 프로세스 생존을 확인했다. 아직 terminal이 아니므로 시험 통과로 집계하지 않는다.

SDK r3 독립 검토는 exact11/manifest 일치와 기존 userdata 설치실패 P2의 소스 수리를 확인했다. 새 확정 UB·조기 Complete는 발견하지 못했으나 실행 안전성 승인은 아니다. 제거 실패 raw Arc 보존과 최종 Drop의 owner 유실 제한은 남으며, production 조립 전 recovery owner 보유 경계의 별도 설계를 담당에 요청했다. Cargo/OS/GUI 검증0이다.

Meari r3 window 원101420/96900은 terminal0/105926ms, 실제16pass/0fail/0ignored/103filtered를 PM이 원 receipt/stdout으로 확인했다. 이는 r2 fixture 실패 수리의 실제 회귀 결과이며 producer 전체 인수·OS 영상 배치 완료는 아니다. 나머지 gate는 기존 conservative2 배정으로 이어진다.

SDK recovery 설계의 exact4 source-only 구현을 배정했다. 생성 전 owner 예약·실패/Unobserved 보유·미완료 shutdown의 registry 반환 및 stale key/용량/세대 소진 반환을 검토한다. 실제 probe/engine uniquelease 분리 및 production 조립은 별도 미완료다.

## 핵심 편집·품질 판정의 현재 범위

독립 읽기 감사와 실제 검증 정본 대조에서 accepted-output SDK 3명령은 이미 구현·한정 CLI 인수된 것으로 확인했다. [wire 검증](../../sots/verification/application/accepted-output-review-wire-2026-10-20.md)의 실제 unit6·fresh CLI10/96호출·저장16·기존 회귀 및 양feature strict/fmt는 합성 deferred 판정 증거다. 사용자 품질 승인·모델 성능·미디어 적용 성공은 아니다. 같은 endpoint를 재구현하지 않는다.

UI 담당에는 기존 inspect target/record/readback typed API를 소비하는 명시 판정 흐름의 exact source scope·상태·stale/NeedsInput/Unsupported/history NotChecked 표시 계획을 요청했다. capture6 입력은 보존한다. 소스 구현 배정 전 계획 단계이며 GUI/추론을 실행하지 않는다. SDK 담당에는 기존 계획 첫머리의 오래된 미구현 표기를 현재 인수 정본 링크로 보완하도록 요청했다.

품질 UI 읽기 감사의 정정: C21 원본 세 패널·공통 품질 컴포넌트와 product_window/mod.rs의 register_output_reviews, 명시 NativeProposal 런처2 실제 인수가 존재한다. 앞선 미연결 단정은 인수하지 않으며 UI 담당과 연구자에게 현재 기본 player 경로/명시 NativeProposal 경로의 도달성과 기존 회귀를 재대조하도록 요청했다. 이미 구현된 화면/API를 중복 제작하지 않는다. 사용자 수동 품질 승인과 기본 player 전환의 잔여를 구별한다.

Meari reservation 문서 예제는 실제 compile-fail2/0fail/3filtered, 원103952/91012 terminal0/79974ms를 확인했다. checked_scene4와 window16에 더해 이 범위가 통과했으며 paint strict는 별도 원55812/49416 실행 중이다.

UI 담당 재감사에서 C21 실제 연결을 확인했다. register_output_reviews Native8 → document_factory capability 검사/enable_output_reviews → window_bridge project_component_props → 원본 세 TSX의 ProposalOutputReview 흐름이다. 기존 NativeDOM4flow+영향23=27pass는 합성 기능 증거이며 수동 GUI/사람 품질 승인은0이다. 남은 차이는 기본 eframe 경로와 명시 NativeProposal 경로, 최신 렌더러 조합 인수 및 일부 mounted negative 회귀다. 신규 품질 화면 재구현 없이 기존 경계의 최소 후속 범위를 UI 담당이 정리한다.

host17 gate 계획을 검토했다. 기존 계획의 고유45 예상치는 실제 실행 수가 아니며 CLI evaluate/override 직접 공백을 발견했다. tests/manual_skip_cli.rs exact1의 최소 회귀 확장 source-only를 담당에 배정했다. 원 host17/patch 핀을 보존하고 새 source17-r3·test-only diff로 분리한다. 동일 endpoint의 revision/override 후 canonical 상태와 stale 거부를 검증 대상으로 삼고 실제 native seek 지원으로 확대하지 않는다. Meari 반환 전 Cargo/CLI/snapshot 변경은 없다.

paint strict 원55812/49416은 terminal0/13464ms였다. native strict 원37332/115716은 아직 terminal=false이며 PM이 child 생존을 확인했다. 전체 producer 인수와 root gitlink 변경은 아직 하지 않는다.

recovery4 manifest A7CF801D64E5EF11A5F5EF8885D781548643436C7F6C489314A8361C46A11909는 독립검토에서 일치했다. nonpanic 계약의 명시 오류 보유는 소스상 확인됐지만 construct가 slot을 제거하고 입력을 consuming factory로 넘긴 뒤 panic하면 상위 catch 후 빈 registry/shutdown Ok가 될 수 있다. 실제 OS 조립 전 필수 공백으로 분리했다. SDK 담당에 원r1 보존 및 exact4 안의 borrowed/two-phase r2 수리를 배정했다. 원 입력/Constructing 상태 보유와 panic 후 미완료 shutdown 회귀를 요구하며 consuming closure 바깥 catch만으로 복구했다고 표시하지 않는다. 실제 partial OS owner unwind 안전은 adapter 단계의 별도 검증이다.

Meari native strict 원37332/115716 terminal0/39381ms를 확인했다. window strict 원60040/106644 생존을 확인했으며 같은 실행을 담당이 회수 중이다. root HEAD eb62672/index 비어 있음도 확인했다.

Meari window strict 원60040/106644 terminal0/85519ms를 확인했다. 실제 명령이 사용한 candidate-bin 실행기와 중앙 final-bin 실행기의 SHA는 모두 EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D로 동일했다. 포맷·최종 보호핀/프로세스 정리·명시 반환은 담당 마감 전이다.

host test 확장 source17-r3 SHA203D85C372ED61FB2F4A9C90976FD6C543C0FF8DCA100CC48E0F7B233D96E4B3, test-only diff SHA5C39C2E5AFE7D831DEF4F1121C7FE45C63C4396029F9DAFB1FAA47248B070EE5를 읽기 전용 독립 검토에 배정했다. 작성 CLI4/37요청·전체46 예상은 실제 통과 수로 집계하지 않는다. 나머지16 원SHA와 shared/core patch는 보존한다.

## 현재 실행 자원 이관

Meari r3 final-observation의 source403 delta[]·8receipt terminal0·owned PID16 부재·heavy_slot_returned=true를 PM이 확인했다. r3 actual window16/checked4/doc2 및 paint/native/window strict·workspacefmt/paintfmt 인수 범위이며, 이전 c21 paint7/native7은 최종핀 재실행 증거와 구별한다. OS focus/표시/Destroy와 약한 owner drop 단독 회귀는 미완료다. root gitlink494는 유지한다.

host17-r3 source203D85C372ED61FB2F4A9C90976FD6C543C0FF8DCA100CC48E0F7B233D96E4B3의 exact1 추가 회귀는 독립 읽기 인수됐다. Meari 명시 반환 기록과 PID 부재 확인 뒤 subtitle 담당에 heavy/shared snapshot을 배정했다. root eb62672·기존Meari494 기반, 새producer2699 overlay 없이 before/보호핀→additive17→integratedpin/mtime→conservative2 실제 CLI·회귀·양strict/fmt 순으로 수행한다. root live/index는 PM 소유다.

recovery r2 source4 SHA4C6C554C23E93055045F97E288B5B0F10A0AFFAB4BE705CC90E20D5488A6F20C는 독립 읽기 검토에서 이전 panic→slot 유실→shutdown Ok 반례를 차단한 것으로 확인했다. borrowed input과 Constructing 보유·owner/partial 선행 Drop·wrong-phase 거부를 확인했으며 새 확정 결함은 없었다. Rust 시험8은 작성 상태이고 실제 OS partial의 unwind 보존·Constructing 회수·host 최종 Drop 방지는 후속 계약이다.

host17 담당의 실제 착수를 확인했다. runtime/verification/manual-skip-host-r3-r1에 before17.json과 protected-before.json이 생성됐으며 아직 실제시험 통과로 집계하지 않는다. PM은 Meari 최종관측 파일 SHA27F473FEB37A9E5F5740AFB85563DD15E4D92F2322676E5FB4A0893EFC69202E도 독립 확인했다.

추가 읽기 전용 조사로 현재 로컬 mpv/libmpv provenance·버전·해시·배포 pin과 실제 렌더 API 지원 근거를 확인하도록 배정했다. master D3D11 header 존재와 실제 고정 binary 지원은 분리하며 새 다운로드/실행을 하지 않는다.

host17 실제 core-host 원13320/88888 생존과 conservative2를 PM이 확인했다. integrated17은 eb62672/Meari494/newproducer2699false, baseline-drift[]다. 원 로그에서 unused host::* import 경고를 확인해 실행중 source는 유지하고 terminal 뒤 별도 import-only 핀으로 보존하도록 담당에 요청했다. 아직 실제 통과 수를 집계하지 않는다.

UI default-native-shell 계획을 PM이 검토하고 exact6 source-only 구현을 배정했다. 기존 web_ui_adapter 등록1과 player_shell/{mod,routes,state,availability,tests} 신규5로 한정한다. 기존 catalog/NoSelection/희망·관측 route/checked generation/dirty·pending·recovery veto 및 source·mount·backend 가용성만 모델링한다. 실제 mount·close·재생 권한은 외부 관측이며 기본main 치환·fixture·새SDK·OS호출0이다. capture6 입력과 gate순서는 보존하고 directfmt/diff 뒤 독립 검토한다.

host r3 core/registry 원13320/88888은 terminal0/112320ms, 실제20pass/0fail/0ignored/417filtered였다. 원경고를 보존하고 r4 source17 SHA5839C15B992C301B53D9CEBD3A0595DABF543BA3AF0B2FCF22D34EAA916D3597/importdiff8E1E36458D485BEF744E5EC598836A6295EB815152BA9024AE94977AD2606EBE의 중복glob 제거1을 PM이 확인했다. 새integratedpin 후 같은 gate를 계속하며 최종r4 core20 재확인은 고유합계에 중복하지 않는다.

SDK 후속 construction guard/strong probe host exact8 source-only를 배정했다. 원probe11r3/recovery4r2는 frozen 보존한다. 생성 전 partial OS owner 보유·실제 receipt 전 Complete0·incomplete shutdown 재보관을 구현하며 uniqueenginelease/wholeouterproxy는 별도다. directfmt/diff만 가능하고 Cargo/OS/공유snapshot은 아직 배정하지 않는다.

host 최종r4 core/registry 원115704/53384 terminal0/22496ms, 실제20pass/0fail/0ignored/417filtered를 확인했다. 원r3와 고유시험 중복 합산하지 않는다.

SDK partial guard/strong host 범위를 exact9로 확대했다. 기존 exact8에 win32/release.rs1을 추가해 생성 호출 중 Events가 HWND를 보유하지만 owner.hwnd가 아직 None인 경우 조기 NotCreated/class해제를 차단한다. 생성전 inert owner/pendingrawArc 보유, 원Events·호출반환 상태 동기화, inflight/unknown Complete0 및 추측회수0를 요구했다. 콜백은 atomic/wake만 수행하고 registry/mutowner 재진입을 금지한다. 원probe/recovery 핀은 보존하며 source-only/OS실행0이다.

Meari 라이브러리 담당의 소비자 호환성 계획을 읽기 인수했다. 현재 root494 소비자와 향후 SDK probe/recovery 후보를 분리하며 실제0test를 인수하지 않는다. 기존 weak-owner drop 단독 회귀의 exact1 test-file 보강을 source-only로 배정했다. 검증2699는 보존하며 유효commit/getter성공 직후 다른 invalidation 없이 strongowner drop→거부를 직접 단언한다. 라이브러리 범위 안에서 진행하고 추가Cargo/공유snapshot변경0이다.

host CLI 새 실행 receipt가 생성됐으며 원 child88484를 추적한다. 앞선 build-cli는 terminal0이다. 실제 요청/결과·소유프로세스 회수 전 CLI 통과로 집계하지 않는다.

host CLI 원34836/88484는 아직 terminal 전이며 원stdout에 개별 정상 응답과 의도한 revision_conflict exit5가 관측됐다. 전체37요청/4시험 통과로 확대 집계하지 않는다.

Meari owner-drop 보강은 별도 d669694a01a00f03f506980df127b8734b6a61d0/source403 EC65CAF2BD8B929B412411648D979FA6C205D9A42922BF24DCE2C40DF104EAF6 후보로 작성됐다. 기준2699를 보존하고 window surface tests exact1/+61의 유효commit→getter성공→ownerDrop만→Stale를 읽기 독립 검토에 배정했다. 실제 Cargo/OS 실행은0이다.

host CLI 원34836/88484 terminal0/122405ms, 실제4pass/0fail/0ignored/0filtered를 PM이 확인했다. stderr의 CliReceipt는37개로 계획 요청수와 일치하며 endpoint 실제IPC 증거다. 아직 CLI 고정해시/요청파일/모든자원/보호핀 및 후속회귀·strict 마감 전이고 합성 native-like port를 실제mpv seek 성공으로 확대하지 않는다.

Meari d669 owner-drop exact1은 독립 읽기검토에서 validcommit/getter성공/strongcount1→drop만→weak0/getterStale 인과와 다른scope필드 불변을 확인했다. 실행통과 및 실제OS 전체수명 인수는 아니다. SDK retention exact9 작성본은 별도 unsafe/계약 읽기검토에 배정했다.

host 통합 PNG API 회귀는 원8952/58296 terminal0/12334ms, 실제7pass/0fail/0ignored였다. host4/37통과 뒤 기존 CliRunner progress/poll 공개 API의 target dead-code 경고가 확인됐다. strict 실패 반복 대신 manual_skip_cli.rs exact1에서 기존 start/progress/poll/finish를 실제 호출 증거로 사용하는 r5 최소 수리를 배정했다. 새 타이머/retry/allow/helper정책 변경은 없으며 최종diff 읽기 확인 뒤 CLI4 재확인과 나머지 gate를 진행한다. 원r4로그는 보존하며 반복CLI37은 중복합산하지 않는다.
