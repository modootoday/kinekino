---
id: plan:kinekino/20261029113733
title: Accepted Card 실제 출력 검증 입장
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Accepted Card 실제 출력 검증 입장

## 최신 상태 — 2026-10-24 canonical 반환·PM 커밋 이후

[canonical integration 원 SOT](../../sots/verification/proposals/accepted-card-canonical-integration-r1-2026-10-24.md)와 [원 slot-return](../../../runtime/verification/accepted-card-canonical-integration-r1/slot-return.json)을 직접 대조했다. 기본 full은1163 passed/0 failed/76 ignored, default all-target strict 통과, feature mapper exact1 및 기존 SDK/CLI parity2 통과다. feature all-target strict는 **명시 승인된 compositional reuse**이며 새 전체 feature 실행으로 표시하지 않는다. S62/alias4 복원·보호 감사 및 슬롯 반환이 완료됐다(status=returned, pending0/writer0, carry229). 자세한 원 FileID·PID reuse·하위 수명 한계는 원 SOT를 유지한다.

PM 커밋 `371ed79414698e49f6ee7444ec00d5e84a0cd1cc`(`feat(cards): export accepted review cards through shared SDK and CLI`)를 직접 확인했다. canonical46(source45+lock1)+PM ignore1의 이전 미커밋 상태는 이 커밋 전 이력이다. 원 integration SOT도 검증 반환 당시의 미커밋 관측을 보존하므로 이후 PM 커밋과 혼동하지 않는다. renderer actual-r2/root export actual-r1의 성공 PNG1+기대 오류9 근거는 유지하며 glyph·가독성·품질·템플릿 사용자 승인과 capability `available_unverified`를 승격하지 않는다.

원 SOT의 **default CLI PDB 미보존/해시 미관측**, **09 mapper build의 fresh RAM/GPU/disk/writer 기록 누락**은 PM이 인수한 예외로 남는다. 재구성·재실행으로 채웠다고 주장하지 않는다. 별도 Meari cleanup5 등 기존 예외도 그대로다. 아래20:13 상태와 그 이전 현재형·미실행·미커밋 문장은 선행 이력이며 최신 완료 상태를 대체하지 않는다. 이 갱신은 다른 제품 범위 완료나 새 실행 정책을 추가하지 않는다.

## 선행 상태 이력 — 2026-10-24 20:13 KST 근거 시점

[renderer actual-r2](../../sots/verification/proposals/accepted-card-renderer-actual-r2-2026-10-24.md)와 [root export actual-r1](../../sots/verification/proposals/accepted-card-export-actual-r1-2026-10-24.md)은 완료·통과했다. 두 exact actual 중 root CLI는10요청=PNG 생성 성공1+기대 오류9다. glyph·가독성·품질·템플릿 사용자 승인은 미인수이며 capability의 available_unverified를 유지한다.

PM 인계 기준 canonical46(source45+lock1)+PM .gitignore1은 적재됐으나 미커밋이다. 진행 중인 canonical-integration-r1의 [05-default-full 원 receipt](../../../runtime/verification/accepted-card-canonical-integration-r1/05-default-full.receipt.json)는 이 시점 terminal=true/exit0/645376ms다. 이 단일 receipt는 전체 후속 완료나 보호 반환을 뜻하지 않는다. 후속 strict/mapper/parity의 검증·최종 인수와 보호/복원 반환은 아직 미완료로 구분한다. 실행 중인 gate의 현재 상태는 [원 입장 기록](../../../runtime/verification/accepted-card-canonical-integration-r1/00-admission.json)과 원 증거를 따른다. 이 문서는 새 실행 배정·재시도·승인 gate를 추가하지 않는다.

아래 모든 기존 “현재”, “미실행”, “rootactual0”, “아직 미배정/미컴파일” 등의 문장은 각 선행 체크포인트의 이력이다. 위 근거 시점의 상태를 대체하지 않으며 원 실패·예산·배정·미검증 기록은 그대로 보존한다. 위 시점 이후의 완료 여부는 추정하지 않았다.

## 현재 배정

