---
id: adr:kinekino/20261029113544
title: 관측 목록 기반 회차 전환과 전체화면 표시 정책
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 관측 목록 기반 회차 전환과 전체화면 표시 정책

## 결정

현재 페이지의 실제 회차 링크를 Rust 회차 큐로 전달한다. 미관측 API나 URL 숫자 조합은 사용하지 않는다. 작품/회차 DOM 수집 어댑터, 목록/경계 정책, 패널 렌더링을 분리한다. 큐는 중개 플레이어 이동 동안 유지하고 카테고리·다른 작품 이동에서 폐기한다.

회차 선택은 원본 사이트의 정상 “바로 보기” 흐름을 실행하고, 네이티브를 사용 중이었다면 이전 프로세스를 정리한 후 검증된 새 소스로 재시작한다. 기존 인증·접근 제한을 해제하지 않는다.

전체화면 표시 정책은 OS와 무관한 단조 시간/입력 모델로 한다. 자식 HWND의 렌더 위에 egui overlay를 올리는 대신 컨트롤의 레이아웃 공간을 제거/복원한다. 현재 WebView/mpv 구조와 호환되지만 표시 전환 시 웹 reflow가 발생한다. 향후 libmpv 텍스처 합성이 필요하면 재검토한다.

## 결과

세부 계약·실제 관측·미검증 범위는 [회차/자동숨김 정본](../../sots/verification/presentation/sot-007-episode-controls-validation.md)을 따른다. 현재 사이트 DOM과 역순 목록에 의존하므로 사이트 변경 시 선택자 및 순서 계약을 재검증해야 한다. 자동 재생이 거절되면 웹 Play로 복구한다. 자동 다음화·영속 이어보기는 별도 기능이다.

## 전환 안정성 보완

전환 상태와 네이티브 일시정지 의도를 application 모델로 분리한다. 브라우저 어댑터는 문서 세대와 IPC 출처를 확인하고 presentation은 폐기된 세대의 이벤트를 적용하지 않는다. 세부 범위와 남은 요청 식별 한계는 [후속 정본](../../sots/verification/playback/sot-008-transition-reliability.md)을 따른다.

## 사이트 내부 URL 변경 동기화

실제 작품 카드 전환에서 주소·회차 목록 동기화 누락을 재현했다. NavigationStarting의 허용 정책은 유지하고 별도 infrastructure SourceChanged 어댑터에서 엔진 Source를 관측한다. 문서 URL이 달라졌을 때만 세대를 갱신하고 Navigated를 전달하여 같은 URL의 중복 초기화를 피한다. 상세 관측과 빌드 범위는 [호스트 검증 정본](../../sots/verification/playback/sot-010-host-endurance-validation.md)을 따른다.
