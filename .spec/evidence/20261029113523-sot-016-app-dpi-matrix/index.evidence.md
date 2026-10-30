---
id: evidence:kinekino/20261029113523
title: 가상 모니터에서 앱 HWND DPI와 실제 UI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 가상 모니터에서 앱 HWND DPI와 실제 UI 검증

## 범위와 근거

Windows11 현 호스트, 설치된 virtual-display-rs v0.3.1의 소유 ID9, 확장 가상 화면3840×2160에서 수행했다. 물리 주 화면3440×1440/96 DPI는 유지했다. 광고 차단은 ON이며 방영중 ‘템빨 ~아이템의 힘~’의1·2·3화로 시험했다. 이전 모니터 probe만의 결과와 달리 **실제 플레이어 HWND의 GetDpiForWindow** 및 창/클라이언트/모니터 영역, egui pixels_per_point를 초당 기록했다.

실행 파일 SHA256 `2749470cf4ee59cfab6ebf056d6e4cf17fc78523411993febf10840bb7c06e8b`. 로컬 CLI `--record-validation --validation-position=3600,100`으로 가상 화면 안에서 시작했다. 웹 입력으로 위치를 지정하지 않는다. 같은 창에서 Windows 설정으로100→150→200→100%를 적용했다. 검증 위치 인수는 기록 모드·유한 좌표 검사 후에만 허용된다.

[Rust 집계 결과](../../../evidence/display/app/dpi-app-matrix-2026-10-18.json), [원본 로그](../../../../runtime/verification/dpi-app-100.jsonl). 로그 SHA256 `df89d2bf6d8ada84d5f3c260bc9982791b746ff44bb96b34d749e871f566a1f7`. 로그 마지막 경과457.026초. 장시간 시험이 아니라 약7분37초의 UI 매트릭스다. stderr는 비어 있었다.

## 실제 조작 결과

| 항목 | 100% | 150% | 200% |
|---|---|---|---|
| 앱 HWND DPI / egui 배율 | 96 / 1.0 | 144 / 1.5 | 192 / 2.0 |
| 가상 모니터에 속한 앱 위치 표본 | 148 | 118 | 177 |
| 정상 창·제목·시간·하단 컨트롤·회차 목록 경계 | 확인 | 확인 | 확인 |
| mpv 전체화면 창 영역=가상 모니터 영역 | 3840×2160 | 3840×2160 | 3840×2160 |
| 전체화면 진입·Escape 해제 | 확인 | 확인 | 확인 |
| 재생 중 패널 닫고 유휴 자동숨김 | 확인 | 확인 | 확인 |
| 입력으로 컨트롤/패널 복원 | P 키 | Space·P 키 | 영상 위 포인터 입력·P 키 |
| 재생·일시정지·+30초 탐색 | 확인 | 확인 | 확인 |
| 볼륨 클릭과 mpv 관측값 | 45(복원 후) | 42 | 76 |
| 회차 선택/전환 | 2화 선택→이전화1화 | 다음화2화 | 패널3화→전체화면 패널2화 |
| 네이티브 오류 표본 | 0 | 0 | 0 |

선택 후 새 회차의 네이티브 연결·시간 진행을 확인했다.100% 이전화 전환과200%3화 선택에서는 일시정지 의도가 유지되는 것도 확인했다.200% 전체화면에서3→2화 선택 후 Escape로 정상 창에 돌아와 재생했다. 표본은3회차 목록이며 대량 목록 스크롤·모든 사이트 콘텐츠를 검증한 것은 아니다.

네이티브 재생/정지 표본 수는100%46/32,150%40/61,200%45/52다. 이 수는 시간 증가량이나 완결 전환 횟수가 아니다. native 상태는 바로 앞의 유효 window-metrics 표본에 연결하므로 배율 변경 경계의1초 이내 분류 한계가 있다. 화면·실제 조작과 함께 평가한다.

스크린샷:

- 100% [자동숨김](../../../evidence/display/app/dpi-app-100-fullscreen-idle.jpg), [복원 후 컨트롤·목록](../../../evidence/display/app/dpi-app-100-controls.jpg).
- 150% [자동숨김](../../../evidence/display/app/dpi-app-150-fullscreen-idle.jpg), [전체화면 패널·정지](../../../evidence/display/app/dpi-app-150-fullscreen-panel.jpg), [탐색·볼륨](../../../evidence/display/app/dpi-app-150-controls.jpg).
- 200% [자동숨김](../../../evidence/display/app/dpi-app-200-fullscreen-idle.jpg), [포인터 입력 복원](../../../evidence/display/app/dpi-app-200-mouse-restored.jpg), [전체화면 패널](../../../evidence/display/app/dpi-app-200-fullscreen-panel.jpg), [웹 전체화면 자동숨김](../../../evidence/display/app/dpi-app-200-web-fullscreen-idle.jpg).

웹→mpv는100/150%, mpv→웹은150/200%, 복원100%에서 웹→mpv를 실제 확인했다. 웹 전체화면은200%에서 진입·자동숨김·Escape 해제를 확인하고, 추가로100/150%에서 동일 항목과 P 키를 통한 회차 패널·컨트롤 복원을 확인했다. 웹 호환 모드는 사이트 페이지의 스크롤/안내 영역을 유지하며 네이티브 영상 표면과 동일한 구성으로 평가하지 않는다.

