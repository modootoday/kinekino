---
id: evidence:kinekino/20261029113934
title: 로컬 영상 실제 시청 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 로컬 영상 실제 시청 검토

2026-10-18. [시청 요소 계약](../../player/presentation/viewing/visual-elements.md), [이전 무소스 관측](viewing-scroll-review-2026-10-18.md)의 후속이다. 이번은 실제 로컬 미디어를 사용한 UI 관측이며 새 Rust 변경은 없다.

## 실행 조건

r22: runtime/review-runs/ui-playback-20261018-r22/player.exe, PID120792. 이전 검토 player가0개임을 확인하고 하나만 실행했다. 기존 trusted target/debug/mpv.exe를 같은 검토 폴더에 복사했다. 실제 child mpv PID80288/parent120792/동일 r22 경로를 확인했다.

소유 가상 모니터 위치3500,100·기본 창1180×760. 기존 runtime/ai-spike/source/Sintel.2010.720p.mkv를 Ctrl+O 파일 선택 대화상자로 열었다. 대화상자에서 기존 검증 경로를 명시 입력했다. 네트워크 미디어나 사용자 파일을 새로 다운로드하지 않았다. VM·DPI·모니터 구성은 변경하지 않았다.

## 실제 확인

| 요소 | 관측 |
|---|---|
| VIEW-02a 제목 | Source Player와 Sintel.2010.720p.mkv가 구분선으로 분리됨 |
| VIEW-03a/06a | 네이티브 연결 후 영상과 00:08 / 14:48 표시. 연결 직후 흰 초기 면은 후속 안정 프레임에서 실제 영상으로 바뀜 |
| 일시정지 | Space 후 재생 glyph로 바뀌며 00:17에 멈춤 |
| seek | 일시정지 중 Right 후 00:22로 이동하고 화면 내용도 바뀜. 일시정지 유지 |
| VIEW-08a | P로 패널을 열면 영상은 남은 작업 영역 안에서 비율을 유지해 축소됨. 제목·X·빈 회차 안내와 transport 표시 |
| 전체화면 | 버튼으로 가상 모니터3840×2160 전체화면에 진입. 패널이 열린 상태에서 두 번 Escape 후 일반1180×760으로 복귀 |
| 상태 유지 | 일반 창 복귀 뒤에도00:22·일시정지 유지 |

첫 Escape 직후 화면 반환의 저해상도 전달 옵션이 지원되지 않아 그 중간 이미지를 직접 재검토하지 못했다. 두 번째 Escape 후 일반 창 복귀는 직접 관측했다. 따라서 이 기록만으로 첫 Escape의 패널 우선순위 시각 인수를 완료하지 않는다. 전체화면에서 제목줄/transport는 일시정지 상태에 표시됐으며, 재생 중 idle overlay의 시안 일치와 별개다.

## 종료와 미검증

캡션 닫기로 r22를 종료했다. 후속 inventory에서 player와 소유 r22 mpv 프로세스가 모두0개임을 확인했다. 저장된 실제 화면 캡처 파일은 없으며 화면을 직접 관측했다.

음성 출력·mute/volume, pointer seek/drag, 재생 중 자동숨김·focus/menu 유지, 전체화면 첫 Escape의 중간 상태, 좁은 실제 재생 창·긴 제목·웹 재생·실제 회차 목록은 남아 있다. 로컬 단일 영상 검증을 전체 시청 상태의 완료로 확대하지 않는다. 최초 파일 대화상자는 열리기 전 window inventory에 나타나지 않았고, 최신 state의 owned modal을 확인한 뒤 진행했다.
