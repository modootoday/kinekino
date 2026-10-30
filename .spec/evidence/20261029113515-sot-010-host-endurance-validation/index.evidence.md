---
id: evidence:kinekino/20261029113515
title: 호스트 반복 회차·장시간 재생 검증
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 호스트 반복 회차·장시간 재생 검증

## 범위와 기준

사용자 지시로 DPI와 VM 설치를 보류하고 호스트의 Rust 앱만 시험한다. VM, Hyper-V 서비스, 드라이버 및 배율을 변경하지 않는다. 광고 차단 체크박스 ON 상태를 직접 관측했다. [최상위 규칙](../../../AGENTS.md)을 따른다.

표본: 어떤 암부의 소녀공서 1·2화의 네이티브 반복 전환, 파리에 피는 에투알(전체 길이 약 7206.2초)의 웹 30분/mpv 60분 연속 재생. 목표 시간은 실제 완료 증거와 별개다. 여러 앱 인스턴스를 동시에 실행하며 공유 WebView2 프로세스의 자원 소유가 한 앱 조상에 묶일 수 있으므로 모드별 메모리 비교로 해석하지 않는다.

## 진단 개선

infrastructure/diagnostics에 Instant 기반 elapsed_ms를 추가했다. 서명 미디어 URL·헤더·본문을 추가로 기록하지 않는다. [당시 집계 스크립트](../../../../scripts/validation/summarize-soak.py)는 단조 시각과 영상 시각을 함께 검사한다. 인접 로그 공백 5초 초과, 역행/탐색 점프, 10초 초과 정체, 일시정지/오류는 연속 구간을 끊는다. 목표 시간은 벽시계 구간과 실제 미디어 증가량이 모두 만족해야 한다. 정상 진행/탐색 점프/정체/누락 표본을 합성 입력으로 점검했으며 합성 입력을 실제 재생 증거로 사용하지 않는다.

[당시 자원 표본 스크립트](../../../../scripts/validation/monitor-host-playback.ps1)는 시험 앱과 그 자식의 프로세스 수·private bytes·working set·CPU 누적값만 30초 간격으로 수집한다. 다른 프로세스를 제어하지 않는다. working set 합에는 공유 페이지 중복 가능성이 있다. 전체 시스템/GPU 성능 시험은 아니다.

## 실행 기록

새 검증 실행 사본을 복사·시작하는 복합 명령이 자동 승인 검토에서 구체적 사유 없이 차단됐다. 해당 명령은 반복하지 않았고, 사본 생성 없이 기존 빌드를 직접 실행하는 방식으로 실제 앱을 시작했다. fmt 정리 후 테스트 17건·Clippy 통과. 실행 중 바이너리 때문에 빌드 교체가 한 차례 실패했으며 해당 앱을 정상 종료한 뒤 빌드 성공을 확인했다.

실제 시험 로그는 runtime/verification/stress-host-final.jsonl, soak-native-host.jsonl, soak-web-host.jsonl, soak-resources-host.jsonl에 수집 중이다. 앱 종료 후 flush된 로그를 엄격하게 파싱하여 최종 수치·해시를 기록한다. 실행 중 부분 마지막 줄은 live 조회에 한해서 제외한다.

## 판정

호스트 반복/웹30분/mpv60분의 정해진 표본 시험 완료. 수정 빌드의 짧은 회귀와 수정 전 진단 빌드의 연속 시험을 구분한다. 이후 사용자는 VM 실패로 호스트 가상 디스플레이 경로를 별도 지시했으며 [드라이버 검토](../display/sot-015-host-virtual-display-review.md)를 따른다.

## 반복 전환 확정 결과

[기계 판독 결과](../../../evidence/playback/stress-host-2026-10-18.json): 초기 재생 이후 완결 네이티브 전환 10회, 로딩 중 이전/다음 10쌍(선택 20회), 최종 2화의 일시정지 표본 34개에서 위치 0.088156초 동일. 이후 재개와 시간 진행 확인. 전환 시간 초과 0건·네이티브 오류 0건. 중간 선택은 대체될 수 있으며 20개 영상이 모두 완결 연결됐다는 뜻은 아니다. [10회 후 화면](../../../evidence/playback/stress-ten-transitions.jpg), [최종 일시정지](../../../evidence/playback/stress-paused-final.jpg). 연결 중 종영 이동 1회에서 목록 비움·시간 0/0·mpv 비활성 확인. 이 결과는 SourceChanged 수정 전 진단 빌드 범위다.

