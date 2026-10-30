---
id: plan:kinekino/20261029113533
title: 자막 채택 이후 Card SDK 합성 인계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자막 채택 이후 Card SDK 합성 인계

## 최신 상태 — 2026-10-24 canonical 반환·PM 커밋 이후

[canonical integration 원 SOT](../../sots/verification/proposals/accepted-card-canonical-integration-r1-2026-10-24.md)와 [원 slot-return](../../../runtime/verification/accepted-card-canonical-integration-r1/slot-return.json)을 직접 대조했다. 기본 full은1163 passed/0 failed/76 ignored, default all-target strict 통과, feature mapper exact1 및 기존 SDK/CLI parity2 통과다. feature all-target strict는 **명시 승인된 compositional reuse**이며 새 전체 feature 실행으로 표시하지 않는다. S62/alias4 복원·보호 감사 및 슬롯 반환이 완료됐다(status=returned, pending0/writer0, carry229). 자세한 원 FileID·PID reuse·하위 수명 한계는 원 SOT를 유지한다.

PM 커밋 `371ed79414698e49f6ee7444ec00d5e84a0cd1cc`(`feat(cards): export accepted review cards through shared SDK and CLI`)를 직접 확인했다. canonical46(source45+lock1)+PM ignore1의 이전 미커밋 상태는 이 커밋 전 이력이다. 원 integration SOT도 검증 반환 당시의 미커밋 관측을 보존하므로 이후 PM 커밋과 혼동하지 않는다. renderer actual-r2/root export actual-r1의 성공 PNG1+기대 오류9 근거는 유지하며 glyph·가독성·품질·템플릿 사용자 승인과 capability `available_unverified`를 승격하지 않는다.

원 SOT의 **default CLI PDB 미보존/해시 미관측**, **09 mapper build의 fresh RAM/GPU/disk/writer 기록 누락**은 PM이 인수한 예외로 남는다. 재구성·재실행으로 채웠다고 주장하지 않는다. 별도 Meari cleanup5 등 기존 예외도 그대로다. 아래20:13 상태와 그 이전 현재형·미실행·미커밋 문장은 선행 이력이며 최신 완료 상태를 대체하지 않는다. 이 갱신은 다른 제품 범위 완료나 새 실행 정책을 추가하지 않는다.

## 선행 상태 이력 — 2026-10-24 20:13 KST 근거 시점

[renderer actual-r2](../../sots/verification/proposals/accepted-card-renderer-actual-r2-2026-10-24.md)와 [root export actual-r1](../../sots/verification/proposals/accepted-card-export-actual-r1-2026-10-24.md)은 완료·통과했다. 두 exact actual 중 root CLI는10요청=PNG 생성 성공1+기대 오류9다. glyph·가독성·품질·템플릿 사용자 승인은 미인수이며 capability의 available_unverified를 유지한다.

PM 인계 기준 canonical46(source45+lock1)+PM .gitignore1은 적재됐으나 미커밋이다. 진행 중인 canonical-integration-r1의 [05-default-full 원 receipt](../../../runtime/verification/accepted-card-canonical-integration-r1/05-default-full.receipt.json)는 이 시점 terminal=true/exit0/645376ms다. 이 단일 receipt는 전체 후속 완료나 보호 반환을 뜻하지 않는다. 후속 strict/mapper/parity의 검증·최종 인수와 보호/복원 반환은 아직 미완료로 구분한다. 실행 중인 gate의 현재 상태는 [원 입장 기록](../../../runtime/verification/accepted-card-canonical-integration-r1/00-admission.json)과 원 증거를 따른다. 이 문서는 새 실행 배정·재시도·승인 gate를 추가하지 않는다.

아래 모든 기존 “현재”, “미실행”, “rootactual0”, “아직 미배정/미컴파일” 등의 문장은 각 선행 체크포인트의 이력이다. 위 근거 시점의 상태를 대체하지 않으며 원 실패·예산·배정·미검증 기록은 그대로 보존한다. 위 시점 이후의 완료 여부는 추정하지 않았다.

## 실제 출력 이후 통합 순서

현재 private SDK r3/renderer r5의 실제 버퍼 시험은 통과했으나 CLI 저장·최종 통합은 진행 중이다. 실제 CLI PNG와10요청 경계를 인수한 뒤 담당은 고정source27+renderer18 중 canonical과다른정확경로/원before·승인lockdiff를 준비한다. 기존자막15/core7·무관WIP는보존한다. PM은 .gitignore의 `!/packages/media-card-render/` 책임디렉터리한줄만검토하며 runtime/모델/폰트추가공개는없다. 패키지자체license결정과배포물notice는외부재배포잔여이고이단계에서임의선언하지않는다.

