---
id: sot:kinekino/viewing
title: 시청 화면
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 시청 화면

시각 구현에는 [요소별 시각 계약](visual-elements.md)을 함께 적용한다. 아래 기능 목록만으로 배치를 임의 구성하지 않는다.

현재 Meari 후계의 transport/문제 해결·시간/음량 props와 미검증 상태는 [일상 transport 계약](everyday-transport-contract.md)이 소유한다. 과거 egui 시각 인수나 SDK 기능 통과를 해당 후계의 실제 GUI 완료로 확대하지 않는다.

## 시각화

- [start](../../../../artifacts/design/v2/viewing/native-player-start-v2.png)
- [playback](../../../../artifacts/design/v2/viewing/native-player-playback-v2.png)
- [web](../../../../artifacts/design/v2/viewing/native-player-web-v2.png)
- [fullscreen](../../../../artifacts/design/v2/viewing/native-player-fullscreen-v2.png)
- [skip](../../../../artifacts/design/v2/viewing/native-player-skip-v2.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| VIEW-01 | 시작 | 검정 영상 영역·파일 열기·작품 탐색, 소스 없을 때 재생 비활성 |
| VIEW-02 | 제목줄 | 작품명·회차, 말줄임과 전체 제목 툴팁, 창 이동·최소화·최대화·닫기 |
| VIEW-03 | 재생 | 응답 전 로딩 표시, Space, 현재 일시정지 의도 유지 |
| VIEW-04 | 이전/다음 | 회차 경계 비활성·대상 제목 툴팁, 연속 선택 마지막 우선 |
| VIEW-05 | 탐색선 | 전폭·위쪽 배치, 드래그 예상 시각, 놓으면 seek, 미상 비활성 |
| VIEW-06 | 시간 | 고정폭 현재/전체, 1시간 이상 HH:MM:SS, 미상 --:-- |
| VIEW-07 | 볼륨 | 0~100, 음소거와 값 분리, 좁은 창 팝오버 |
| VIEW-08 | 회차 패널 | 260폭·행40, 현재/연결 중 행 구분, 긴 제목·스크롤 |
| VIEW-09 | 전체화면 | 영상 더블클릭/버튼, Esc 단계 복귀, 입력 복원 |
| VIEW-10 | 자동숨김 | 재생 중 2.5초 제안, 일시정지·오류·드래그·메뉴·focus 중 유지 |
| VIEW-11 | 작품 탐색 | 웹 보조 모드, 광고 차단 상태, 이전 네이티브 일시정지 |
| VIEW-12 | 네이티브/웹 | 명시적 전환, 준비 확인 후 영상 이동, 이중 소리 금지 |
| VIEW-13 | AI 스킵 | 확인 구간에서 버튼 또는 자동 카운트다운, SEG 정책 적용 |
| VIEW-14 | 제작 진입 | 기능이 준비되면 노출, PTS·미디어 ID를 초안에 전달 |

## 상태와 오류

재생 없음·연결 중·재생·일시정지·버퍼링·실패·종료를 구별한다. 오류에는 재시도와 명시적 웹 복구를 제공한다. 웹에서 전달된 오래된 이벤트는 현재 회차를 변경하지 못한다.

## 저장 계약

2026-10-26 상세: [시청 설정·이어보기](../../application/viewing-preferences-and-resume.md)는 사용자 의도와 실제 엔진 관측·source 재해석·새 engine readiness·CAS/replay·재열기를 분리한다. 현재 영속 이어보기 지원이나 새 CLI 실행 성공을 의미하지 않는다.

현재 회차, volume, mute, pause_intent, panel preference, fullscreen 이전 bounds를 저장한다. 서명 URL은 저장하지 않는다.

## 인수 기준

영상 더블클릭이 버튼을 방해하지 않음, 회차 경계, 포커스, 네이티브/웹 오디오, 자동숨김, DPI.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

## 네이티브 공통 재생 제어 연결 후속

2026-10-19. GUI play/pause/seek/volume은 바인더가 주입된 실행에서 typed ApplicationClient.playback_control로 전달한다. optional 바인더 없는 기존 실행은 typed mpv handle 호환 경로를 사용하며 SDK provider 준비를 주장하지 않는다. 바인딩은 engine lifecycle 책임이고 UI는 접수 receipt로 native pause/time/volume 관측을 만들지 않는다. pause 관측 전 toggle은 요청을 보내지 않고 안내한다. volume UI 편집 버퍼는 실제 property 관측으로 동기화한다.

실패는 transport 위 공용 Warning으로 표시하고 자동 숨김을 고정한다. 오류 발생 후 성공한 새 제어 요청에서 안내를 해제한다. 연결 전·대상 없음·큐 포화·닫힘을 성공으로 취급하거나 legacy로 조용히 우회하지 않는다. startup mute 해제는 새 포트 지원 밖의 기존 adapter 명령으로 남으며 실패를 표시한다. 기존 native status publisher 경로를 유지한다. 새 연결의 실제 미디어 검증은 별도 기록하며 명령 접수를 적용 완료로 표현하지 않는다.

