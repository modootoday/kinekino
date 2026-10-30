---
id: plan:kinekino/20261029113621
title: 채택 Card의 실제 이미지 export 최소 체크포인트
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 채택 Card의 실제 이미지 export 최소 체크포인트

Status: active (PM 검토 제안),2026-10-24. 문서·정확 소스 읽기만 수행했다. 구현/새 정책/시험/Cargo/OS/모델/다운로드/공유 소스 편집0, 자막 projection source16B639/expected6 불변이다.

## Outcome

**한 개의 명시 채택 Card와 명시 선택된 원본 프레임 PNG를 기존 Rust offscreen 렌더 경로로 결합해 실제 PNG 한 장을 공통 SDK/CLI에서 내보내는 것**을 최소 체크포인트로 제안한다. 원 accepted front/back·source_refs·프레임·font/template identity를 결과와 연결한다. JSON은 provenance/편집 정보이며 카드 이미지 산출 자체를 대신하지 않는다.

완성 범위는 “선택 카드1장/PNG export”다. 전체 카드뉴스의 카드 순서 편집·표지/본문/끝장·플랫폼별 파생·JPG·묶음·게시문/alt 초안·편집 프로젝트·게시 성공을 모두 완성했다고 표시하지 않는다. 원래 YouTube 커뮤니티/Instagram 제작 목적은 이 잔여와 함께 유지한다. 단일 PNG이므로 기존 file publication의 새파일 계약을 그대로 사용하며 새 다중파일 원자 transaction을 만들 필요가 없다.

## Authority

| 범위 | 결정·책임 |
| --- | --- |
| CARD 승인 UX/자산·원문/수락 규칙 | 기존 사용자 승인 계약, 임의 화면정책 추가0 |
| accepted identity/공통 API·CLI/출력 게시 | SDK, PM 후속 배정 |
| 제품 카드 template·두 text slot/프레임 레이어 문서 | 제품 UI 담당 소유 선언/공통 디자인 계약, Meari 소비 지원 |
| Rust offscreen/layout/font asset host | Meari 공식 Rust API 소비, 제품 adapter/renderer port는 제품 담당 |
| 신규 package/public root/정확 feature·lock·실제 자원 | PM 별도 승인, 현재 제안 |

## Evidence

다음 정확 문서를 읽었다. [cards 생산 계약](../../../sots/player/application/production/cards.md)은 빈 template/선택 frame/cue→문구/layout→플랫폼 preview→순번 이미지 묶음, 원문 인용/요약 분리·근거, 공통 폰트·팔레트·넘침 탐지, 원본 재사용을 요구한다. [CARD 승인 UI](../../../sots/player/presentation/studio/cards.md) CARD-03의 1:1/4:5/3:4/9:16은 제안비율이며 공식 플랫폼 한도 보장이 아니다. CARD-09는 몰래 자르지 않는 넘침 처리, CARD-14/15는 PNG/JPG·순번·글꼴/픽셀 검사다. 이번 조사에서 최신 외부 플랫폼 규정을 조회하거나 숫자 한도를 새로 확정하지 않았다.

| 정확 소스/계약 | 재사용 가능한 사실 | 재사용으로 보장되지 않는 것 |
| --- | --- | --- |
| domain/proposals/mod.rs::Card | front/back/source_refs의 채택 문구·근거. output revision 안의 position으로 선택 가능 | front=제목/back=본문 또는 front/back=두 이미지라는 자동 의미 대응, 영구 card_id, 프레임/PTS/font/layout는 없음 |
| web_ui_adapter/proposals/capture_card.rs·card-copy-fields.tsx | 두 문구 필드를 별개로 편집, mutation은 기존 guarded adapter | exporter/이미지 파일 렌더 아님. 제목·요약으로 몰래 재명명하지 않음 |
| presentation/studio/screens/cards.rs, components/artboard.rs | 같은 Artwork/공통 text preview 및 contain 기하·UI painter, source-based preview | eframe::egui Ui painter를 headless file exporter로 재사용하는 API는 아님. toolbar도 일부 unavailable |
| cards-visual.md·공통 artboard_text | canvas/thumbnail 같은 문서, Pretendard·안전영역·넘침/원문 보존 | preview 84%/70%·50% auto-fit 수치는 실제 출력/사용자 최소font 정책을 대체하지 않는다고 명시돼 있음 |
| application/snapshot_export.rs, domain/snapshot_export.rs, infrastructure/snapshot_export/{mod,raster}.rs | 고정 SHA PNG·명시 PixelCrop·CRC/정적PNG/픽셀·바이트 bound·새 PNG 게시/출력 SHA. 기존 media_file_publication 사용 | crop는 scaling/text/layout/font/card renderer가 아님. PNG crop 성공을 카드뉴스 완성으로 부를 수 없음 |
| application/frame_capture.rs::CapturedFrame | 요청/실제 source PTS·PNG 크기·선택적 output SHA·회전·색공간 정보 | 구버전 receipt SHA가 없으면 이미지 identity를 자동 인증할 수 없음. 카드 source_refs가 frame PTS를 발급하지 않음 |
| meari-native/application.rs::Renderer, infrastructure/engine.rs::NativeRenderer | layout/bounds/rasterize→Vec RGBA, from_html_with_assets(VerifiedAssets), window/gl 없는 CPU Vello paint 경로가 존재 | 제품 Card template→전체 PNG exporter 연결은 아직 없음. bounds만으로 glyph/overflow 검증 전체가 된다는 근거 없음 |
| Meari docs/consumer-integration.md assets 계약 | AssetManifest의 path/hash/bytes/role/license·VerifiedAssets, 메모리 asset provider/외부 fallback 거부. Native font/image 전달·checked CPU paint 근거 | asset_report의 delivered bytes는 complete font shaping/browser parity 인증이 아니라고 명시 |