미실행 root application/inspection/mapper 시험8과 기존 API/CLI parity를 포함해 기본·feature 주입 차이를 검증해야 한다. 기본 전체회귀와 feature에서 추가로 달라지는 실제 범위는 SDK가 기존실행근거와대조해중복없이제안한다. narrow CLI pure8의성공을 rootbusiness8성공으로혼동하지않는다. 전체fmt/test/clippy 및 feature차이검증·최종감사후에만 PM이명시경로로stage/단계커밋하며push0이다. 이후 [채택된 빌드 기준](build-cache-checkpoint.md) 전환과 UI J4의새부모closure인수를 진행한다. 실제실패시기존단계에자동채택하거나이계획을실행승인으로쓰지않는다.

## 현재 상태: type-r2 CLI 빌드·목록 확인 / 최종 감사 대기

다음 실제 검증의 단일 책임 계획은 [실제 출력 입장](accepted-card-actual-admission.md)에 둔다. 이 문서는 source 합성·부모 보존과 채택 흐름을 소유한다.

2026-10-24 PM 직접 관측: effective61 `BF9D4328…`(SDK source27 + renderer r4 source18), private lock `B2E3D540…`에서 renderer default24/native24 통과(native ignored1 제외), root default 및 card-review-export all-target strict가 통과했다. `13-card-cli-no-run.receipt.json`은 terminal=true/exit0/190578ms이며 `14-card-cli-list`는 exit0/15ms·시험9개를 나열했다. Cargo JSON에는 feature normal player-cli(profile.test=false)도 존재하며 독립 사본과 해시를 기록했다. 목록은 시험 실행 성공이 아니며 최종 복원 감사는 아직 미완료다. 아래 r3/미컴파일 문장은 당시 체크포인트의 역사이며 현재 상태를 대체하지 않는다.

다음 실제 검증은 현 gate 정상 반환·산출물 pin·새 자원 입장 뒤 별도 배정한다. renderer는 native ignored exact1의 버퍼 PNG/폰트 검사, root는 accepted Card CLI exact1의 실제1080 PNG 새 파일 출판과 10회 요청 경계를 각각 검증한다. 전자는 제품 저장 시험을 대신하지 않는다. 후보 외부 예산은 root180000ms/cleanup10000ms, renderer60000ms/cleanup5000ms이며 root 내부10×15s 외 setup/postcheck가 있어 성공 시간을 보장하지 않는다. 동일 runner의 필수 child 인자는 renderer 단독 시험에서 artifact guard로만 동일 EXE를 지정할 수 있으며 CLI 자식 실행으로 보고하지 않는다. 최초 deadline·오류·EOF·강제종료 여부를 보존하고 자동 재시도하지 않는다.

canonical 채택 전에는 패키지 공개 경계/라이선스 검토와 .gitignore 책임 디렉터리 허용, full regression 및 단계별 명시 경로 커밋이 남아 있다. PNG 기술 성공은 글꼴 가독성·디자인 품질·사용자 승인이 아니다.

SDK r2 source 작성 체크포인트: source27 `83F47C8E…`/delta27 `EAA4B6F7…`·effective61/parent23/read-parent12를 읽기 전용 구현/fixture 두 검토가 대조했고 새 확정 P1/P2는 없었다. immutable target/position/source refs·동일 frame bytes·template pin·strict PNG before_pixels·원 ApiError·렌더 후 new-file publication과 subtitle 부모 보존을 source 수준으로 확인했다. 신규 실제 CLI fixture는10회 요청과 원 owned15s/reserve2 helper를 사용하므로 inner 최대150s에 setup/postcheck 비용이 추가된다. 외부 단일 예산·정확 선택1pass와 PNG/폰트 실제 근거는 아직 없다. glyph/사용자 품질·기존 parity helper 수명 한계·취소/timeout 미지원은 별도다. source 검토를 compile/실행 성공으로 승격하지 않는다.

