---
id: sot:kinekino/card-text-templates
title: 텍스트 전용 카드의 제품 연결
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 텍스트 전용 카드의 제품 연결

R120 [실제 검증](../../../verification/application/card-text-templates-20261028.md): 텍스트전용3종을 원CLI Jobs에서 발행하고 PNG크기/원본frame없는 bundle/provenance null·원EOF를 확인했다.13CLI요청/완료6작업과 원R115descriptor2개의 SHA/재직렬화bytes 동일,양strict/fmt 통과. 기본 gate no-run 진행 중이며 전체190/정상bins/custody는 아직 미완료다. 아래 구현 중 표기는 최초 작성 시점이다. 역할별/페이지번호·부분실패·GUI/사용자품질은 계속 남는다.

R118 구현 중. 검증된 [프레임 없는 renderer](card-frame-free-rendering.md)를 공통 카드 프로젝트/Jobs 경로에 연결한다. 별도 blank_templates.rs에 card.text-square1080×1080/card.text-portrait1080×1350/card.text-landscape1920×1080를 제안 버전0.1.0-proposal.1로 추가한다. 기존 product_template.rs/canvas_templates.rs bytes는 유지하며 새 선언SHA는 두 원 선언+새 blank 선언 bytes를 순서대로 해시한다. 플랫폼 규격/사용자 승인은 아니다.

BundleTemplatePin.frame_required는 과거 descriptor에서 생략된 경우 true다. true는 직렬화에서도 생략하여 기존 준비 descriptor bytes를 보존하고,false인 새 텍스트 템플릿만 명시한다. 카탈로그 소비자는 생략=true를 따른다. PreparedBundleCard.frame/observation은 optional이며 기존 object값은 Some으로 동일 직렬화한다. 새 프레임 없는 카드만 둘 다 null이며 불완전한 쌍은 거절한다. 기존 Jobs record/blob/hash를 변환하거나 덮어쓰지 않는다.

프레임 필수 템플릿의 기존 정책을 유지한다. 텍스트 전용 템플릿은 저장 card.frame=None을 요구하고 frames 배열의 사용하지 않은 PNG도 거절한다. front/back/alt 초안은 출력 전 채워야 한다. 시작용 빈 프로젝트/문구 편집과 내용이 없는 최종 산출물은 구분한다. Jobs는 원본 프레임이 없으면 복사/가짜파일 생성 없이 PNG·provenance·게시문·alt·편집project·manifest를 발행하며 source_frames=[]다. 기존 제안4종+새3종은 같은 catalog 포트로 제공한다.

preimage-r117-blank-product에 기존9파일을 보존했다.2075fmt종료0,2076input-r118 snapshot 진행 중. 기존 실제 CLI 여정에 새 텍스트전용 프로젝트의 typed저장/CLI jobs.submit/원host polling·manifest 확인을 추가했으며 completed_total은4(기존 카드2+텍스트전용1+자막1)로 검증한다. 아직 컴파일/실제시험 통과로 계상하지 않는다. 역할별 배치·페이지번호·부분retry/cleanup/ACK·GUI/사용자품질은 남는다. 최신 완료 제품gate R115/renderer기반 R117,기존 WIP/소유권/원증거/private overlay·보류 유지.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 optional의 부재와 가짜 미디어를 구분하고 원 Jobs 수명/발행 정책을 재사용한다.