폰트는 `assets/fonts/pretendard/README.md`의 공식 v1.3.9 Regular/SemiBold/JP fallback·각 SHA·OFL 원문 계약과 `presentation/typography.rs`의 bundled 등록을 확인했다. 사용자의 후속 지시대로 Meari 공통 asset manifest/VerifiedAssets로 같은 source/version/hash/license를 소비해야 한다. 글꼴을 재다운로드/별도 복제하지 않는다. `infrastructure/fonts.rs`의 Windows malgun.ttf 읽기는 legacy 경로여서 새 export의 공통 bundle/fallback 계약으로 채택하지 않는다. [타이포그래피 정본](../../../sots/player/presentation/shared/typography.md)은 UI 폰트와 출력 콘텐츠 폰트의 책임도 구분한다.

기존 [PNG crop SDK·CLI 실제 기록](../../../sots/verification/application/snapshot-export-sdk-cli-2026-10-20.md)은 동일 SDK port/domain bounds/atomic no-overwrite publication의 원 실제 근거로 재사용한다. 기존 proposal/accepted-output 검증은 명시 채택/저장 불변성의 근거이며 image exporter의 새 결과가 아니다. 새 모델실험·실제 CLI·Meari 렌더 실행은 이번에 하지 않았다.

## Unverified

새 Card export operation/DTO/template/image renderer binding과 fresh CLI PNG는 미구현이다. 현재 Card payload에는 layout/template/font/frame/alt/project가 없으므로 raw front/back만 자동 배치해 승인된 카드뉴스 문서가 생겼다고 할 수 없다. caller가 선택한 제품 template의 **두 명시 slot에 front/back을 각각 binding**하는 규칙을 제품 담당/SDK가 제안·인수해야 한다. 기존 field의 의미를 headline/summary·학습효과로 재정의하지 않는다.

headless CPU API의 소스 존재는 확인했으나 root CLI feature/asset delivery/font shaping/정확한 text-overflow metric/PNG encoder 결합은 아직 검증하지 않았다. 특히 Renderer::bounds의 DOM box와 실제 긴 한글 glyph overflow가 같은지 확인되지 않았다. 필요한 metric을 얻지 못하면 “모두 표시됨” success를 꾸미지 않고 해당 adapter 계약이 미완료로 남아야 한다. 새 screenshot/GUI 자동화로 그 기능을 대신하지 않는다.

출력 canvas 크기는 명시 pixel 입력/기존 viewport·image 예산으로 제한해야 하며 플랫폼 공식 규격으로 승격하지 않는다. 최초 CLI fixture의 예제 square pixel 수는 제품 기본정책이 아니다. crop/contain 선택, 안전영역·minimum font/넘침 응답은 기존 승인 디자인의 제품 template 선언으로 명시돼야 하고 임의 shrink/ellipsis를 넣지 않는다. JPG는 root image feature의 PNG 경로만 보고 지원으로 선언하지 않는다.

## Phases

**단일 체크포인트: selected accepted card + pinned source frame → PNG1.** 세부 입력 제안은 다음과 같다(새 API 등록 아님).

