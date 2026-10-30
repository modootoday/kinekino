---
id: evidence:kinekino/20261029113922
title: 일반 제작 제목줄 문맥 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 일반 제작 제목줄 문맥 검토

2026-10-18 공용 제목줄. queue-refined.png의(0,0,540,46)4×는 제품명 뒤 구분선과 스튜디오 문맥이 있다. native-player-studio-v2.png 같은 영역에는 제품명만 있다. 두 시안을 모두 확인하고 명시 문맥이 있는 queue-refined 기준으로 일반 제작 제목줄을 공통화한다. 파생물 studio-context-r98-4x.png/studio-title-r98-4x.png. 제품명 문자열은 PRODUCT에서만 가져온다.

## 재확인 r98

이전 r96 capture에서 제작 왼쪽 제목이 비어 보였으나 새 진단 실행의 안정된 화면에서는 브랜드와 제품명이 정상 표시됐다. 지속 누락으로 확정하거나 이번 코드가 브랜드 누락을 해결했다고 주장하지 않는다.

`runtime/review-runs/ui-studio-title-20261018-r98/player.exe`(PID51648), 일반 실행·기존 가상 모니터3500,100.1180 시작→제작1440에서 제품명만 표시되고 스튜디오 문맥은 없었다. 임시 geometry 진단은 bar[0,0,1440,32], clip[0,0,1440,33], visible=true/sizing=false를 확인했다. 진단은 범위 확인용이며 최종 코드에서 제거했다. r98은 캡션 닫기로 종료했다.

## 변경

일반 제작 제목줄은 공용 window_title에 ‘스튜디오’ 또는 ‘스튜디오 · 소스 제목’을 보조 문자열로 전달한다. 제품명과 구분선/말줄임/전체 hover는 기존 공용 표시를 공유한다. 시청 복귀는 기존 소스 제목만 표시한다. 보조 영역의 간격12/선 높이16·최소 여유48은 기존 PANEL/FIELD/CONTROL+GROUP 토큰을 재사용한다. 캡션 고정 영역과 drag rect를 변경하지 않는다. offline demo의 명시적 검토 host 문구는 별도다.

## 검증

최종 cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo build --bin player, cargo test가 통과했다. 라이브러리69통과/1ignored 및 CLI·통합 통과, 기존 외부 미디어 ignored 유지. 임시 title-bar-geometry 진단은 최종 소스 검색에서도 제거를 확인했다.

## 수정본 실제 실행 r99

`runtime/review-runs/ui-studio-title-20261018-r99/player.exe`(PID23020), 일반 실행·가상 모니터3500,100.1180 시작 제목은 제품명만 표시됐다. 제작 버튼으로1440×900 진입 후 ‘제품명 | 스튜디오’가 표시되고 오른쪽 세 캡션은 끝에 유지됐다. 구간 감지→모델 관리로 전환해도 제작 문맥과 구분선이 유지됐다. Escape로1180 시청 복귀하면 스튜디오 문맥/구분선이 제거되고 제품명만 표시됐다. 캡션 닫기로 종료했고 후속 CIM에서 player.exe/mpv.exe가 없었다. VM·DPI·모니터 구성은 변경하지 않았다.

이 실행에서 일반 MODEL의 후보(0)도 관측했다. 실제 카탈로그 부재/미연결과0개 결과를 구별하는 표시인지 read state를 추가 확인해야 하며 이번 제목줄 검증의 성공으로 해당 의미를 승인하지 않는다.

## 남은 범위

긴 실제 미디어 제목,760/980/1180 제작 폭의 말줄임·전체 hover, 실제 스크린리더, 정상 제작에서 최대화 조합은 미검증이다. 일반 MODEL0 표시는 별도 상태 검토 대상이다. 전체 제목줄 상태/모든 화면의 시각 인수는 미완료다.
