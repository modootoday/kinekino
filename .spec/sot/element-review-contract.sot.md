---
id: sot:kinekino/element-review-contract
title: 요소별 설계·비교 기록 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 요소별 설계·비교 기록 계약

[화면 구성 정본](screen-composition.md)의 각 하위 화면을 구현하기 전에 아래 형식의 요소 기록을 해당 화면 설계에 작성한다. 요소 누락을 발견하면 코드보다 설계 기록을 먼저 보완한다. 동일 책임의 규격을 여러 화면 문서에 복제하지 않고 공통 ID로 연결한다.

| 필수 항목 | 기록 내용 |
|---|---|
| 식별 | 화면 ID/하위 상태/요소 ID. 기존 탭 계약 ID를 유지하며 하위 구성은 접미 ID 부여 |
| 참조 | manifest 선택 경로·버전·원본 크기·crop x/y/w/h·확대율. 미측정은 미측정으로 표시 |
| 형태 | 배치 순서·정렬·폭/높이·padding/gap·radius·stroke·glyph/텍스트 크기·색 토큰 |
| 크기 기준 | 원본 raster pixel, 정규화 배율, 최종 논리 단위를 분리. 전체 창 배율만으로 추정하지 않음 |
| 동작 영역 | 보이는 도형과 hit rect·focus rect·drag rect를 별도로 기록 |
| 상태 | normal/hover/pressed/focus/selected/disabled, 데이터 없음/입력 필요/실행/실패/완료 |
| 구조 | Rust 컴포넌트와 Flexbox/Grid 부모·축·grow/shrink·overflow 책임 |
| 데이터 | typed SDK 필드·안정 ID·단위·미상 값·원본 revision·DesignSample 여부 |
| 실행 계약 | TSX Props 선언, Rust 명시 producer, binding 이름·서명, 공통 후처리/feedback 경계 및 exact 회귀 시험의 동일 변경 묶음 |
| 반응형 | 1440/1280/1180/980/760의 재배치·숨김 대신 접근 경로·스크롤 |
| 자산 | SVG IconId 또는 등록 raster AssetId·폰트. 임의 경로 로딩 금지 |
| 비교 | 참조와 실제 같은 요소의 1:1/2x/4x 근거, 관측 일자·실행 빌드·상태·차이·판정 |
| 결론 | 일치/의도적 공통화/불일치/미검증. 의도적 차이는 공유 계약과 이유를 연결 |

## 완료를 막는 누락

- 제목줄과 최소화·최대화·복원·닫기를 비교 목록에서 제외하지 않는다.
- 화면의 전용 rail·툴바·보조 미리보기·레이어·파일 tree·파형·검증 목록이 빠졌으면 일반 미리보기가 렌더돼도 미완료다.
- 모델 미설치·입력 없음 때문에 결과 영역을 제거하지 않는다. 구조와 입력 안내를 보여주고 실제 수행하지 못하는 명령은 이유와 함께 비활성한다.
- headless render·테스트·Clippy 성공은 컴파일/동작 근거이며 시각 판정을 대신하지 않는다.
- 기존 이미지 간 충돌은 공통 규격으로 결정하고 기록한다. 생성 오류·샘플 숫자·가짜 진행률을 규격으로 고정하지 않는다.
- 데모만 확인한 상태를 실제 추론·저장·재생 성공으로 보고하지 않는다. 에이전트의 검토용 player는 소유 PID·경로를 확인해 종료하고 사용자·다른 세션의 실행은 보호한다.

## Props·콜백 계약 변경 인계

2026-10-25 PM 지시에 따라 props나 콜백을 추가·수정할 때는 같은 후계에 다음 항목을 모두 포함한다. 기존 검증 묶음에 부분 변경을 섞지 않는다.

- 실제 소비하는 TSX 선언과 Rust의 명시 producer를 함께 바꾼다. 다른 컴포넌트의 전체 props 맵을 상속한 뒤 알려진 필드를 계속 삭제하는 임시 수리는 사용하지 않는다.
- callback 이름·Text/Action 등 서명·원본 queue/node/input token 공급원을 대조한다. SDK 정책은 복제하지 않는다.
- 초기·working draft·편집/저장 revision·검증/SDK 오류·판정/이력 상태를 실제 strict NodeSlot 조립 계약으로 검증한다. required key 누락과 임의 extra key 거부를 함께 유지한다. 선언을 읽어 자동 retain하거나 미지의 필드를 조용히 버리지 않는다.
- bootstrap, 공통 feedback, transport pending, raw/output review 등 후단 producer도 포함해 최종 전달 맵을 검사한다. 중간 함수의 props 일치만으로 완료 판정하지 않는다.
- 정확한 선임 manifest·후계 hash·소유 leaf·실행 gate 결과를 기록한다. 문서 설계, 소스 구현, 타입/시험 실행, GUI 연결·시각 인수, 사용자 품질 승인은 각각 구분한다. 실패 이후 미실행 시험을 통과로 합산하지 않는다.

컴포넌트 계약 정본은 현재 선택된 소스와 연결하고, 변경 기록/계획은 정본을 링크한다. 과거 수리나 설계를 현재 검증 상태로 재사용하지 않는다. [ManualRepair 계약](../studio/manual-repair-projection-contract.md)은 이 규칙을 적용한 현재 사례다.