- 기존 proposal/source/accepted output identity와 그 immutable output 안의 card position. source_refs와 actual accepted front/back을 읽으며 caller가 임의 바꾼 text를 “채택본”으로 표시하지 않는다. Accept/CAS를 새로 실행하지 않는다.
- 원 frame/crop PNG의 local path+exact SHA 및 기존 frame/crop receipt가 있으면 그 원 identity. 필요하다면 사용자가 이미 실행한 production.capture_frame/export_snapshot 출력을 소비하며 이번 exporter가 새 영상 PTS를 추정하거나 source_refs에서 capture를 자동 시작하지 않는다.
- 제품 소유의 고정 template version/hash·두 slot binding·명시 width/height·프레임 배치/크롭·font/style/안전영역. 공통 declared asset inventory(기존 frame PNG/font/fallback/license)만 메모리 provider로 전달한다. 모델 text는 plain text로 escape하며 HTML/CSS/remote URL을 임의 실행 입력으로 받지 않는다.
- 명시 새 PNG output_path. 결과는 PNG path/bytes/SHA/width/height와 used proposal/output/source/card position/frame/template/font identity·실제 overflow/미지원 글리프·손실/제한 관측이다. 부수 JSON은 이 receipt이지 완성 이미지의 대체물이 아니다.

소유 경계 추천: root domain/application이 기존 accepted Card/원문 연결과 명시 export 유스케이스·typed SDK/CLI·publication을 소유한다. 재사용 가능한 순수 렌더 adapter가 필요하면 **제안 신규 `packages/media-card-render`**가 Meari Rust Renderer/VerifiedAssets·고정 template에서 RGBA/PNG bytes와 layout diagnostics만 만든다. proposal store/CAS/모델/파일게시/제품 정책을 이 renderer나 Meari 본체에 넣지 않는다. root infrastructure가 기존 PNG/guard/publication을 연결한다. 실제 package 신설·manifest/public allowlist는 PM 결정이며 지금 생성0이다. 기존 player-ui-plugins/Meari 공통 플러그인 계약과 연결할 adapter도 제품 소유로 두고 CLI가 GUI를 조작하게 하지 않는다.

최소 실제 인수는 중앙 배정 후 **fresh CLI의 한 selected-card 여정**이다. 기존 이미 accepted fixture+실제 pinned frame PNG·공통 Pretendard asset/template로 export→독립 PNG decode로 dimensions/형식/bytes/SHA·원 frame의 식별 pixel 영역·두 문구의 layout/폰트 전달·원문/근거/페이지 identity 대응을 확인하고 실제 artifact를 열어 한글/줄바꿈·잘림을 검토한다. OCR/추론이나 새 UI 클릭 gate는 필요 없다. font delivered bytes만으로 glyph 검증을 대체하지 않으며, 불확실한 coverage/overflow는 명시 미확인으로 둔다. 이 원 artifact 검토는 이미지 출력 QA이고 제품 GUI 수동 시각 인수가 아니다.

같은 새 소비자 경계의 작은 오류 확인은 accepted identity 불일치·frame SHA 변경·긴 문구 overflow/미지원 glyph·기존 output 경로 충돌의 전달이다. 이미 검증된 Accept/CAS/PNG parser/publication의 전체 fault suite를 다시 만들거나 반복하지 않는다. 기존 source/generated/receipt·frame bytes는 불변이어야 한다. CLI 결과 파일은 원 crop PNG의 단순 복사/정적 demo thumbnail/JSON-only가 아니어야 한다. fmt/test/strict와 새 renderer/type/CLI actual은 별도 배정 전0이다.

후속 완성 잔여는 다중 카드/순번 파일·template 전환/undo·플랫폼별 layout revision·PNG/JPG·게시문/alt·편집 프로젝트 묶음과 품질 검토다. 첫 PNG1 성공으로 이를 완료 처리하거나 YouTube/Instagram 업로드 성공을 표시하지 않는다.

## Rollback

지금은 문서만 있어 코드/자원 rollback0이다. 후계 구현 시 새 export capability/adapter/template binding의 정확 목록을 철회하며 기존 proposal/store/snapshot/publication/common font assets는 보존한다. gate가 생성한 명시 새 PNG만 소유 inventory로 추적하고 사용자 원 frame·기존 파일·font/모델을 삭제하지 않는다. root index/commit/push는 PM 소유다.

modoo.today에서 다시 찾은 `modootoday/spec-authoring/plan-authoring`의 앞서 읽은 본문(측정·제안·권한·미검증·완료조건·복구 분리)을 적용했다. 이미 읽은 QA review-verification 원칙으로 “UI preview/crop/asset bytes 전달=카드 export 완료” 주장을 source 경로와 실제 기존 근거로 분리했다. 스킬 이름만 채택하거나 타 언어 렌더/검증 도구를 도입하지 않았다. source16B639는 변경하지 않았다.

## Public renderer API와 overflow 관측 경계 — 2026-10-24 추가

