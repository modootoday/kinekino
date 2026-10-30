---
id: evidence:kinekino/20261029113851
title: 자막 변환 문법 비교 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자막 변환 문법 비교 검토

2026-10-18 SUB 변환. 기준 convert.png 전체와 비교영역(263,501,899,328)2×, 첫 행(265,546,442,138)4×를 검토했다. 파생물 sub-convert-panels-r93-2x.png/sub-convert-row-r93-4x.png. 시안은 번호 rail·시각/본문·문법 블록·쌍의 선택 표시와 하단 안내가 구분된다. ASS 지원은 실제 능력이 아니므로 구현하지 않는다.

## 변경 전 실제 실행

r93(PID30620), `runtime/review-runs/ui-sub-conversion-20261018-r93/player.exe`1440×900에서 비교는 선택 한 cue의 일반 텍스트 카드였고 문법/번호가 없었다. 출력 카드가 correction 열의 ‘교정 제안 예시 · 미적용’을 변환 출력으로 표시하는 오류를 확인했다. r93은 캡션 닫기로 종료했다.

## 구현과 데이터 경계

CueComparison은 표시 전용 cue ID/번호/시각/본문/문법을 받아 공용 selectable_row로 렌더한다. 양쪽 패널은 같은 selected ID,980미만 적층, 전체 행 선택/focus, 문법 monospace를 공유한다. 원본·출력을 일반 교정 표로 다시 나열하지 않는다. 공용 notice는 실제 손실 검토 연결 필요를 설명한다.

screens/conversion/sample은 DesignSample에만 고정된 세 시간 범위의 WebVTT/SRT 문법 예시를 조립한다. snapshot의 typed start/end가 고정 범위와 일치하고 original Text가 있을 때만 해당 쌍을 만든다. correction 열은 사용하지 않는다. 이는 별도 converter나 실제 파일 결과가 아니다. Populated/LongLabels/NeedsInput 예시를 기존 domain parser로 양쪽 읽어 동일 cue 시간/본문을 검증했고 Live origin에는 예시가 생성되지 않는 것을 확인했다. 실제 converter command/receipt/storage와 정상 UI 경로는 변경하지 않았다. 실제 syntax/loss preview 계약 필요 사항을 SDK 세션에 전달했다.

## 실행 검증

최종 cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo build --bin player, cargo test가 통과했다. 라이브러리67통과/1ignored, CLI·통합 통과이며 기존 외부 미디어 ignored는 실행 성공이 아니다. 실제 파일 변환·저장 완료나 전체 SUB 시각 인수를 주장하지 않는다.

## 수정본 실제 실행 r94

`runtime/review-runs/ui-sub-comparison-20261018-r94/player.exe`(PID111304)를 기존 가상 모니터3500,100에서 한 개만 실행했다.1440×900에서 원본 WebVTT/출력 SRT 각3개, 번호·시각/본문·문법 배경 블록과 점/쉼표 시각 차이를 확인했다. 출력 두 번째 cue 클릭은 양쪽 두 번째 선택선/번호를 표시했고 Down은 양쪽 세 번째 선택 및 출력 행 focus·자동 스크롤로 이어졌다. 교정안 문구는 출력에 표시되지 않았다.

변환 탭 첫 전환 직후 한 번의 capture에서는 글자가 세로로 잘게 줄바꿈되고 검사기 폭도 흔들렸다. 추가 입력 없이 다음 capture에서는 정상 폭으로 안정됐다. 전환 순간의 시각 결함이 제거됐다고 주장하지 않고 남은 범위로 기록한다.

760×480에서는 원본·출력이 세로 적층됐다. 스크롤로 출력 첫 cue를 클릭하고 End를 눌러 세 번째 출력 cue와 하단 안내까지 자동 이동했다. 설정 열기/Escape 후 세 번째 선택과 스크롤이 유지됐다. LongLabels의 원문과 syntax가 패널 폭 안에 표시됐고, Empty에서 cue·선택·문법 블록이 제거되며 미리보기 연결 필요 안내를 확인했다. 원본/출력 패널 개수는 원문 문서의 실제 cue count를 의미하지 않는 디자인 표본이다.

r94는 캡션 닫기로 종료했고 후속 CIM에서 player.exe/mpv.exe가 없었다. VM·DPI·모니터 구성은 변경하지 않았다.

## 남은 범위

실제 문서 dry preview·format/revision/loss/read 계약, 정상 변환 화면의 공용 비교 컴포넌트 연결, 첫 전환 폭 안정화,980/1180·1440 LongLabels,Home/Up·원문 hover/OS 접근성·실제 미디어 시각 동기화는 미검증/미완료다. 검사기 옵션/경고·내보내기 실제 능력도 후속이다. 생성 시안의 ASS 문법 지원이나 손실 경고를 실제 지원/분석 결과로 복제하지 않는다.
