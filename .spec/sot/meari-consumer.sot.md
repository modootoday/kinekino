---
id: sot:kinekino/meari-consumer
title: Meari 제품 consumer의 폰트·아이콘
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Meari 제품 consumer의 폰트·아이콘

2026-10-19 사용자 추가 지시를 적용한다. Meari는 공식 Rust/React SVG 팩 기반 내장 Icon 카탈로그·Pretendard 공통 폰트와 provenance/license·resolver를 소유한다. 제품은 안정 의미 role·font token과 SDK 상태를 소비한다. 제품 로고/시작 표식 등 전용 커스텀 SVG는 assets/design에서 계속 관리한다.

현재 요구 목록은 src/presentation/web_ui/assets/requirements.json이다. 이 JSON의 role은 제품 의미 ID이며 아직 등록된 Meari catalog ID가 아니다. 공식 namespace/pack/revision·매핑·지원 진단은 담당이 인계해야 한다. 잘못된 ID를 빈 glyph나 폰트문자로 대체하지 않는다. caption glyph12/hit42×32와 content glyph20/hit32를 분리하며 최소화/최대화/복원/닫기 형태를 모두 비교한다.

기존 Pretendard1.3.9 Regular400/SemiBold600·JP400와 OFL 전문·SHA는 assets/fonts/pretendard/README.md가 기록한다. Meari resolver로 실제 font delivery와 같은 weight/fallback이 검증되기 전 프로젝트 파일을 삭제하지 않는다. 새 중복 다운로드·런타임 CDN·시스템 폰트 의존은 도입하지 않았다. JP fallback을 다국어 전체 지원으로 확대하지 않는다.

현재 Web CSS의 @font-face/기존 SVG mask는 전환 전 원본 요구를 보존한다. 내장 카탈로그 API 도착 후 같은 source를 resolver 소비자로 변경하고 bundle·font glyph·색/stroke·accessibility name·focus/hover를 재검증한다. CSS mask 선언/요구 목록 작성만으로 asset delivery나 시각 일치를 완료로 표시하지 않는다. 공식 팩 선택/라이선스 인수는 Meari 담당 범위이며 제품에서 별도 팩을 수집하지 않는다.
