---
id: sot:kinekino/card-role-templates
title: 역할별 카드 배치와 페이지의 제품 연결
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 역할별 카드 배치와 페이지의 제품 연결

R123 [실제 CLI/통합 검증](../../../verification/application/card-role-templates-20261028.md) 완료:6family×3role18PNG·19CLI요청/완료12·role/page/크기/frame/manifest·원descriptor2개bytes동일·기존회귀·양strict/fmt·지정190/정상2bins·사후2366입력동일. 아래 구현 중/첫실패는 당시 기록이다. 실제시각/사용자품질·부분실패 인수는 계속 남는다.

R122 구현 중. 기존 product_template/canvas_templates/blank_templates 선언 bytes는 유지하고 role_templates.rs를 새로 추가한다. card.editor-frame/text × square/portrait/landscape인6제안 family를 version0.1.0-proposal.1로 제공한다. 원3선언+새 role선언 bytes의 SHA가 새 핀이며 기존7종의 핀/저장 descriptor는 변환하지 않는다. catalog는13종이다.

새 family는 CardRole의 cover/body/closing에 따라 별도 layout DTO를 만든다. 공통 폰트/팔레트/프레임fit을 재사용하고 cover는48px semibold/center,body는40px front/36px back·start,closing은44px semibold front/center다. 각 canvas의 문구영역과 페이지영역을 다시 배치하며 전체PNG 왜곡확대가 아니다. 긴 문구 overflow/미측정은 기존 공통 엔진이 거절한다. 사용자 시각/품질 승인이 아니다.

BundleTemplatePin.role_layouts_and_pages는 default=false/false직렬화생략으로 과거 descriptor bytes를 유지하고 새 family만 true다. 실제 role/current/total은 고정 publication의 card역할·순서·총수에서 구한다. caller가 페이지텍스트나 임의markup을 입력하지 않는다. document0..127위치를1..128로 표시하며 번호는 독립 PageFooter로 렌더한다. slot은 아래64px 위치/공통font28px·line40px이며 앞/뒤문구·존재하는frame과 겹치지 않아야 한다. 새 family를 원 프로젝트 문맥 없이 직접 PNG export로 실행하면 NeedsInput이다.

새 PNG provenance에는 card_role/page선언/page_layout 실제관측을 추가하고,기존 family의 provenance/문구/핀은 유지한다. 원 project manifest/Jobs 발행/취소·검증정책을 재사용한다. 실제CLI 시험은6family 각각3role카드를 만들어18PNG의 크기·서로 다른layout SHA·role·page1/3,2/3,3/3·frame정합성·manifest entry hash와 원EOF를 확인한다. baseline 포함19CLI요청/완료12작업을 기대하며 아직 실행하지 않았다.

preimage-r121-role-templates의 기존7파일을 보존했다.2108fmt는 새 nested시험모듈 경로 해석으로 실패했으며 preimage-r122-role-module-resolution에 원파일을 보존하고 명시path로 수정했다.2109fmt종료0,2110snapshot 진행 중. R122 컴파일/실제CLI/strict/190/정상bins/custody 미완료,최신 완료제품gate R120/renderer R121. 부분retry/cleanup/ACK·나머지제작·GUI·사용자품질과 기존소유권/WIP/private overlay·보류 유지.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 선언/실제layout·원문/표시번호·원 descriptor를 분리한다.
