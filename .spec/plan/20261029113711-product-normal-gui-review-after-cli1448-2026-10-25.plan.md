---
id: plan:kinekino/20261029113711
title: source1448 공통 CLI 이후 정상 player UX 검토 최소 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# source1448 공통 CLI 이후 정상 player UX 검토 최소 계획

PM 요청에 따른 계획만 작성. 새 소스/Cargo/GUI/프로세스/사용자 저장소 쓰기 없음. source1448 전체 SHA는 `1448D972F9A7C024F5ED35821783A8956239CF8A3DEF0BB4852E7BD6513E2A95`다. 중앙 정적 검사/71개 및 실제 Airing owner Ready·activated·native position 증가 성공은 PM 전달 사실이며, completed→movies→adjacent는 전달 시 진행 중이다. 이 계획은 그 후속 성공을 선계상하지 않는다.

## 실행 전 단일 인계

공통 CLI 필수 범위 결과·최종 소스 pin·최신 custody/반환·중복 player 없음과 **정상 player.exe producer/hash**를 PM/SDK에서 받는다. 기존 검증 바이너리를 다시 빌드하거나 실행하지 않는다. 이번 정상 사용자 GUI 인수는 사용자에게 보이는 주 모니터의 기존 정상 창 기본 배치를 우선하며 DPI·가상 장치 설정을 변경하지 않는다. 지원되지 않는 placement flag를 만들지 않는다. 특정 검증 recipe에 명시된 가상 모니터 조건은 그 recipe에만 적용하고 이번 사용자 인수의 필수 조건으로 승격하지 않는다. 추가 GUI 실행 배정은 공통 CLI 종료 뒤 PM이 담당한다.

이번 읽기 확인의 경로 상태:

| 항목 | 정확 경로 | 확인 범위 |
|---|---|---|
| 정상 exe 인수 | %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native/debug/player.exe | 161437696B, UTC mtime 2026-10-25T01:56:02.3104772Z. Media가 원22 provenance 1A46282495614ABF36D29BF559D3F43EA9AB2C7FB7417B6B155AACCA141D09DC와 metadata/FI를 대조해 source1448/SHAE4B866F874351D79EE964009A45AE2C447B79DAE6F1EBBA17A08008C9D9D728F를 인수했다. 이번 대형 재해시/실행 없음. |
| 실제 Airing 검증 host | %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/product-review-next260-checks-r3-gate/bin/kinekino_player-de5b10d79d4a860f.exe | 중앙 artifact metadata 183775744B/SHA5A54D9F1715BBF6E59957A3EF63A9627930EB04EC9E2F105FBE773B6658B8F55. libtest/production assembly이며 정상 player 대용 실행 금지. |
| 원 CLI | %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native/debug/player-cli.exe | 중앙 metadata SHA1CE000727D77C3FCEE15F48F40315D1C3345CBCADD6E530510BE6C6A530B8DFC. 이 계획에서 실행/재해시하지 않음. |
| mpv | D:/workspace/@myso-kr/com.kinekino.player/target/debug/mpv.exe | 존재/121051136B stat. 기존 expected SHA f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790은 중앙 TrustedMpvExecutable 검증 계약을 따르며 이번 재해시 결과 아님. |
| historical verifier | D:/workspace/@myso-kr/com.kinekino.player/runtime/quality-review/historical-import-2026-10-19-r1/player-review-sdk-export.exe | 존재/1127424B stat. 기존 expected SHA e00c1726603ef7d0167c6d58deb79e694071044c20a2723de8abe8a504f45e41. 모델 추론 worker 아님. |
| 실제9 review root | %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/product-gui-real-ai9-input-r1/review-root | 디렉터리 존재만 확인, 내용/사용자 판정 변경 없음. |
| 실제9 proposal root | %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/actual9-historical-import-r1/proposals | 디렉터리 존재만 확인, 내용 변경 없음. |

직접 읽은 중앙 admission: `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/product-review-next260-airing-r1-gate/06-admission.json`은 source1448, retry0, normal_player_execution=false, gui_host original fixed libtest production assembly를 명시한다. 따라서 Airing 성공을 정상 exe 사용자 입력 시험 성공으로 바꾸지 않는다. airing-frame.png는 원 mpv 영상 프레임이며 GUI chrome/layout 캡처가 아니다.

