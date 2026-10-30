---
id: sot:kinekino/viewing-transport
title: Viewing 일상 transport와 문제 해결 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Viewing 일상 transport와 문제 해결 계약

VIEW-03/06/07/08/09와 [공통 요소 검토](../shared/element-review-contract.md)를 적용한다. 기존 egui r97/r128 등 관측은 역사적 근거이며 현재 Meari 제품 실행 증거가 아니다.

## 현재 구현 후보와 상태

별도 private [source16](../../../../../runtime/development/product-viewing-everyday-ux-r1/HANDOFF.md), SHA256 `ECE5632F894D03F3361BA2CB2D8C12B05EC546AB549302A328DEFB51B30BBCBE`. 선임720AA4F17C15955A2A7FC3ECBCC15CF72FA3DD024DFAFE43EBB09929E033A137 / canonical Root77fd593e. 기존15 leaf 교체와 시간 presentation 모듈1개 추가다. retirement source6/guard2/redraw4와 SDK source15, 미검증 range 후계는 이 선택에 포함되지 않는다.

소스 구현·회귀 작성 및 rustfmt/check 성공 단계다. 타입·Clippy·시험·새 GUI·시안 1:1 인수는 미실행이다. source267r3의 기존27 통과를 이 Viewing 후계의 성공으로 재사용하지 않는다. 이후 중앙 합성은 변경된 선임을 재확인한다.

| 요소 | 원본 공급/입력 | 현재 소스 계약 |
| --- | --- | --- |
| VIEW-03 transport | 기존 onNativePlay/Pause Action + 원본 instance/admission | 공통 play/pause icon button, 상태 텍스트. SDK 상태/제어 정책 불변 |
| VIEW-06 시간 | 원 connected observation position/duration u64 μs | nativePositionLabel/nativeDurationLabel string. H+:MM:SS.mmm, 밀리초 절삭으로 끝점 초과 표시 방지. 미관측/단절은 미확인, 0초로 성공 위장하지 않음 |
| VIEW-06 고급 seek | 기존 onNativeSeekDraft/Text → onNativeSeek/Text | H:MM:SS[.ffffff]를 Rust checked 정수로 정확 u64 μs 변환. ASCII·분/초 범위·overflow 검사, 원 SDK validator/instance 확인 유지. 초과 정밀도 반올림/암묵 raw-us 입력 없음 |
| VIEW-07 음량 | 원 connected volume observation와 기존 독립 draft | nativeVolumeLabel string, 분수 백분율 보존. 초기 입력100은 관측값 아님. numeric Apply는 고급 임시 도구이며 최종 rail UX 아님 |
| VIEW-08/09 | 기존 회차/전체화면 Action | 원 공통 아이콘·상태·disabled·오류 표시 유지. 실제 fullscreen/cursor/입력 복원은 별도 GUI 인수 |
| VIEW-10b 문제 해결 | diagnosticsExpanded boolean + onToggleDiagnostics/Action | 원 local queue만 사용, 초기 접힘. transport와 sidebar/catalog가 같은 상태를 공유. SDK ticket 생성/자동 재제출 없음 |

추가 root props4개는 diagnosticsExpanded/boolean 및 nativePositionLabel/nativeDurationLabel/nativeVolumeLabel/string이고 callback1개는 onToggleDiagnostics/Action이다. 초기 read_props/native_default_props, current ReadProjection/NativePlaybackProjection, TSX root/child 선언, 원 binding signature와 회귀를 같은 source16에 포함한다. 원 raw μs/volume props는 고급 진단에 남긴다.

## 상태·복구와 공통 자산

일상 영역은 재생/일시정지·관측 시간/음량·회차·전체화면·명시적인 문제 해결 진입을 제공한다. 개발용 capability/engine/status 조회, ACK·불확정 복구와 수치 입력은 고급 영역에 둔다. Catalog의 poll/read-series/read-episodes/inspect/play 기술 단계는 원 SDK/자동 진행 소유자가 처리하며, 고급 패널을 열어도 최종 사용자 DOM의 개별 수동 단계 버튼으로 다시 노출하지 않는다. 관련 원 SDK 등록·typed 계약·자동 progression은 보존하며 표시 제거를 SDK 지원 삭제로 해석하지 않는다. 취소와 정상 탐색·상태·오류는 일상 경로에서도 접근할 수 있다. 고급 영역의 나머지 버튼은 기존 disabled/동일 ticket/불확정 결과 재제출 금지 규칙을 그대로 따른다. 새 외부 기능이나 진단용 SDK를 만들지 않는다.