현재 상태는 첫 actual 실패 후 source 수리다. renderer actual-r1은 원exit101/nonforcedEmpty/ownershipcomplete, 첫 red sample mismatch였다. rootactual0, 원 로그 보존·보호delta0/EOF6/PID3부재/slot 반환을 PM이 인수했다. 원 ledger40113ms와 최종 stdout40081ms는 같은deadline의관측시점차이며 서로대체하지 않는다. renderer r5 source18 78DAFE39…는2x2고유texel을8x8solidquad로바꿔bilinear내부샘플전제를보완하고 exactRGBA/기존좌표/나머지assert를보존했다. SDK는 rootfixture의같은전제를확인해 private r3정확2leaf만수정중이다. 이새조합의CPU/actual은아직미배정이며 아래 r4실행배정은선행이력이다.

2026-10-24 PM은 type-r2 SOT EECDD064…와 slot을 전문 인수하고 아래 exact2를 SDK 단일 슬롯에 명시 배정했다. carry202 SHA F5505D4CAB1BBB04BAE9A456E56CE22D4B59F1754C2C148833DAFBA49A1BAC1B, returnedcache11 SHA953B10CBFFE7B2A67B740407373C2F8A7A4A29F8A11C3E5ED82860A9D4DB210F를 새 기준으로 한다. 원FI예외6·Meari cleanup5 예외는 유지한다. fresh 입장 전 실행0, renderer 첫실패 시 root후속0이다. 아래의 '실행 승인이 아니다'는 이 명시 배정 이전 준비 단계 기록이다.

## 범위와 선행 조건

SDK type-r2의 S62·alias4 복원 및 보호 감사와 SOT/slot/carry 인수는 완료됐다. 위 현재 배정에 따라 fresh 입장 후 단일 실행 슬롯을 사용한다. source 자동 변경·resolver·재컴파일·재시도 없이 고정된 산출물을 소비한다. Meari D cleanup5는 별도 예외이며 이 C SDK 입력의 정상 반환과 혼동하지 않는다.

type-r2 closeout 추가 관측: cache11 중 media-vad-worker build-script EXE2가 10B560FE…에서 EEED798B…로 바뀌었다. 새 mtime2026-10-24T09:18:04.4164995Z는 root default strict09:18:00~09:18:55 구간에 있고 stderr의 Compiling media-vad-worker와 일치해 PM이 이번 gate 쓰기로 귀속했다. 원 backup 핀 fresh 확인→새 producer 독립 보존→정확2경로 unlink/독립 복사로 원bytes/mtime 복원을 배정했다. 원 FI 차이와 최초 delta를 예외로 보존하며 아직 정상 반환으로 판정하지 않는다. all-cache 보호를 주장하지 않는다.

원본은 `runtime/verification/accepted-card-sdk-type-r2/14-fixed-root.json`, native renderer의 `09-fixed-renderer-native2.json` 및 Cargo artifact JSON이다. CLI SHA `69D6F5AC187E585BA4C7A4F0287E05C06E9531DC7265F57CE0A93D2847B4BCA4`/91438080B, Card test SHA `B97502F22D6DBEF943C80895F299A439C92582F0EBA843E9238BB53493759072`/4727296B가 현재 인계 핀이다. 최종 반환 manifest와 다시 대조한다. warm cache 경로가 아닌 독립 사본을 사용한다.

## 실행 분리

1. native renderer exact `tests::native_png::real_frame_and_pinned_fonts_render_decodable_png_with_explicit_limitations`: 기존 공통 폰트 TTF3/OFL2의 명시 절대 `MEDIA_CARD_RENDER_FONT_ROOT`를 사용한다. 버퍼 PNG 검사이며 제품 저장·accepted binding·1080 출력의 증거가 아니다.
2. Card CLI exact `accepted_card_review_cli_publishes_real_png_and_preserves_binding_boundaries`: 새 미존재 절대 `ACCEPTED_CARD_EXPORT_EVIDENCE_DIR`, 고정 `ACCEPTED_CARD_EXPORT_CLI`와 `ACCEPTED_CARD_EXPORT_CLI_SHA256`를 제공한다. 10회 원 요청·예상 exit·실제1080 PNG와 frame/template/accepted revision 불변·new-file 충돌/오류 경계를 원 fixture 그대로 검사한다.

