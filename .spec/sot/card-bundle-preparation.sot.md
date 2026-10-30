---
id: sot:kinekino/card-bundle-preparation
title: 카드 묶음의 준비 입력과 렌더링 책임
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카드 묶음의 준비 입력과 렌더링 책임

2026-10-28 R105/R106 구현. [전체 카드 요구](cards.md), [프로젝트 저장](card-project-storage.md), [SDK/CLI 저장](card-project-sdk-cli.md)을 잇는 application 준비 단계다. 아직 production.export_cards·Jobs·실제 묶음 발행과 연결하지 않았다. 완료 범위를 축소하거나 이 단계만으로 전체 제작 완료를 선언하지 않는다.

CardBundleRequest는 저장 document ID·발행 revision·발행 SHA, 명시 frame artifact ID→PNG path/SHA 선택 목록, 새 출력 디렉터리, review_artifact_only를 가진다. prepare는 기존 CardProjectStore에서 지정 발행을 읽고 ID/revision/직렬화 hash를 대조한다. 단순 caller JSON의 document나 accepted 식별자를 출력 권위로 사용하지 않는다. 저장된 편집 문서와 source·카드 순서·역할·문구·인용/요약 구별·alt/게시문 초안을 PreparedCardBundle에 보존한다.

CardBundleAssets는 정적 제품 adapter가 구현할 port다. 템플릿/캔버스·공통 폰트 hash/라이선스·실제 PNG 관측·새 출력 경계를 검사한다. JSON 입력으로 adapter 구현이나 자체 asset 승인 결과를 공급하지 않는다. 현재 이 port의 실제 파일/제품 자산 adapter는 후속 구현이다. application 시험의 합성 adapter를 실제 PNG/폰트 검증으로 계상하지 않는다.

중복/외부 frame 선택·누락·저장 frame hash와 다른 선택·실관측 hash 불일치·사용하지 않는 추가 선택을 거절한다. 공통 폰트 hash와 라이선스 참조가 빠지거나 adapter가 다른 캔버스를 반환하면 실패한다. 현재 두 슬롯 템플릿은 비어 있지 않은 front/back/alt와 선택 프레임을 요구한다. 초안 문서의 빈 카드/빈 텍스트가 유효해도 출력 준비는 NeedsInput이다. 빈 템플릿·다른 캔버스·역할별 세부 레이아웃은 여전히 전체 카드 요구에 남으며 지원했다고 표시하지 않는다.

출력 예정 이름은 1부터 3자리 순번과 역할(cover/body/closing)을 결합한다. 예: 001-cover.png. 카드 ID를 파일 경로로 사용하지 않는다. 준비는 파일/모델/품질을 발행하지 않는다. 준비 후 실행에서 frame/template/font를 다시 고정/검증하고 취소·deadline·no-overwrite 발행을 Jobs의 원 publication gate와 연결해야 한다. 입력 준비 성공은 실행/발행·미디어 provenance·사용자 품질 성공이 아니다.

## 공통 픽셀 엔진

media-card-render에 CardText(front/back), TextRenderRequest, render_text, RenderedTextCard를 추가했다. 이 타입에는 proposal ID·accepted output revision/hash·프로젝트 ID가 없다. 기존 RenderRequest/SelectedCard/render/RenderedCard API는 보존하며 기존 render는 텍스트 픽셀 엔진을 호출한 뒤 원 SelectedCard를 결과에 그대로 결합한다. 픽셀 엔진이 fake acceptance를 만들지 않는다. CardScene은 텍스트 뷰만 받으며 layout/자산/PNG 검사·encoding·실패 cause·limitations는 동일 구현을 공유한다.

묶음의 출력 계보는 이후 저장 발행 hash/카드 ID/position/실제 frame·template·공통 자산 pins로 구성해야 한다. 기존 accepted CardReviewArtifact로 프로젝트를 위장하거나 accepted 원본을 자동 수정하지 않는다. rasterizer의 native 구현은 기존 Meari CPU buffer 경로를 사용하며 GUI 수동 시각 승인과 구별한다.

SDK application/domain/API/CLI, Media 원 입력/시간축, UI 표시/템플릿, Meari 독립 본체의 기존 소유권을 유지한다. 수정한 렌더러는 제품의 media-card-render package이며 독립 Meari 소스는 수정하지 않았다. private mirror·기존 WIP·원 증거·C-only/heavy1·WARP/추론/다운로드/수동GUI/push 보류를 유지한다.