2026-10-25 중앙 r4/source37541DB6D...의 actual factory30은 diagnosticsExpanded=true에서도 기술 단계 binding을 금지하는 원 assertion에서 실패했다. 고급 구역의 Catalog5버튼이 원 onCatalog*로 연결된 표시 계약 위반을 확인해 [후계1](../../../../../runtime/development/product-catalog-final-actions-r1/HANDOFF.md), source1 SHA `C445C58B4BFFAB71F2DE94FF7D091D6D16E9359E2222437AF6BAE24DF36C2728`로 DOM 버튼만 제거했다. 검사·자동 진행·SDK/owner 정책·190 목록은 유지한다. 이는 source-only 수리이며 해당 후계의 실제 factory/GUI 통과는 아직 미검증이다.

seek 초안은 같은 instance/artifact 관측 갱신 중 유지하지만 교체·관측 상실 시 버린다. 원 SDK 저장/제어 정책은 application/domain이 소유한다. 음량 draft는 기존 시작 음량 입력과 공유하는 별도 의도이며 관측 음량으로 계상하지 않는다.

Button, Pretendard/fallback, 색·간격·border·radius·아이콘 크기는 기존 공통 토큰을 사용한다. 기존 product pause.svg를 CSS/임베드 allowlist에 연결하고 play/volume/list/fullscreen과 같은 자산 경로·검증 방식을 사용한다. 새 SVG 생성/다운로드/기존 자산 삭제 없음. Meari 공식 공통 자산 전환 완료나 실제 glyph 인수로 보고하지 않는다.

## 검사와 미완료

작성된 회귀는 시간 unknown/큰 시간/끝점/u64/형식, 실제 원 callback의 잘못된 입력 거부와 μs/instance payload, 초기/connected/disconnected 표시, 초안 instance 변경, local toggle·SDK 요청 불변·닫힌 scope 거부를 포함한다. 실제 Root/NodeSlot factory에 required/extra/wrong-type/wrong-signature negative와 advanced 활성 상태의 원 callback 캡처를 추가했다. 실행 결과는 아직 없다.

Native range A7/B는 Meari 담당 구현·검증 이후 별도 소비자 계약으로 통합한다. PointerUp/KeyUp 최종 commit, 취소, ordinary value/표시 props 중 원 gesture owner 연속성을 확인하기 전 슬라이더 지원/최종 PotPlayer UX 완료를 표시하지 않는다. WindowInputActivity를 release 대신 사용하지 않는다.

용량과 전달량은 별도 책임이다. canonical Meari0844의 host_callbacks/document.rs 큐 한도는1024건/4MiB이며 DocumentCallbacks::drain은 pending 전체를 반환한다. NativeHostRuntime은 이를 PendingTurn에 보유하고 원 admission 결과로 진행/보류한다. forwarding.rs160의32는 early_notifications 단위시험 반복 횟수이며 턴당 drain32 제한 코드가 아니다. 앞선32건 설명은 이 canonical 관측으로 정정하며, 별도 private 후계가 있다면 exact path/pin 확인 전에는 지원 규격으로 계상하지 않는다. 큐 용량 확대나 남은 콜백 소유권 변경 없이 원 후속 turn/wake 계약을 유지한다.

정정 이력(2026-10-25): PM의 초기32건 해석을 문서에 반영했으나, Meari 담당 이의 제기와 양측 원문 재열람 후 PM이 해당 요구를 철회했다. 현재 규격은 queue1024/4MiB, 원 Capacity 응답 시 PendingTurn tail 보유, 원 Completion 후속 전달 유지다. source16 변경이나 새 실행을 동반하지 않았다.

시안과 normal/hover/pressed/focus/disabled·좁은 폭·긴 번역·실제 cursor/소리/키보드·전 화면 인수는 [최종 normal GUI 계획](../../../../plans/player/normal-gui-final-batch-acceptance-2026-10-25.md)에서 별도로 수행한다. 모형/공통 SDK 시험은 이 결과를 대신하지 않는다.
## 후속 range 소비자

[Range 소비 계약](range-consumer-contract.md)은 별도 source-only 후계다. 기존 UX16의 text-Apply가 최종 슬라이더 UX를 대신하지 않으며, 새 후계도 통합 실행·정상 창 인수 전에는 지원 완료로 판정하지 않는다.