읽기 전용 CPU 입력 조사(2026-10-24): SDK C warmS에는 renderer 패키지와 root member/optional dependency/card-review-export feature가 없다. 최종 SDK r2와 renderer r3 exact18, 새 local package의 lock resolution이 후속 최소 입력이며 아직 Cargo/타입 검증0이다. 기존 TTF3/OFL2와 product_styles의 CSS/SVG include 입력은 존재하고 폰트/라이선스 SHA가 일치해 추가 복사가 필요하지 않다. runtime font-root는 명시해야 한다. root default-members 밖 renderer는 별도 default24/native25 예정 분모를 실제 no-run/list로 확인하며 root 기본 시험으로 대체하지 않는다. 새 공개 패키지 채택 전 .gitignore의 책임 디렉터리 허용 한 줄·재배포/민감정보를 PM이 검토한다. runtime/.spec나 자산 전체 공개 해제는 없다. D Meari 미러 cleanup5와 이 C 입력을 혼합하지 않는다.

후속 renderer r3: Media HANDOFF를 PM이 읽고 source18 `F62852CC…`/changes3 `C253D9B4…`의 읽기 전용 교차검토(P1/P2 없음)를 인수했다. production/Cargo 변경 없이 module 등록·asset_contract·native_png 시험3만 보완했다. raw18/parent16 SHA일치와 직접 fmt3/patch 검사는 있으나 compile/list/actual은0이다. 예정25는 default24/native ignored1이며 실제 Meari+기존 폰트 바이트의320² 버퍼 검사는 root의1080² accepted Card 출판/CLI·glyph/사용자 품질 인수와 구별한다. 재기반화 담당은 `runtime/development/accepted-card-png-r3/HANDOFF.md`의 새 시험과 실제 CLI fixture 공백을 소비하며 현재 자막 회귀에 혼합하지 않는다. 자막 후속 API tests와 CLI parity 기대표 보완도 새 부모에 유지한다.

2026-10-24 연구 서브에이전트가 읽기만 조사했고 PM이 manifest 경로 교집합과 SHA를 직접 대조했다. subtitle r3 후보13 `7C253957…`, Card source22 `052A4C9D…`/integration38 `C67D2420…`는 같은 옛 부모에서 갈라졌다. 실제 겹침은6개이며 동일 완성1/합성5다. 아직 재기반화·빌드·실행하지 않았다. 현재 자막 actual 슬롯과 무관한 후속 계획이다.

| 경로 | 보존할 합성 책임 |
|---|---|
| application/proposals/mod.rs | subtitle_export와 card_export 선언 모두 보존 |
| application/proposals/output_review/schema.rs | FA57C293… 동일 target() 재사용; 재작성 불필요 |
| application/api/mod.rs | 자막 command/schema·store&&SubtitleStorage 투영 유지, Card optional port/defaultNone/with_* 소유 이동·등록 추가 |
| application/api/client.rs | 두 typed export 메서드 모두 기존 execute_typed 사용 |
| application/api/capabilities.rs | 자막 supported/unsupported와 Card available_unverified/unsupported를 각각 원 가용성에 따라 표시 |
| bin/player-cli/runner.rs | 자막 help/generic root 유지, Card pure command/schema 분기·feature port 주입 추가, 기존 snapshot/endpoint 보존 |

표의 앞 다섯 경로는 src/ 아래이며 마지막도 src/ 아래다. renderer16과 SDK22의 경로 교집합은0이다. 자막13 실제 인수·채택 후 새 root before를 고정하고, 독립 Card 신규 usecase/schema/adapter·strict PNG·renderer16·template를 배치한 뒤 중복5를 추가 hunk 단위로 합성한다. Cargo/assembly/infrastructure 등록은 새 부모와 비교해 추가하고 lock은 별도 검증한다. 새 합성 manifest와 시험 분모를 고정해 source 리뷰→CPU로 진행한다. Card22 전체 덮어쓰기는 금지하며 자막 domain 등록/projection2/usecase3/CLI 시험은 유지한다.

원 RenderFailure<ApiError> cause 직접 반환·단계 진단 분리와 decode_static before_pixels의 기존 crop 검사 위치/CRC·Adler·APNG·trailing·allocation 순서를 보존한다. immutable target, review_artifact와7개 limitation, cancel 미지원 경계를 유지한다. 새 available_unverified 상태를 기존 parity/schema/클라이언트가 정확히 소비하는지 합성 gate에서 확인하고 supported로 바꾸지 않는다. 구조 확대·새 정책·추가 실행 배정은 이 인계에 포함하지 않는다.