원 실행 argv 정본은 [NORMAL-PLAYER-AI-REVIEW-INPUT](../../../runtime/coordination/ai-review-input-r1/NORMAL-PLAYER-AI-REVIEW-INPUT.md) 최신 SHA BC525908B4C677CBB9CCD45D18C4C1FC484C675DAECA5930BD05DEA4AC832644다. 현재 정상 exe 절대경로가 채워져 있고 `--native-viewer --record-validation` 및 jobs/catalog/browser-profile/review/proposal/historical 옵션을 그대로 사용한다. 독립 launcher/자동 proposal 선택/실행 SHA flag/추론 worker 옵션을 추가하지 않는다. 라이브 jobs/catalog/browser-profile은 원 packet의 normal-product-review-r1 아래 세 별도 경로를 사용하며, 실제9 root와 혼합하지 않는다. 이 세 경로는 현재 미존재로 PM 생성·소유권 배정이 남아 있다. 종료된 actual Airing 검증 root를 임의 재사용하지 않는다. 최신 parser 변경 여부는 SDK가 확인한다.

Media 최신 USER-REVIEW-HANDOFF는 5FCD5F4CB02A0D5AA5D8091C01E3FB903A9547F1ED1B5C5ADE9BACE690697D35, 경로10개 대조 NORMAL-PATH-EXISTENCE.json은 200FD2D9D38FF222751B95A5134823525410BB04244D5A5FF931937BDACEF496이다. 존재7/미존재3이며 정상 binary 인수와 실행 준비 완료는 구분한다. 이번 계획 작성자는 디렉터리 생성·GUI 실행·사용자 저장소 쓰기를 하지 않았다.

후속 PM 인계: normal-product-review-r1/{jobs,catalog,browser-profile} 세 디렉터리를 PM이 직접 생성하고 정상 GUI 원 owner 전용으로 배정했다. 작성자도 세 경로의 존재를 읽기 확인했다. 앞의 미존재3 조건은 당시 경로 점검 결과이며 현재는 해소됐다. 실제9 두 root·종료된 검증 roots와 분리는 유지한다. 현재 SDK adjacent actual 검증이 진행 중이며 반환 뒤 PM의 별도 GUI 실행 배정이 남는다. 경로 준비를 실제 GUI 실행 성공으로 표시하지 않는다.

## 한 정상 창에서 묶어서 확인

1. 원 Native 창·영상·상태·실제 callback 연결을 확인하고 **GUI 전체 레이아웃 캡처**를 별도로 남긴다. 영상 프레임만으로 시각 일치 인수하지 않는다. 기존 시안과 헤더·캡션·간격·토큰·컨트롤 위치를 확대 비교한다. 아직 disabled인 최소화/최대화/닫기 chrome을 지원으로 표시하지 않는다.
2. 현재 공통 SDK에 연결된 재생/일시정지·μs 탐색·음량 입력은 사용자 입력과 결과 표시 정합성만 확인한다. CLI가 이미 검증한 저장/CAS/취소/변환 기능을 GUI에서 중복 인수하지 않는다. 이전/다음·목록 표시는 중앙 category/adjacent 검증 결과를 전제로 UI 활성/오류·현재 제목 정합성을 확인한다.
3. 전체화면 버튼 요청→pending→원 ActionRecord Observed 및 실제 borderless coverage를 구분하고 해제까지 확인한다. 수락/Issued/클래스 변경은 성공 증거가 아니다. 회차 패널은 토글만 바뀌고 SDK owner·진행 작업을 유지해야 한다.
4. 관측된 fullscreen+focus+inside+닫힌 패널에서 실제 입력 후 3초를 기다린다. 3초 전 숨김 금지, 2.9초 새 입력 뒤 이전 completion으로 조기 숨김 금지. 실제 포인터·키 입력 복원, focus/leave/route/close 복원과 stale hide 거부를 묶어서 본다. cursor Applied는 setter 반환이며 OS 가시성 실측과 구분한다. DOM target별 hover 고정 억제는 아직 API 한계로 별도 기록한다.
5. 같은 정상 창의 chooser에서 `actual9-review-r1-gemma270-correction`을 명시 선택한다. package `659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197`, item `gemma270-correction`으로 원문/후보/model/기준·이력 표시를 확인한다. 다른8개는 Media USER-REVIEW-HANDOFF의 원 목록을 따른다. developer는 실제9 수정/저장/채택/verdict를 쓰지 않는다. 사용자 명시 품질 판정 단계는 그 뒤 별도다.
6. 사용자 직접 종료 후 원 cleanup/cursor/worker 상태를 기록한다. 실패한 입력은 원 오류·범위·스크린샷과 함께 구분하고, 같은 실패를 무계획 반복하지 않는다.

상세 체크리스트는 [GUI 인수 소스 정본](../../sots/verification/presentation/product-r4-gui-acceptance-source-audit-2026-10-25.md)을 따른다. 제품 모델4/Native11/CPU paint 통과는 실제 창 입력·커서·사용자 AI 품질 승인 근거가 아니다. 이 계획 작성 시 정상 GUI 입력/사용자 승인0 유지.

적용: 기존에 읽은 modoo.today defect-triage-evidence/test-fixture-and-mock-build의 provenance·원 namespace·소스/실행/승인 구분. 새 코드·실행 도구를 만들지 않았다.