추가 웹 시험의 실제 앱96/144 DPI·전체화면 영역/숨김/표시·패널 표본은 [보충 집계](../../../evidence/display/app/dpi-web-supplement-2026-10-18.json), [100% 숨김 화면](../../../evidence/display/app/dpi-app-100-web-fullscreen-idle.jpg), [150% 숨김](../../../evidence/display/app/dpi-app-150-web-fullscreen-idle.jpg), [150% 전체화면 패널](../../../evidence/display/app/dpi-app-150-web-fullscreen-panel.jpg)에 보존했다. [추가 로그](../../../../runtime/verification/dpi-web-supplement.jsonl) SHA256 `25702bc3b83210891f467b9ea83141b5efc8a444bb45cd6507d6d4404965f02b`. 이 보충 실행은 mpv를 실행하지 않아 보충 집계의 native 표본0은 정상이며,192 DPI 행은 이 실행의 대상이 아니다. 웹 오류0, 실제 영상 증가157.003644초/벽시계157.054초의 연속158표본을 [재집계](../../../evidence/display/app/dpi-web-progress-2026-10-18.json)했다. 이것은 추가 UI 조작 중 연속성 관측이며 새 장시간 시험이 아니다.

## 발견 사항과 변경

이전 검증은 모니터 probe만 측정해 앱 자체의 DPI를 입증하지 못했다. infrastructure/window_metrics.rs에 실제 HWND의 읽기 전용 관측을 분리하고, presentation은 어댑터 값과 UI 상태만 기록하도록 보완했다. application/launch.rs는 로컬 시험 위치의 계약/검사를 담당하고 main은 조립한다. tools/validation의 domain/dpi.rs는 실제 앱 DPI·배율·비주 모니터 일치 조건, application/dpi_report.rs는 집계, infrastructure/files.rs는 파일/해시 책임이다. OS 호출을 UI에 추가하거나 별도 언어로 검증 로직을 만들지 않았다.

150% 회차 로딩 도중 [일시적인 웹 축소·빈 영역](../../../evidence/display/app/dpi-app-150-web-before.jpg)을 관측했다. 연결 완료 후 mpv 표면과 웹 복귀 화면은 정상 경계로 돌아왔다. Wry0.57.0의 set_bounds가 자식 HWND DPI로 논리→물리 변환하는 구현을 확인했으며 지속적인 배율/클릭 좌표 오류로는 재현되지 않았다. 원인을 확정하지 않은 상태에서 배율을 이중 적용하는 패치를 넣지 않았다. 로딩 중 레이아웃 전환의 시각적 개선은 후속 항목이다. 이번 UI 표본에서 수정이 필요한 지속적인 제품 DPI 결함은 발견하지 못했다.

추가 웹100→150% 재생 유지 시험에서도 정상 경계를 확인해 지속적 축소는 재현되지 않았다. 별개로 검증 도구의 soak가 과거 영화 표본에 맞춘 duration>7000 조건을 갖고 있어 일반 회차를 모두 제외하는 문제를 재현했다. [교정 전 잘못된 실패 집계](../../../evidence/display/app/dpi-web-progress-duration-filter-before.json)를 보존했다. domain/playback의 eligible_media로 유한한 시간·양수 목표·영상 길이≥시험 목표·위치 범위를 판정하도록 수정하고, 일반 회차 허용/짧은 조각·초과 위치 거부 회귀를 추가했다. 동일 로그가157초 연속으로 교정됐고 기존 mpv60분/웹30분 로그를 재집계해 기존 결과가 변하지 않는 것도 확인했다. 제품 재생 실패와 집계 결함을 혼동하지 않는다.

실제 정상 창 client는 초기1180×760,144 DPI1761×1115,192 DPI2343×1467,복원96 DPI1177×750을 관측했다. Windows 권장 창 사각형에 따른 크기 변화를 보존했으며 모든 단계에서 컨트롤/목록이 창 안에 표시되고 클릭됐다. 원래 논리 client 크기와 정확히 동일하게 복원되는지 또는 여러 배율 왕복의 크기 누적은 별도 시험으로 남긴다.

## 종료·검사·남은 작업

가상 화면 배율100% 복원 후 앱/설정 창을 정상 종료했다. [복원 관측](../../../evidence/display/app/dpi-app-restored-100-2026-10-18.json) 뒤 ID9만 제거했고 [최종 화면](../../../evidence/display/app/dpi-app-final-displays-2026-10-18.json)은 물리 주 화면3440×1440/96 DPI 한 개다. 승인된 드라이버·인증서는 설치 상태를 유지했다. ChatGPT/Terminal·다른 앱·VM·서비스는 종료/변경하지 않았다.

제품 cargo fmt --check / cargo test19건 / cargo clippy --all-targets -- -D warnings 통과. 독립 Rust 도구도 fmt/test5건/clippy 통과. 회귀는 위치 인수의 검증 모드·유한 좌표와 모니터 probe/egui zoom만으로 앱 DPI를 통과시키지 않는 조건을 포함한다.

남은 범위:최소 창·장문/대량 회차·키보드 전체 탐색, 혼합 DPI 모니터 사이 드래그/분리·재연결, 여러 배율 왕복의 client 크기 복원,125/175% 분수 배율, 고배율 장시간 재생. 기존 수정 빌드60분·실제30초 연결 만료 시험도 별도다. 이번 결과를 모든 해상도·무기한 안정성·모든 광고 유형 제거로 확대하지 않는다.
