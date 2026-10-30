---
id: plan:kinekino/20261029113835
title: Accepted Card → PNG1 제품 template 계약 인계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Accepted Card → PNG1 제품 template 계약 인계

2026-10-24 UI 담당의 기존 승인 규격·소스 읽기 결과다. Media에 직접 전달했으며 새 GUI/Cargo/OS/수동 시각/제품 코드 작성0이다. renderer는 Media, accepted identity/SDK·CLI 정책은 SDK 소유다. 이 문서는 미정 template 값을 승인하거나 구현을 완료하지 않는다.

## 기존 값과 적용 한계

| 항목 | 기존 값·경로 | 출력 template으로 승격하면 안 되는 부분 |
| --- | --- | --- |
| 두 문구 | accepted-card-image-export-checkpoint 계획의 front/back 독립 slot binding. 원 accepted text/source_refs/position 보존 | front=제목/back=요약·headline/summary·두 페이지라는 자동 의미 대응 없음 |
| 공용 문구 group | cards-visual.md, artboard_text.rs: 가운데 group·가운데 wrap, 기본 W84%/H70% | legacy title/subtitle 공용 preview이며 두 출력 slot의 각 rect가 아님 |
| CARD 안전 영역 | page_marker::text_bounds: band=(24+4×2)/132×W, 중앙 rect=(0.84W,max(H−2band,1)) | 일반70% 높이와 중복 적용하지 않음. 정사각에서 H약51.515%는 preview 예약 공식의 결과이며 새 출력 safe margin 승인값 아님 |
| font fit | design_tokens.rs/artboard_text.rs: scale=min(W/320,1), 기본32/18×scale×factor, gap8×scale, factor1부터×0.9·최저0.5 | GUI 논리 단위이며 export px 기본/절대 최소 폰트 크기·개별 front/back style이 아님 |
| overflow | 원문 삭제/ellipsis 금지, 최소 factor에서도 넘치면 검토 안내. legacy painter는 bounds와 부모 clip 교차 | preview의 clip+안내를 실제 PNG의 무조건 성공/잘림 허용으로 승격하지 않음. 출력 fit·ink·glyph 검증 별도 |
| frame | artboard fitted_rect는 canvas contain. cover=true는 원본비율 유지 max(W/sw,H/sh) 중앙crop, false는 contain | cards.rs DesignSample은 aspect1/covertrue지만 “실제 카드 문서 아님” 표시. PNG1의 cover/contain 정책 승인이 아님 |
| 폰트 | typography.md: 공식 Pretendard v1.3.9 Regular400/SemiBold600, JP Regular fallback/OFL, 번들·OS/CDN 비의존 | GUI 폰트 설정과 출력 콘텐츠 폰트 설정은 별개. legacy artwork 둘 다 proportional이므로 UI heading600을 front 의미에 자동 적용하지 않음 |

원 경로: `.spec/sots/player/presentation/studio/{cards.md,cards-visual.md}`, `.spec/sots/player/presentation/shared/typography.md`, `src/presentation/components/{artboard.rs,artboard_text.rs,page_marker.rs}`, `src/presentation/design_tokens.rs`, `src/presentation/studio/screens/cards.rs`. 기존 r55 safe-area review는 지정 preview 표본의 근거이며 실제 output/임의 길이/비정사각 전체 인수가 아니다.

현재 font 파일은 assets/fonts/pretendard의 Regular SHA6D0AF5258997AEC7354A6E340FC2325BA321C410CA48B3AF858C8C3D6E92A324, SemiBold SHA5E1C548732AF70873103066C16E1369B9A8A871F0B38C321A1D5BC73E43CEA2D, JPRegular SHAB14698ED6D82737F45E918D079BCA6FC498A107BCB094B56C5229DBDDF3246D3이다. LICENSE.txt SHAB04538C9ABEC39A3DB75108CF0AF0FD9C77032FE8AA2CF38345B4D250E98E38E, LICENSE-JP.txt SHA6CB2ADCE04090E5E321168D5B47E12D1CB703DA64929A4F34DBA9162EB55EB98를 함께 보존한다. 이 기존 자산 identity를 Meari 공통 declared asset 계약으로 소비하며 검증 전 기존 자산을 삭제하지 않는다.

## PM에 제안하는 정확 gap

첫 고정 제품 template을 결정하려면 다음 선언이 필요하다. 현재 승인된 실제 값은 찾지 못했으므로 임의 수치·기본값을 만들지 않았다.

1. 제품 template ID/version/hash와 출력 width/height·원본 frame layer rect, contain 또는 crop·crop anchor.
2. 서로 다른 `front`/`back` 두 slot ID에 accepted text를 그대로 연결하는 binding 및 각 rect·z-order·alignment·safe area. legacy title/subtitle 이름을 의미 대응으로 사용하지 않는다.
3. 두 slot 각각 font asset/fallback/style·기본 크기/절대 최소 크기·line spacing과 fit/shrink 허용 범위. UI의32/18·50% preview 수치를 output px로 그대로 쓰지 않는다.
4. overflow/미지원 glyph/ink 미관측 시 명시 실패 또는 검토 필요 결과와 출력 허용 조건. box bounds 포함·font delivered bytes만으로 fits/coverage를 보증하지 않는다.

제안은 이 값들을 제품 소유의 명시 versioned 선언으로 먼저 동결하고 Media renderer에는 필수 입력으로 전달하는 것이다. 미정 값을 renderer가 선택하거나 accepted front/back을 재해석하게 하지 않는다. 원본 frame/문구를 보존하고 새 template 승인·출력 QA와 원 SDK 기능 인수를 구별한다.

같은 세션에서 읽은 modoo.today frontend-first-journey의 공용 디자인/제품 의미와 실제 인수 구분, test-fixture-and-mock-build의 DesignSample/실제 산출물 provenance 구분을 적용했다. 문서 인계만으로 새로운 style·template·출력 지원을 승인하지 않았다.
