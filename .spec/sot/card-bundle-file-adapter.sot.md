---
id: sot:kinekino/card-bundle-file-adapter
title: 카드 묶음의 실제 파일·공통 자산 어댑터
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카드 묶음의 실제 파일·공통 자산 어댑터

2026-10-28 R107 구현. [준비 계약](card-bundle-preparation.md)의 CardBundleAssets와 CardBundleRasterizer를 CardBundleFiles가 구현한다. 기본/native 구성에는 주입하지 않으며 기존 card-review-export 제품 구성에서만 컴파일한다. 아직 Jobs/production.export_cards·묶음 발행은 연결하지 않았다.

템플릿은 현재 제품 소유 PNG1 선언의 id/version/선언 파일 SHA 및 1080×1080 canvas와 정확히 일치해야 한다. current_proposed_canvas는 그 제안의 식별 값을 제공하는 편의 함수이며 사용자 승인/자동 선택이 아니다. 저장 문서의 다른 템플릿·이전 선언 pin·다른 캔버스를 새 값으로 자동 치환하지 않는다. 공통 폰트는 기존 prepare_product_styles의 검증된 자산에서 읽고 PNG1의 Pretendard 1.3.9 세 hash/라이선스 참조와 대조한다. 모델/폰트/자산 다운로드·OS 설치·전역 설정 변경은 없다.

PNG는 기존 로컬 경로 읽기/바이트 한도·SHA 대조와 root snapshot_export의 엄격한 정적 스트림 검사(CRC/Adler/APNG·trailing 제한)를 재사용한다. 새 parser를 만들지 않는다. 실제 bytes/크기/hash 관측을 PreparedCardBundle에 보존한다. 입력 path의 존재만으로 image/PTS/권리·품질을 승인하지 않는다.

새 출력 디렉터리는 명시적인 일반 로컬 절대 경로·기존 parent를 요구하며 기존 immutable_storage_paths의 ancestor/reparse guard를 적용한다. 상대/상위 이동/네트워크·ADS/모호한 끝 문자 이름을 거절한다. 기존 항목은 파일/디렉터리/링크 여부와 관계없이 충돌이다. 이 단계는 조회만 하며 출력 디렉터리를 생성·예약하지 않는다. guard는 검사 호출 동안만 유지되므로 후속 발행 시 다시 guard·원 Jobs publication gate·원자적 no-overwrite를 확보해야 한다. 사전 검사만으로 race 없는 발행을 보장하지 않는다.

render_card는 고정 publication의 문서 ID/revision/hash·문서 유효성·카드 수·position/ID·예정 파일명·저장 frame pin을 다시 대조한다. 실제 template/common font admission을 재구성해 준비 값과 비교하고 PNG를 재검사한다. 준비 후 frame 변경은 SourceExpired다. RenderedBundleCard는 PNG 버퍼와 provenance만 가지며 파일 발행/완료 receipt가 아니다. output_directory를 생성하지 않는다.

기존 accepted CardReview API와 새 프로젝트 렌더링은 render_content의 같은 TextRenderRequest/strict PNG/폰트/CPU rasterizer/encoder를 호출한다. accepted entry는 원 application이 검증한 카드 내용을 제공하고, 프로젝트 entry는 고정 발행 내용을 제공한다. 프로젝트 renderer가 accepted target/제안 ID·revision을 제조하지 않는다. 프로젝트 provenance는 card_project pin·document edit revision·card ID/position을 기록하며 quality_approved=false와 기존 renderer limitations를 유지한다. front/back·frame·font pin·원 에러의 의미를 바꾸지 않는다.

남은 것은 Jobs prepared source blob/재시도·취소/검증 gate·PNG 묶음/문구/alt/프로젝트/manifest의 실제 발행과 정상 CLI 인수다. 빈 템플릿·다중 캔버스·역할별 디자인·overflow/glyph 검토·GUI 시각 인수·사용자 AI 품질 요구를 유지한다. 실제 파일 adapter 성공을 전체 카드 제품 완료로 확대하지 않는다. 기존 WIP·소유권·원 증거/private overlay·C-only/heavy1·WARP/추론/다운로드/수동 GUI/push 보류 유지.
