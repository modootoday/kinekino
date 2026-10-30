---
id: sot:kinekino/card-page-rendering
title: 카드 페이지 번호의 공통 렌더 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카드 페이지 번호의 공통 렌더 계약

R121 [실제 공통renderer 검증](../../../verification/application/card-page-renderer-20261028.md)27시험·실제CPU3·native strict/fmt·사후2364입력동일 완료. 제품 역할별 템플릿/Jobs/SDK·CLI는 후속이며 아래 구현 중/미실행은 최초 작성 시점이다.

R121 구현 중. 페이지 번호를 원문 front/back에 합치지 않고 별도 PageFooter(current,total,TextSlot)로 전달한다. 번호는1..=total이며 formatter는 current / total을 표시한다. slot은 양수크기/캔버스내경계/명시font·line-height가 필요하고 front/back 및 존재하는 frame과 겹치지 않아야 한다. 기존본문 bytes·프로젝트 원문/근거를 바꾸지 않는다.

render_with_page는 additive API이며 기존 RenderRequest/TextRenderRequest/OptionalFrameRenderRequest 호출을 유지한다. CardRasterizer의 기본 page 메서드는 page가 없으면 기존 rasterize를 호출하고,page가 있으면 명시거절한다. 미지원 adapter가 페이지를 조용히 누락하지 않는다. Native는 같은 실제 layout/raster에서 page-slot/page-content bounds를 관측하고 엔진이 containment를 검사한다. 페이지 여부·관측이 다르거나 overflow/측정불가는 완료PNG로 반환하지 않는다. footer텍스트도 공통 text byte 한도에 포함한다. font/palette/asset/PNG/원errorcause 정책은 재사용한다.

preimage-r120-page-renderer에 기존9파일을 보존했다.2100fmt종료0,2101input-r121 snapshot 진행 중이다. 범위/overflow경계/겹침·미지원adapter 거절과 실제CPU footer영역 ink/bounds/원문불변 시험을 작성했으며 아직 실행하지 않았다. 제품 역할별 템플릿/페이지총수·SDK/CLI/bundle/provenance 연결은 다음 단계다. 기존7템플릿 선언을 수정하지 않고 별도 버전의 역할별 선언으로 연결할 예정이다. R121 전체검증 미완료,최신 완료제품gate R120.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 adapter미지원과 실제bounds를 분리한다. 원AI9 NeedsRepair/사용자승인0/9·기존WIP/소유권/private overlay·보류 유지.
