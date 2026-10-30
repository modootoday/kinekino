---
id: sot:kinekino/card-canvas-presets
title: 카드의 명시 캔버스 프리셋
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카드의 명시 캔버스 프리셋

R113 실제 검증 완료: [새3프리셋 렌더/기존4시험·통합gate](../../../verification/application/card-canvas-presets-20261028.md). 아래 구현 중 표기는 최초 작성 시점이며 실제 GUI/사용자 시각 승인은 아니다.

2026-10-28 R113 구현 중. 승인된 [카드 요구](cards.md)의 캔버스별 재배치에 대응한다. 플랫폼의 최신 업로드 규격이나 사용자 시각 승인을 주장하지 않는 명시 제안이다.

원 product_template.rs와 card.frame-two-slot/0.1.0-proposal.1의 선언 bytes는 수정하지 않는다. 별도 assembly/card_export/canvas_templates.rs에 card.frame-square(1080×1080),card.frame-portrait(1080×1350),card.frame-landscape(1920×1080)를 등록한다. 새 선언 SHA는 원 선언과 새 선언 bytes를 순서대로 SHA256한 값이며 version은0.1.0-proposal.1이다. 기존 저장 canvas의 ID/version/SHA와 실제 크기는 계속 검증한다.

정사각형은 기존 배치,세로형은 프레임/제목/본문을 세로로 재배치,가로형은 프레임 왼쪽·문구 오른쪽으로 재배치한다. 프레임은 기존 contain,공통 고정 폰트·팔레트·overflow 거절을 재사용한다. 이미 렌더된 이미지를 비율 왜곡으로 확대하지 않는다. SDK의 전체 편집 문서에 canvas를 저장하고 동일 publication/Jobs 경로로 렌더링한다. CardBundleFiles::proposed_canvases는 현재 구체 adapter 카탈로그이며 공통 API discovery 소비는 아직 남는다.

이번 변경은 프레임을 가진 카드의 캔버스 확장이다. 프레임 없는 빈 템플릿,역할별 배치/페이지 번호,GUI 미리보기,사용자 시각 승인은 여전히 남는다. 기존 유효한 출력·핀·소유권/WIP·보류를 유지한다. 추가 실제 시험은 세 프리셋의 정적 PNG 전체 파싱/출력 크기/원 publication 보존/변조 핀 거절을 검증하며 buffer 결과를 제품 발행으로 계상하지 않는다. R113 snapshot 및 컴파일·실제 시험은 진행 중,최신 완료 통합gate는 R112다.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 선언 검사와 실제 CPU 렌더 검증을 분리한다.
