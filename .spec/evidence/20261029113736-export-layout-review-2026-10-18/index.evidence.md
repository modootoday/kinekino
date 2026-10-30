---
id: evidence:kinekino/20261029113736
title: OUT 저장 확인 레이아웃 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# OUT 저장 확인 레이아웃 검토

2026-10-18. 기준 export.png 전체와 확인사항(1011,552,537,350)2×, 하단 동작(1015,801,529,59)4×. 시안은 요약/저장 확인 두 영역이며 별도 세 번째 검사기가 없다. 이 기록은 전체 OUT 시각 인수가 아니다.

## 기준 실행

r78(PID59704), `runtime/review-runs/ui-export-layout-20261018-r78/player.exe`, 1440×900에서 중앙 두 card와 우측 출력 설정이 별도로 표시됐다. 출력 위치·파일 충돌·출력 형식·저장 전 검증 값이 중앙 확인사항과 우측에서 중복됐다. 요약과 저장 card가 각각 약460폭으로 압축됐다. 충돌 radio 두 개는 설명 없는 작은 행이었다. r78은 캡션 닫기로 종료했다.

## 수정 책임

OUT는 자체 요약/저장 확인 화면이므로 shell의 고정 inspector와 설정 drawer/버튼을 생성하지 않는다. 중앙 destination이 typed snapshot의 확인사항을 한 번만 표시한다. 다른 화면의 inspector는 유지한다. 화면별 별도 shell을 복제하지 않고 기존 shell의 inspector 존재 여부만 분기한다. 사용하지 않는 OUT inspector 렌더 분기는 제거했다.

충돌 옵션은 공용 described_choice로 제목/설명·32높이 radio·들여쓰기를 공유한다. 실제 collision policy는 미연결·미선택으로 유지하고 새 후보명·실제 동명 충돌·쓰기 성공을 만들지 않는다. 980 미만은 기존 요약→저장 한 열, 동작은 한 destination에 유지한다.

## 수정본 실제 결과

r79(PID103444), `runtime/review-runs/ui-export-layout-20261018-r79/player.exe`에서 확인했다.

- 1440×900: 별도 출력 설정 inspector가 없고 요약/저장 card가 중앙 폭 전체를 나눴다. 확인사항은 저장 card에 한 번 표시됐다. 두 충돌 radio 아래에 각각 보존/새 이름·덮어쓰기 설명이 줄바꿈되며, 미선택·비활성을 유지했다. 하단 세 동작과 로컬 저장 안내가 card 안에 표시됐다.
- 760×480: OUT에 설정 버튼이 없고 요약→저장 확인이 적층됐다. 본문을 끝까지 스크롤하면 확인사항과 취소/뒤로/저장 시작 세 동작 모두 접근됐다. 이 동작은 미연결·비활성이므로 실행 성공을 의미하지 않는다.
- 760 NeedsInput: 출력 형식·저장 전 검증은 `—`로 바뀌고 비활성 저장을 유지했다. 화면이 샘플 충돌 정책 문자열을 보여 주는 것과 실제 collision policy 선택은 별개이며 실제 검증 결과가 아니다.
- 다른 화면 회귀: SEG로 이동하면760 설정 버튼과 drawer가 나타났다. drawer를 연 상태에서1440으로 바꾸면 SEG 고정 inspector가 표시됐다. 이어 OUT 진입 시 inspector가 제거됐고 Escape 한 번으로 디자인 데모가 종료됐다. 숨은 설정 상태가 첫 Escape를 소비하지 않았다. 실제 시청 복귀는 이 데모 종료 관측과 구분한다.
- 1440→760 전환 직후 egui request_discard 연속 프레임 PERF WARNING이 한 번 관측됐다. 다음 안정 캡처에서는 사라졌다. 지속 오류로 단정하지 않으며 전환 프레임의 시각 완성도는 미완료다.

최종 fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드 통과. 숨은 설정 상태 해제 변경 후 전체 검사를 다시 통과했다. 외부 미디어 조건으로 ignored인 기존 검사는 실행 성공으로 취급하지 않는다. 종료 후 player/mpv 프로세스 조회는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다. Empty/LongLabels·980/1180과 실제 시청 복귀는 이번 실행 표본 밖이다. 하단 버튼 문자의 중앙 정렬과 info glyph도 추가 대응이 필요하다.

## 남은 시각·동작 범위

요약의 출력 항목별 thumbnail/원본 범위·속성 glyph/transport·확인사항 의미별 glyph와 실제 preflight 데이터는 남는다. 실제 경로 선택·파일명 검증·충돌 처리·저장/취소·receipt는 미연결이다. 시안의 성공 값·후보 이름을 검증 근거 없이 채우지 않는다. 시청 화면과 나머지 제작 화면 전체 검토도 계속 진행한다.
