---
id: evidence:kinekino/20261029113848
title: PROJ 파일 행 시각 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# PROJ 파일 행 시각 검토

2026-10-19. PROJ-13a. 기준 projection-package.png의 (860,194,235,203)을4배 확대했다. 파생물 `runtime/review-runs/projection-files-r105-4x.png`. 폴더와 image/mask/depth/JSON/cube 기호, 이름 시작선, 행 선택 면을 비교했다. 이전 r104 실행에는 glyph가 없고 텍스트 주변만 선택됐다.

공용 FileRow는32 높이·20 SVG·8 간격·행 전체 hitbox를 사용한다. 선택 면·왼쪽 표시·키보드 focus ring을 구분한다. 이름은 한 줄 제한과 전체 hover를 제공한다. scene 폴더는 기존 공용 folder SVG와 egui collapse state를 사용한다. 파일 종류는 fixture의 명시 데이터이며 실제 파일 검사나 미리보기 구현이 아니다.

## 실제 관측

r105 PID24560,1440×900: 폴더와 다섯 파일 glyph가 정렬됐다. mask 행의 오른쪽 빈 공간(x1098,y300)을 클릭한 후 Down 입력으로 depth.png 선택과 안내가 바뀌었고 행 전체 선택 면·focus ring을 확인했다. 원본 참고 이미지는 선택 파일 내용으로 바꾸지 않는다. texture에 사용한 기존 Image 기호가 카메라 모양인 차이를 발견해 FileImage SVG로 분리했다. r105는 caption 닫기로 종료했다.

## 최종 실행과 검사

최종 r106 PID13696: 1440에서 texture.png의 그림 SVG 교체, 행 오른쪽 클릭 선택, End로 scene.glb 선택과 안내를 확인했다. 760에서는 목록까지 스크롤한 뒤 mask 행 오른쪽(x700,y312) 클릭 → Home으로 texture.png 선택·focus 이동을 확인했다. 폴더 화살표 클릭으로 다섯 행이 접히고 선택 파일 안내와 원본 참고 영역은 유지됐다. r106도 caption 닫기로 종료했으며 player.exe/mpv.exe 조회 결과가 비어 있었다. 기존 가상 모니터 위치를 사용했고 VM·DPI·모니터 구성은 변경하지 않았다.

최종 코드에서 cargo fmt --check, cargo test(lib70 passed/1 ignored 및 CLI/통합 검사), cargo clippy --all-targets -- -D warnings, cargo build --bin player가 통과했다. 외부 미디어 의존 ignored 검사는 이번에 실행하지 않았다.

## 미검증

전체 PROJ 시각 일치가 아니다. 실제 파일 kind/size/dimensions/preview/검증 결과와 카메라·깊이 모드는 여전히 연결되지 않았다. 긴 파일명·접근성 낭독·pressed 지속 상태·모든 창 크기와 실제 파일은 별도 검증한다.