## 발견 문제와 수정

사이트 방영중 작품 카드를 눌러 상세 페이지로 이동하면 웹 콘텐츠는 바뀌지만 Rust 주소와 회차 패널은 목록에 남는 문제를 재현했다. [수정 전](../../../evidence/playback/history-navigation-before.jpg). NavigationStarting만으로는 동일 문서 URL 변경을 모두 추적할 수 없다. infrastructure/browser/location.rs에 WebView2 SourceChanged 관측을 분리하고 엔진 Source를 허용 목록 검사 후 문서·세대·Navigated 이벤트에 반영했다. 기존 IPC 출처 일치 검사를 완화하지 않았다. 같은 URL은 세대를 중복 증가시키지 않는다. [Microsoft 탐색 이벤트 설명](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/navigation-events).

수정 후 동일 카드 클릭에서 상세 주소·두 회차 목록을 실제 확인했다. [수정 후](../../../evidence/playback/history-navigation-after.jpg). 패널 선택→웹 재생→내장 mpv 재생도 확인했다. 단위 회귀 포함 Rust 18건 통과. 장시간 창은 수정 전 진단 빌드이며 수정 빌드 전체의 60분 시험으로 확대하지 않는다.

수정 빌드에서 이어서 이전 1화→다음 2화의 실제 네이티브 진행과 연결 중 종영 이동 취소를 확인했다. [수정 빌드 기계 판독 결과](../../../evidence/playback/history-fix-host-2026-10-18.json), [취소 후 화면](../../../evidence/playback/history-fix-cancelled.jpg). 실제 카테고리 취소는 수정 전 1회+수정 후 1회이며 계획의 5회로 부풀리지 않는다. 네트워크 장애를 강제로 만들어 30초 시간 초과를 재현하지는 않았다.

## 웹 연속 재생 확정

정상 종료 후 엄격한 JSON 파싱 결과: 벽시계 1859.008초, 영상 증가 1859.005초(30분 59초), 1860개 연속 표본. 웹 30분 기준 통과. 웹 오류 표본 0·네이티브 오류 0, 알려진 visible_ads 선택자 최대 0·요청 차단 카운트 최대 1. 알려진 선택자의 0이 모든 광고 유형의 부재를 보장하지는 않는다. [기계 판독/로그 해시](../../../evidence/playback/soak-web-host-2026-10-18.json), [종료 직전 실제 화면](../../../evidence/playback/soak-web-final.jpg). mpv 60분은 계속 진행 중이다.

## mpv 연속 재생 확정과 시험 종료

60분 통과 뒤 실제 프레임/광고 차단 ON 화면을 확인하고 정상 종료했다. 엄격 파싱 결과 벽시계 3629.438초·영상 증가 3628.926425초(60분 28초), 3541개 연속 표본. 네이티브 오류 0·웹 오류 표본 0, 알려진 visible_ads 최대0·차단1. [Rust 기계 판독 결과/로그 해시](../../../evidence/playback/soak-native-host-2026-10-18.json), [종료 직전 화면](../../../evidence/playback/soak-native-final.jpg). 이 표본의 전체 영상 길이는 약7206.2초다. 1시간 성공을 전체 콘텐츠·무기한 안정성·메모리 누수 부재로 일반화하지 않는다.

소유 검증 플레이어를 정상 종료하고 소유 자원 수집기 PID33124만 종료했다. WindowsTerminal PID2580의 생존을 확인했다. ChatGPT/Terminal/다른 앱을 종료하지 않았다. 이후 기본 kinekino-player.exe를 SourceChanged 수정 포함 소스로 다시 빌드했다. 장시간 표본은 수정 전 진단 빌드이며 수정 빌드 전체의 60분 회귀는 남은 작업이다.

Rust 통일 지시 이후 연속성 집계·AI 측정 도구를 tools/validation으로 이관했다. 이관 중 웹 error=0/false를 오류로 세던 파싱 문제를 발견·수정하고 회귀를 추가했다. 최종 수치는 교정한 Rust 집계다. 초기 Python 웹 집계와 연속 시간은 일치한다. AI 추론은 이 재생 시험 종료 이후에만 시작했다. 이 시험 중 개발 빌드/문서 조사/모델 데이터 다운로드가 있었으며 완전 유휴 호스트 시험은 아니다.
