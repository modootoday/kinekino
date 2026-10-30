---
id: evidence:kinekino/20261029113849
title: SEG/SUB/OBJ 공통 미리보기 r160
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SEG/SUB/OBJ 공통 미리보기 r160

2026-10-19. [설계](../../player/presentation/shared/media-preview.md)에 따라 raw artboard+작은 별도 toolbar를 기존 media_preview 조합으로 연결했다. 공통 callback은 artboard::fitted_rect로 계산한 contain 영역을 전달하며 ROI·마스크·도구는 그 영역을 사용한다. 기존 unavailable 소비자는 빈 callback을 사용한다. 정책·SDK·실제 재생 가용성 변경은 없다.

## 기준 비교

skip-settings.png 전체와 transport(275,510,894,55)3배 파생물 `runtime/review-runs/seg-transport-r160-3x.png`를 확인했다. 기준은 검은 영상 프레임과 하단 일체형 transport, 작은 둥근 경계다. 기존 CHROME 측면 빈 공간과 작은 단독 텍스트 도구를 VIDEO_SURFACE 면·THIN/BORDER·CONTROL radius·하단 CHROME transport로 바꿨다. 시안의 활성 재생/볼륨·시간·설정 버튼은 기능 근거 없이 복제하지 않는다. 현재 공용 transport의 아이콘 순서·시간/seek 순서·볼륨 rail은 시안과 여전히 차이가 있다. 실제 재생 미연결과 시각 완성도를 구분한다.

## Rust 검사와 실행

session11297 exit0: fmt --check, 전체 test141passed/9ignored(lib96passed/1ignored), clippy --all-targets -- -D warnings, player build. test compile1m18s, lib43.55s, clippy10.59s, build0.88s. 기존 모든 workspace/fixture 렌더와 subject view focus 회귀가 통과했다.

검토본 `runtime/review-runs/ui-media-overlay-20261019-r160/player.exe`, SHA256 `9c6eefd5e0f39aeabbd3216497f8e3d26406b2983297def2abe6952ade6724aa`, PID100752. 기존 가상 모니터3500,100에서 --ui-demo --record-validation 실행. VM/DPI/모니터 설정을 바꾸지 않았다.

## 실제 관측

- 1440×900 Populated에서 SEG, SUB 음성·화자·OCR·교정·변환 다섯 탭, OBJ에 각각 진입했다. 전체 폭 검은 면/하단 transport와 각 화면의 파형·타임라인·비교 목록이 유지된다. OCR ROI는 contain 영상 하단에 놓이고 transport를 덮지 않는다.
- OBJ 첫 진입 이미지 로딩 spinner가 보였고 다음 별도 캡처에서 정상 원본·마스크·필름스트립으로 바뀌었다. 합성→원본 클릭으로 mask가 제거되고 focus가 표시됐다. Right로 마스크→Right로 합성을 전환했고 선택/focus와 원본 좌표가 유지됐다. 좌측 편집 도구·우상단 보기 선택은 영상 안에 있다.
- 760×480 Populated OBJ는 최소 캔버스 높이 때문에 transport가 첫 화면 아래에 있다. 본문 스크롤 후 온전한 transport에 접근했다. 영상 비례·마스크는 유지되지만 한 화면 접근성은 후속 차이다.
- 760 SUB 변환/OCR에서 프레임·transport가 들어가며 OCR ROI가 축소된 contain 영역 안에 표시됐다. Empty로 바꾸면 이전 영상/ROI가 사라지고 소스 연결 필요가 표시됐다. OBJ Empty에서도 이전 mask가 제거되고 스크롤 위치가 유지됐다. SEG Empty는 검은 프레임·transport·시간범위 연결 필요가 표시됐다.
- 캡션 닫기 후 fresh list_windows에서 해당 경로 창0개를 확인했다.

## 미검증과 다음 우선순위

760 SUB 음성/화자/교정, SEG Populated, OBJ 좁은 폭 보기 키보드, drawer/Escape, LongLabels,980/1180,일반 시청/실재생은 이 변경본으로 실제 검토하지 않았다. 기존 CLIP AI/SAVE/OUT는 API 외형이 유지되지만 r160 실제 재진입 미검증이다. 시안의 transport 세부 구성과 각 화면 표/검사기/타임라인 차이도 남는다. 전체 UI 일치 완료가 아니다.

조율 세션의 확정 방향에 따라 다음은 자막 교정·카드 문구·하이라이트의 원문/제안 비교→사용자 수정→명시적 채택 UX다. [소스 감사](proposal-ui-gap-audit-2026-10-19.md)에 누락을 기록했다. domain/application/API/CLI 계약은 SDK가 소유하며 UI에서 가짜 저장/자동채택을 만들지 않는다. 모델 품질 판정은 별도 연구 세션 범위다.