각 단계는 새 증거 디렉터리와 정확 source/Cargo/산출물 baseline을 사용한다. runner baseline은 1MiB 이내라는 입력 제약만으로 신뢰되지 않으며 부모·소스·producer provenance를 따로 기록한다. type-r2에서 나열한9개 목록을 실제9개 통과로 보고하지 않는다. 기본 pure8과 ignored actual1의 실행 여부를 구별한다.

## 원 수명과 종료 근거

고정 runner는 `owned-test-runner-budget180-type-r1/RUNNER-READY.json`의 normal EXE SHA `642007FC51980FA1246693D3EEA5819A100866760B7DF34AB498A0572F6026C1`/916992B다. 제공된 `--test`, 필수 `--child`, `--baseline`, `--evidence`, `--case`, `--ignored`, `--total-ms`, `--cleanup-ms` 인자만 사용한다. 환경은 상속하며 없는 env 플래그를 만들지 않는다. renderer 단독 단계의 child는 동일 테스트 EXE의 artifact guard일 뿐 실제 CLI 자식 실행이 아니다.

후보 예산은 renderer60000/cleanup5000ms, root180000/cleanup10000ms다. root 내부10×15s에 동기 setup/postcheck가 추가되므로 잔여20s는 성공 보장이 아니다. 관측 시간초과로 실행을 다시 시작하지 않는다. 최초 deadline/owner/첫 오류·원 exit·raw EOF·family 종료·forced 여부를 보존하고, 강제 정리 후 Empty를 정상 성공으로 표시하지 않는다. 실패하면 원 근거를 보존해 소유 담당의 최소 수정으로 돌아간다.

## 반환과 후속

fixture 수리 후 재검증 원칙: Media r5 한 leaf와 SDK r3 두 leaf 외 production/Cargo/lock/부모가 그대로임을 먼저 대조한다. 새조합/산출물은 원r4 핀을재사용해성공으로간주하지 않는다. 변경된 native renderer와 Card CLI harness의 strict/no-run/list·fixedproducer를 갱신하고 실제exact를다시배정한다. 영향없는 default24/이미인수된순수회귀의반복을별도완료조건으로늘리지않는다. root application8 및 전체회귀는앞선type검사만받았으므로최종통합에서실제시험해야한다. 새로운실패/범위변경이있으면검증범위를다시판단한다. 구체실행은새후보HANDOFF인수후실행배정정본으로확정한다.

공개 경계의 별도 근거는 [Media 소스 검토](../../sots/player/infrastructure/card-render/public-source-review-2026-10-24.md)(SHA C7D1B4D18AABFC2D3621484B0604F10B18903141B9824CBD5FAC2D0DBC9ACE65)다. exact18은 authored Rust/Cargo와 합성 fixture이며 개인 미디어·모델·폰트 바이너리가 없다. 자체 license 미선언과 실행물별 전이 notice는 외부 공개/재배포 결정에 남긴다. 내부 actual을 중지하거나 임의 MIT/Apache 선언을 추가할 근거로 사용하지 않는다.

실제 선택1pass·산출물 경로/해시/PNG decode 및 fixture assertions, 보호 대상 delta·writer0·원 identity 종료를 반환한다. glyph 가독성·미학·template 승인·실제 방송 프레임 PTS·AI 품질은 이 fixture의 성공으로 승인하지 않는다. 이후 full regression·공개 경계 검토·canonical 채택·명시 경로 커밋을 별도 수행한다.

근거: 기존 runner READY와 type-r2 Cargo producer/list를 PM이 직접 읽었다. 읽기 전용 CPU 예산 연구의 원 deadline/guard/환경 제약을 반영했다. 구현·검증 책임 분리와 단일 통합 보고는 `%USERPROFILE%/.codex/skills/parallel-agents/SKILL.md`의 소유권/합성 원칙을 적용하되, 해당 문서의 타 제품 전용 도구·모델 예시는 채택하지 않았다.
