---
id: evidence:kinekino/20261029113756
title: 공용 영상 프리뷰 크기 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공용 영상 프리뷰 크기 검토

2026-10-19. r113 최대화 SEG의320고정 높이와 과도한 여백을 후속 수정했다. skip-settings.png 전체와 (274,138,895,565)2배 확대를 비교했다. 파생물 `runtime/review-runs/media-preview-r114-2x.png`. 시안의 영상→transport→filmstrip→결과 계층을 유지하며 이미지의 원본 contain 비율을 보존한다.

## 구조와 1차 관측

공용 artboard::media_height가 가용 폭/원본 비율과 viewport 높이 비례 토큰 MEDIA_PREVIEW_VIEWPORT40% 중 작은 크기를 사용한다. SEG/OBJ/SUB/SAVE의 media preview가 소비하고 카드·crop·PROJ·레이어 canvas에는 일괄 적용하지 않는다. 비례값은 논리 설계 결정이며 시안의 pixel을 정확 추출한 값이 아니다.

r114 PID72420,1440×900: 프리뷰가640×360으로 확장되고 원본 비율을 유지했다. 시간축·표가 이어졌고 하단 편집 도구는 세로 스크롤 아래에 있다. 최대화3840×2112에서는 프리뷰가 약1502×845로 확장돼 이전568×320 고정 표시를 벗어났다. 최대화 뒤 복원1440에서도 비율이 유지됐다.760×480 SEG에서는 약341×192 프리뷰·재생 도구·시간 눈금이 첫 화면에 함께 표시됐다.

같은760 OBJ로 이동해 실제 overlay 도구를 확인하니 모드 컨트롤이 피사체 상단을 가리고 도구 레일 하단도 잘렸다. 이미지 로드 후 새로 캡처해도 같은 배치였다. 따라서 이 상태를 통과로 두지 않고 OBJ에는 기존 최소320높이를 유지하도록 보완했다. r114는 caption으로 종료했다.

## 최종 보완 실행

r115 PID71272,최종 코드:1440 SEG360높이 유지,760 SEG192높이를 확인했다.760 OBJ는320높이로 복구되어 피사체 상단이 mode overlay에 가려지지 않았고5개 도구 레일이 영상 안에 온전히 표시됐다. 마스크 클릭 →Right로 합성 복귀 시 mode 선택과 원본 좌표의 overlay가 유지됐다.760 SUB 음성 인식은192높이 프리뷰/transport와 아래 파형 영역으로 이어졌고 Empty 전환 시 같은 크기의 '소스 이미지 연결 필요' 골격으로 바뀌었다. 파형 전체/하단 cue까지 이번에 새로 시각 인수한 것은 아니다.

최종 cargo fmt --check, cargo test(lib71 passed/1 ignored 및 CLI/통합), cargo clippy --all-targets -- -D warnings, cargo build --bin player가 통과했다. 외부 미디어 의존 ignored 검사는 실행하지 않았다. r114/r115는 기존 가상 모니터 위치3500,100에서 한 개씩 실행하고 caption 닫기로 종료했다. 종료 후 player.exe/mpv.exe 조회 결과가 비어 있었다. VM·DPI·모니터 구성은 변경하지 않았다.

## 남은 검토

실제 원본 크기/회전/PAR·HDR·실제 PTS·웹/영상 합성·다른 비율, 모든 화면/탭/상태·장시간 성능과 접근성은 이번 수정으로 검증되지 않는다. 다른 편집 canvas의320고정 높이는 별도 남은 범위다. 프리뷰 확대를 전체 화면 시각 일치로 판정하지 않는다.
