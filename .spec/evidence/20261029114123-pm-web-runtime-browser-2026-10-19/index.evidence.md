---
id: evidence:kinekino/20261029114123
title: 중앙 PM의 Web runtime 브라우저 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 중앙 PM의 Web runtime 브라우저 검증

## Q04 graph 추가 고정본 — 17시험

담당이 고정한 `browser-c298fd9b3a439556.wasm`의 SHA256 `42b254072eab3edcad3544e16070e21141c29f04952422697e4e3a4aa18444fb`와 localhost8000의 소유 listener PID85940을 실행 전에 확인했다. 새 숨김 IAB 탭8에서 자동 시험을 실행하고 fresh AX로 17 passed/0 failed/0 ignored/0 filtered, 0.19초를 관측했다. graph canonical SHA256·cycle 거부와 기존 DOM 시험이 포함된다. 추가 키 입력이나 JavaScript 평가·상태 주입은 하지 않았다.

실행 뒤 디스크 SHA는 사전 값과 같았고 탭8을 닫아 IAB 탭0개를 확인했다. 담당은 source/artifact 동결과 복사본 SHA 일치, 소유 runner 명시 종료를 보고했다. 고정 source/WASM·세부 결과는 [담당 관측](../../../../packages/meari/.spec/evidence/verification/build-artifacts-2026-10-19.md)에 있다. 별도 로컬 screenshot 파일 보존이나 실제 Native/Web compiler bundle 생성 완료를 주장하지 않는다.

## 최종 고정본 재검증 — 08시42분 KST

담당 세션이 중간 runner PID19028의 served artifact는 `99638c5e3df9600c2b6ce707f404f1e856bd1a1d11ac6e97960fbc6f34c77ffe`이고, 이후 controlled input 진단 수정으로 디스크 파일이 바뀌었다고 보고했다. 따라서 아래 최초 실행의 사후 디스크 hash를 그 served artifact 식별자로 사용하지 않는다.

별도 최종 runner PID68476/localhost8000의 listen과 실행 전 WASM SHA `c5d7dca0bc30798cd2c68874c1072f2ea9c22058ff1d0477149b897967a491a4`를 PM이 확인했다. 담당자는 해당 runner 시작 전 같은 hash를 읽고 검증 종료까지 source/build를 동결했다고 밝혔다. 기존 탭6은 정리된 상태여서 숨김 IAB 탭7을 새로 열었다.

- 최종 AX 결과는16passed/0failed/0ignored/0filtered/0.17s다. 시험명은 아래 최초 실행과 같은16개다.
- 실제 버튼 Meari Web increment에 표준 Enter를 입력해 Web count0→1, fresh AX 뒤 Space로1→2를 확인했다. 버튼 focus가 유지됐다. JS 평가나 내부 상태 직접 변경은 하지 않았다.
- 최종 화면을 브라우저 screenshot 도구로 캡처했다. 도구가 절대 파일 경로를 반환하지 않아 로컬 이미지 파일 보존을 완료했다고 주장하지 않는다.
- 실행 후 WASM SHA도 시작 전과 같았다. 소유 탭7을 닫고 IAB 목록0개를 확인했다. runner 원문·고정 WASM/source 보존은 담당 세션으로 인계했다.

이 최종 결과와 최초0.21s 결과를 서로 다른 artifact 실행으로 구별한다. 전체 CSS/provider/OS 접근성이나 전체 Q01~Q09의 완료로 확대하지 않는다.

## 최초 중간 실행

2026-10-19 08시41분 KST. UI 변환/HMR 담당 세션에서 브라우저 도구 연결이 없어 중앙 PM이 기존 시험 페이지를 확인했다. 새 테스트 코드나 JavaScript를 주입하지 않았다.

## 실행과 직접 관측

담당 세션의 `open_in_codex` 기록에서 `http://127.0.0.1:8000/`을 확인하고, 로컬 TCP 조회에서 소유 runner PID19028이127.0.0.1:8000을 listen하는 것을 확인했다. runner의 관측된 인수는 `browser-887d7a3ba17acc65.wasm`이었다. 기존 IAB 탭0개에서 숨김 탭6 한 개만 열었다.

접속 직후 Loading Wasm module이 표시됐고 다음 AX 관측에서 아래 결과를 직접 읽었다.

- running16tests /16passed /0failed /0ignored /0filtered /0.21s.
- DOM5건: dirty editor·focus·text node 유지, mount별 hook ID와 중복 소유자 거부, mixed hook 결과, keyboard story, dispatch 실패·외부 mutation 시 마지막 상태 유지.
- 기존11건: structured errors, UTF8 mapping, closure, shared state runtime, public WASM roundtrip, native plan contract, mixed hook order, media subscription, host viewport, observer 해제, component action/stale lifetime.
- 마지막 화면에는 Meari Web runtime, Web count0, increment 버튼이 보였다. 수동 추가 클릭은 하지 않았다.

검증 뒤 소유 탭을 닫고 IAB 탭 목록0개를 확인했다. 다른 사용자 브라우저 탭이나 앱을 종료하지 않았다. 담당 세션에 runner 원문·고정 source와 결합해 보존하도록 인계했다.

## 식별과 한계

실행 후 디스크에서 읽은 WASM SHA256은 `c5d7dca0bc30798cd2c68874c1072f2ea9c22058ff1d0477149b897967a491a4`다. 이 사후 hash만으로 전체 compiler inputs나 served bytes의 독립 인증을 주장하지 않는다. runner 로그·빌드 source 식별과 보존은 담당 세션의 추가 근거와 연결한다.

이 관측은 해당16개 브라우저 시험의 통과다. 공식 예제 전체 지원, 전체 Q01~Q09, 제품 player 연결, 모델 품질 통과를 의미하지 않는다. 전체 조율 순서는 [중앙 계획](../../../plans/player/session-coordination.md)을 따른다.
