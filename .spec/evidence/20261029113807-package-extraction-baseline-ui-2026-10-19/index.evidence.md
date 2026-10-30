---
id: evidence:kinekino/20261029113807
title: 패키지 순수 분리 고정 GUI 실제 기준 — r168
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 패키지 순수 분리 고정 GUI 실제 기준 — r168

## 검증 분류 정정

최신 사용자 기준에 따라 이 기록의 GUI 변환·손실·덮어쓰기 조작은 화면 표시/사용성/API 연결 관측으로 분류한다. 변환 정확성·원문 보존·파일 publication/CAS 등 기능 판정은 SDK 테스트와 고정 player-cli 실행 로그 `runtime/quality-review/package-extraction-r1/cli-conversion` 및 [SDK 검증 정본](../application/package-extraction-2026-10-19.md)을 근거로 한다. 아래 실제 파일 읽기는 연결 결과를 대조한 보조 관측이며 독립 기능 완료 판정이 아니다. 이후 GUI에서 같은 기능을 반복 실행하는 대신 고정 CLI 결과/fixture의 표시 정합성·레이아웃·키보드·포커스를 검토한다.

## 실행 식별

`runtime/review-runs/ui-package-baseline-20261019-r168/player.exe` SHA256은 `75c8a2d1be79fa27d71d05040081d9bbc85eeb25b751d77cd336728f9055020b`이다. CLI는 `a22a78611936402c5d0862748a71e3cbf0af3d92b466979835f06618260dded1`. SDK 순수 분리 checkpoint의 fmt/test155 passed·9 ignored/clippy/build 성공과 구별하여 실제 GUI 관측을 기록한다. 새 focus 소스 수정은 이 실행본에 포함되지 않는다.

이전 r164/r167 소유 창이 없음을 fresh 목록에서 확인한 뒤 메인 위치100,100에 실행했다. PID75472/window8917314, 제품 GUI 한 개다. 콘솔은 숨김 실행하고 제품 GUI를 명시적으로 활성화해 사용자가 보는 메인 모니터에서 조작했다. VM·DPI 변경은 없다. r164/store와 변환 입력을 r168에 복사했으며 원본 저장자료는 수정하지 않았다.

## 실제 자막 변환

시작→제작1440×900→SUB 변환 탭을 클릭했다. 입력 original.srt 전체 경로와 신규 출력 gui-converted.vtt를 직접 입력하고 SRT→WebVTT로 새 파일 변환을 클릭했다. GUI는 자막 변환 완료·자막1개·70bytes·UTF-8 새 파일 저장 완료를 표시했다. 실제 출력은 WEBVTT, `00:00:01.001 --> 00:00:03.456`, 한글·일본어와 두 줄 본문을 보존했다. 원본 복사본 SHA는 `6699e55cf3f0936a97e96e23219c8c55e61e1021b10b65110de476a87ec4b6b6`으로 SDK 원본과 같다.

같은 출력 경로로 다시 변환을 클릭했다. 처리 중 필드/실행 비활성화 후 자막 변환 오류·`Output already exists; choose a new path`가 표시되고 경로 입력은 보존됐다. 기존 출력 SHA `64fdfb671c5b6cc22add85ab4a87a91d48c285f4892c2347d25bc485d632bfad`는 전후 동일했다. 오류가 발생하면 기존 완료 표시 대신 현재 결과 영역이 비워지는 동작도 확인했다. 실제 미디어를 연결하지 않았으므로 preview/transport는 미연결 상태다.

## 남은 확인

후속 이전 저장자료 실제 읽기: 같은75c8 GUI에서 SUB교정의actual-gemma270-correction ID입력→Tab으로제안열기focus→Enter조회가 동작했다. source revision1/draft3·assetSHA·outer/extraction·inner/5·원문과수정문·receipt3·재채택비활성이 복원됐다. 이어 CARD actual-llama1b-cards를 조회해 앞면/뒷면 한국어·일본어와 cue1선택·원문cue1…7·receipt3·재채택비활성을 확인했다. 새save/accept로 저장자료를 바꾸지 않았다. 따라서 r168은 이전 S1 cue/S2 document 저장자료의 실제 GUI 읽기·채택상태 복원 근거를 포함한다. 순수bin으로 새편집/재채택/추가재실행은 아직 하지 않았으며9전체GUI확인으로 확대하지 않는다. 두화면의우측기존inspector 연결필요불일치는 재현됐다.

후속 손실 흐름을 실제 확인했다. with-loss.vtt 입력·WebVTT 형식·신규 gui-loss-reviewed.srt/SRT 출력을 지정하고 변환을 클릭했다. 확인 필요 상태와 미선택 확인 checkbox·비활성 변환 버튼이 나타났다. 본문 스크롤에서 메모/스타일/영역 메타데이터, 식별자, 위치/영역 설정, 화자 구조의4개 손실 항목을 확인했다. 이 시점 Test-Path는 출력false였다. 검증용 복사자료의 확인 checkbox 클릭 후 버튼이 활성화됐고 별도 손실 수락 후 변환 클릭으로 완료1cue50bytes/새 파일 저장을 확인했다. 이 검증용 선택은 제품 모델/사용자 콘텐츠 품질 승인이 아니다. 실제 SRT에는 시간과 한글·일본어 본문이 남고 NOTE/cue ID/align/화자구조는 유지되지 않는다. 원본 WebVTT는 보존됐다. 손실 경고의 SDK 영문 remediation이 그대로 표시되는 점은 한국어 UI 정리 대상이다.

일반 왕복 변환·순수 분리 bin에서 새편집/재채택/별도재시작·9전체GUI 확인은 아직 하지 않았다. Source1/Source2 이전 저장자료 읽기·기존receipt 복원은 위에서 실제 확인했다. 이전 r164/r167 근거를 이 SHA에 소급하지 않는다. 긴 경로가 단일 입력 안에서 잘리는 모습, 문법 비교 대신 경로/완료 요약만 보이는 현재 비교 패널, 원문/제안 inspector 불일치와 본문 밀림은 시각 미완료다. focus 수정 소스 전체 gate와 새 fixed EXE 실제 Escape→Enter는 별도 checkpoint다.
