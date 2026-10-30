---
id: evidence:kinekino/20261029114031
title: 시청 회차 패널 공용화·입력 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 시청 회차 패널 공용화·입력 검토

2026-10-18, VIEW-08. 기준 native-player-playback-v2.png 전체·패널2×·현재 행(1198,325,386,72)4×를 검토했다. 기준 현재 행은 play glyph·amber 문자/좌측선·선택면을 갖는다. 번호와 제목이 별도 데이터라는 근거 없이 실제 제목을 파싱해 번호를 만들지 않는다.

## 실제 시청 기준

r81(PID52600), `runtime/review-runs/ui-viewing-panels-20261018-r81/player.exe`,1180×760 normal 실행에서 P로 빈 회차 패널이 열렸다. 패널은 영상 영역을 줄였고 transport를 가리지 않았다. 작품명 빈 label 때문에 header와 구분선 사이에 빈 행이 남았다. Escape는 패널만 닫고 시작 화면을 유지했다. 실제 회차 데이터가 없어 채워진 목록/회차 이동은 검증하지 못했다. 소유 창은 캡션 닫기로 종료했다.

## 공용 구현·데모 경계

episode_list가 header/닫기·선택적 작품명·본문 ScrollArea와 stable ID 행을 소유한다. hit40·play slot32·glyph20·현재선2·공용 색/곡률/focus를 사용한다. 현재 행과 포커스를 분리하고 긴 제목은 truncate+원문 tooltip으로 처리한다. Up/Down/Home/End는 focus만 바꾸고 활성화 결과를 만들지 않는다. Enter/Space/클릭의 activate index만 호출자가 처리한다. 제품은 기존 episode URL을 기존 choose_episode에 전달한다.

명시적 --ui-demo caption의 회차 예시는 같은 컴포넌트에18개 디자인 행/긴 문구/Empty를 제공한다. URL·실제 영상·연결 성공을 주입하지 않는다. 중앙에 디자인 샘플 경계와 입력 안내를 표시하고 활성화는 샘플 current만 바꾼다. Escape/닫기는 제작 데모로 복귀한다. 실제 회차 페이지/PTS/자동 재생은 이 표본으로 검증하지 않는다.

## 수정본 실행

r82(PID76720), `runtime/review-runs/ui-episode-panel-20261018-r82/player.exe`의 명시적 회차 예시에서 확인했다.

- 1440: 현재03행은 play·amber문자/좌측선·선택면, 클릭 후 focus는 별도 외곽선이다. Down은04행 focus만 옮기고 현재03을 유지했다. Enter 뒤에만 현재 표식이04로 이동했다.
- 760: 현재04 클릭→End는18행 focus와 자동 스크롤로 이동했다. Space 뒤에만18행에 현재 play·선택선이 표시됐다. 스크롤 끝에서도 header/작품명/닫기가 고정됐다.
- 760 LongLabels: 한국어·日本語·English 제목이 패널 폭 안에서 줄임됐다. 원문 hover 처리는 구현했으나 이번 dwell 캡처에서 tooltip 표시는 관측되지 않았으므로 실제 tooltip 인수는 남긴다.
- 760 Empty: 이전18행과 현재 표식이 제거되고 안내만 표시됐다. Escape는 회차 예시를 닫고760 제작 SEG 빈 상태로 복귀했다. 시청 영상이나 네트워크 이동은 실행하지 않았다.

r83(PID100544), `runtime/review-runs/ui-episode-native-20261018-r83/player.exe` normal1180×760에서 P로 공용 빈 패널을 열었다. 빈 작품명 행은 예약되지 않았고 안내가 header 구분선 아래로 이동했다. Escape는 패널만 닫고 시작 화면과 transport를 유지했다. 실제 회차가 있는 공급자 목록은 여전히 미검증이다.

최종 fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드 통과. r82/r83은 각각 캡션 닫기로 종료했고 최종 player/mpv 조회는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다.

## 남은 범위

실제 공급자 회차 제목/현재·connecting/error 표시·번호 계약·P의 포커스별 우선순위·패널 폭 조절·전체화면 및 HW 영상과 패널 경계는 별도 검증한다. 전체 시청 시안 일치로 확대하지 않는다.