외부 소비자 호출은 `meari::application::assets::VerifiedAssets::new(manifest, blobs)` → `meari_native::infrastructure::engine::NativeRenderer::from_html_with_assets(&html, assets)` → `meari_native::application::Renderer` trait의 `layout(&viewport, 0.0)` → `bounds(id)` → `rasterize(&viewport)` → concrete renderer의 `asset_report()` 순서다. `meari_native::domain::Viewport { width, height, dark: false }`의 각 축 허용 범위는 1..=8192다. manifest descriptor의 `output_path()`로 얻은 content-addressed URL을 fixed template의 frame/font 참조로 사용한다. 기존 Pretendard와 pinned frame bytes만 공급한다.

제품 Rust adapter가 작성하는 trusted HTML/CSS에 front/back plain text를 escape해 각각 독립 text node로 넣는다. frame-layer, front-slot/front-content, back-slot/back-content는 제안 template ID이며 기존 SDK API가 아니다. front/back을 headline/summary 또는 두 페이지로 재해석하지 않는다. 고정 slot과 높이가 자동인 content box를 clamp/ellipsis 없이 layout한 뒤 bounds를 비교하면 layout box의 안전영역 이탈은 관측 가능하다. 고정 높이 content box의 포함만으로 모든 text가 맞는다고 판정하지 않는다.

공개 bounds는 x/y/width/height만 반환한다. scrollWidth, paragraph/glyph metrics, painted ink bounds, 완전한 glyph coverage 또는 전용 overflow API는 확인되지 않았다. box 포함 여부와 ink/coverage 미확인을 별도 진단으로 반환해야 한다. 긴 비분리 문자열의 수평 ink 돌출·fallback 정확성까지 fits=true로 확대하거나 자동 잘림/축소를 성공으로 숨기지 않는다. 완전한 fit 보증이 제품 조건이면 추가 관측 계약 또는 출력 QA가 남는다.

window/gl 없는 CPU 경로는 checked paint 뒤 RGBA Vec<u8>를 반환하며 PNG API는 아니다. 제품 adapter가 checked width×height×4와 byte budget을 검증한 뒤 Rust PNG encoder와 기존 publication을 연결한다. asset_report의 delivered/rejected/failures는 자산 요청 근거이며 shaping/글리프 적합성 증거가 아니다. 최대16 asset resolve rounds는 wall deadline을 보증하지 않는다. validate_asset_css/validate_asset_references는 pub(crate)이므로 외부 호출로 설계하지 않는다.

근거: packages/meari/packages/meari-native/src의 application/domain/infrastructure/engine.rs/infrastructure/asset_provider.rs, packages/meari/packages/meari/src의 application/assets.rs/domain/assets.rs/domain/viewport.rs, packages/meari/docs/consumer-integration.md의 현재 소스. 문서 제안이며 신규 package·제품 소스·Cargo·실제 render 실행0이다. 앞서 읽은 modoo.today plan-authoring과 QA review-verification의 제안/소스 관측/실제 검증 구분을 적용한다.

## 2026-10-24 중앙 private source 배정 후계

초기 문서-only 상태 뒤 중앙이 순수 PNG1 renderer의 private 구현을 별도 배정했다. 후보는 runtime/development/accepted-card-png-r1/HANDOFF.md에 보존했다. root packages 승격은 하지 않았으며 SDK/application/CLI 소유 파일도 변경하지 않았다. source15 37F624FD4FCE9C46EB464C2949554D3B25795D1C3E3A1BCAD41741FC28CCCD96, before15 94BE1D6D86DAD01812D82D3E9F2AAF8BA6DA77972487D0EC8B9EFBD16D1C5B75, delta15 BCD0055765656256F7D43A876E7C63C209849768A9E953DBB8B7961B462869C7, expected19 B0BD776E618EE4A8171BC62307283A99F9B1F8627A9E54CBB1F82FC4DBB2539A를 고정했다. Rust14 직접fmt/check 및 patch apply-check만 통과했고 authored19/type0/list0/test0/native render0이다.

호출자 명시 template DTO·immutable selection carry·VerifiedAssets/frame hash·PNG inspection 포트·bounded RGBA encoder·공개 Meari adapter를 책임별로 분리했다. font/glyph/ink/hostfallback/frame pixel/template approval와 cancellation/deadline의 미검증·미지원 범위를 typed limitation으로 유지한다. 기존 strict PNG parser는 root private라 후속 SDK adapter 연결이 필요하며 검증 로직 복제0이다. source16B639 및 root Cargo/lock/index 해시 불변을 확인했다. 향후 독립검토·중앙 타입/실제 CPU·SDK/CLI 연결과 출력 품질 인수가 남으며 PNG1/카드뉴스 완료 주장은 없다.
