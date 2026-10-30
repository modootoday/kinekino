---
id: sot:kinekino/design-preview
title: 디자인 데모 데이터 파이프라인
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 디자인 데모 데이터 파이프라인

시청 회차 패널은 caption의 `회차 예시`로 진입한다. `design_preview/episodes`가 명시적 UI 표본을 공급하고 실제 제품과 같은 `components::episode_list`로 렌더한다. 채워진18행·LongLabels·Empty와 현재/focus/활성화를 비교하며 샘플 선택은 UI current만 변경한다. URL·실제 공급자 결과·영상/PTS·재생 성공을 생성하지 않는다. Escape/닫기는 제작 데모로 복귀한다. [실제 검토](../../../verification/presentation/episode-panel-review-2026-10-18.md).

시안의 채워진 상태를 검토하기 위한 명시적 실행 모드는 `player.exe --ui-demo`다. 실제 플레이어와 같은 StudioShell·Taffy 레이아웃·Pretendard·SVG·표·검사기 컴포넌트를 사용한다. 타이틀에 `디자인 데모 · 샘플 데이터`를 표시한다.

## 책임 경계

- `application::workspace_read`: 페이지 식별자, 소스·시간·타임라인·표·검사기·대기열의 읽기 모델 및 `WorkspaceReadPort` 계약. UI 색상·픽셀·렌더링과 쓰기 명령을 포함하지 않는다.
- `application::workspace_read::preview`: 결정적인 샘플 공급자. Empty/Populated/LongLabels/NeedsInput을 공급하고 origin은 항상 DesignSample이다.
- `presentation::design_preview`: 데모 실행 호스트와 시나리오 선택. 실제 브라우저·미디어 프로세스·네이티브 파일 선택기를 조립하지 않는다.
- `StudioShell`: 동일 포트로 Live/DesignSample 데이터를 받아 공통 컴포넌트에 연결한다. 미구현 실제 공급자는 Unsupported로 표시하고 데모를 실제 성공 상태로 대체하지 않는다.
- `components/data_table`: column_id로 셀을 연결하고 row_id로 선택한다. 마이크로초 정수 표시는 UI 포맷 책임이며 계산 정책을 바꾸지 않는다.
- `studio/read_view`: 미리보기와 타임라인 표현. 등록된 asset_id만 내장 SVG에 연결한다. AssetReference.path를 임의 파일·URL 로딩이나 프로세스 실행에 사용하지 않는다.

## 확장 절차

새 페이지나 상태의 인터페이스는 SDK 소유 세션과 조율한다. UI가 application 타입이나 샘플 값의 복제본을 만들지 않는다. 페이지별 표 열·상태·폼 필드·시간 범위를 먼저 계약으로 정하고 같은 컴포넌트에 주입한다. 실행·저장·모델 설치 같은 명령은 샘플 읽기 모델과 분리한다.

각 페이지를 4개 시나리오, 기본·넓은·좁은 폭, 선택·포커스 상태로 검토한다. 이미지의 데이터 밀도·행 높이·열 폭·문자 길이·비활성 표현을 비교한다. Empty 상태 통과만으로 시각 일치 완료 처리하지 않는다. SVG 예시 프레임은 디자인 자산이며 실제 미디어·AI 산출물의 관측 증거가 아니다.

계약: `runtime/coordination/workspace-read-model.md`. 제품 정본의 전체 재배치는 수행하지 않는다.

데모제목줄의창프리셋은1440×900,1180×760,980×760,760×480이다. viewport의창크기만변경하고OS배율/DPI는변경하지않는다. Esc입력은StudioShell의공유핸들러를통해팝오버→설정→작업공간순서를일반실행과공유한다.
