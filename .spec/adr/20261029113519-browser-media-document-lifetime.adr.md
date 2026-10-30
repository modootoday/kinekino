---
id: adr:kinekino/20261029113519
title: 미디어 관측은 문서별 소유 WebView 수명으로 분리한다
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 미디어 관측은 문서별 소유 WebView 수명으로 분리한다

상태: SDK 설계 선택, 구현·실제 media 인수 전. 기존 ReadCatalog DOM 경로와 v3 실제3category 성공을 소급 변경하지 않는다. [현재 관측 감사](../../sots/player/infrastructure/browser-provider-media-observation.md), [application 계약](../../sots/player/application/browser-provider.md)을 따른다.

## 문제와 대안

응답 시점 current page 또는 URL 매칭만으로는 이전 문서의 늦은 응답을 새 document generation에 귀속할 수 있다. WebResourceRequested와 ResponseReceived의 request COM identity가 같다는 근거도 아직 없다. URL-only registry는 제외한다.

| 대안 | 장점 | 문제/판정 |
| --- | --- | --- |
| COM IUnknown request ledger | 기존 WebView 재사용·추가 연결 ID 불필요 | 두 이벤트 동일 identity 보장 미입증. 현재 선택하지 않음 |
| CDP Network request/frame/loader ID | 명시 network identity 사용 가능 | 별도 프로토콜·이벤트/본문 경계·기존 adblock과의 수명 통합 필요. bounded 본문 획득 미검증이므로 현재 선택하지 않음 |
| 문서별 새 소유 WebView·관측기 | COM pointer 추정 없이 캡처 instance가 ticket/document generation에 고정됨 | stage마다 생성 비용과 redirect/iframe의 명시 이동 필요. 증명 가능한 최소 수명 경계로 선택 |

## 선택과 불변 조건

ResolveEpisode의 명시 staged navigation마다 이전 캡처/관측기를 닫고 버린 뒤, 같은 소유 STA·격리 profile·hidden parent 안에 새 WebView를 만든다. 생성 전 ticket과 단조 증가할 document generation을 고정한다. 각 instance는 한 main-document admission만 허용한다. 이후 main-navigation/redirect 의도는 기존 instance에서 승인하지 않고 Rust worker/application이 URL 정책·남은 deadline·navigation 상한을 확인한 별도 stage로 전달한다. HTML iframe/player 이동도 Rust에서 추출해 명시 stage로 처리한다. 실제 episode 구조에 없는 자동 click/임의 URL fallback을 발명하지 않는다.

관측기 callback과 async GetContent closure는 고정 instance identity 및 closed flag를 가진다. 이전 instance가 닫힌 뒤 반환한 내용은 무효다. worker는 instance identity→현재 ticket/document generation을 다시 대조하고, application은 같은 identity의 terminal 결과만 채택한다. 각 stage에서 adblock filter 설치 성공을 확인하며 request/blocked count는 worker가 이전 stage 누계에 더해 단조 증가를 유지한다. counter 초기화나 filter 재설치를 첫 Started 처리만으로 검증했다고 하지 않는다.

증거가 나타내는 document는 소유 main-document 수명이다. iframe의 정확한 initiator/frame generation을 입증하지 못하면 해당 frame 출처라고 표기하지 않는다. service/shared worker에서 독립 수명의 응답을 현재 문서에 귀속하지 않는다. 현재 문서에 속함이 검증되지 않은 응답은 성공으로 채택하지 않는다. origin-only/missing Referer를 완전한 실제 Referer로 발명하지 않고, fallback을 사용할 경우 provenance에서 명시한다.

원 navigation intent는 실제 browser 관측이며 HTTP redirect chain을 뜻하지 않는다. DOM/status=null과 manifest-prefix4096bytes/부분본문을 구별하고 원 capture를 바꾸지 않는다. 요청 식별용 외부 header 주입·인증서 우회·웹 payload의 셸/프로세스 실행은 사용하지 않는다. cancel/deadline/publication CAS는 기존 jobs 계약을 재사용하며 새 상태 머신을 추가하지 않는다. 새 WebView 생성의 COM 영구 대기를 interrupt 가능하다고 주장하지 않는다.

## 등록 전 인수

1. 순수 Rust instance identity 회귀: 이전 ticket/문서·같은 URL 재탐색·instance 교체 뒤 늦은 응답/async 완료/오류가 새 문서 상태를 바꾸지 않는다.
2. lifecycle 회귀: cancel 접수 후 old response 무효·worker Cancelled ack 구분, capture-first terminal 보존, EOF 후 callback 무효, 재획득 뒤 세대 증가.
3. 한도 회귀: stage≤32, queue≤16, pending prefix≤4, prefix≤4096bytes, DOM≤2MiB, 누계 counter overflow·깨진 identity·관측기 설치 실패는 명시 오류.
4. 실제 회차 DOM→Rust next-stage 추출→frame.example/player 진입과 유효 응답을 고정 CLI로 보존한다. source URL·ticket/doc generation·원문/부분본문 SHA·engine/filter/출처를 재조회하고 fresh CLI에서 검증한다.
5. 명시 native host의 media open·제어·회차 전환과 실제 디코드/시청은 별도 실제 인수다. media URL 관측을 재생 성공으로 세지 않는다.

이 선택은 아직 등록된 media 지원이나 실제 성공의 증거가 아니다. 기존 미등록 observer는 이 수명 설계에 맞춰 통합하고 검증하기 전 등록하지 않는다.

같은 profile을 사용해도 문서별 재생성이 cookie·redirect·POST·iframe·worker 동작을 보존한다는 증거는 아직 없다. 실제 episodes.list DOM 취득을 먼저 수행하고 episode/player 구조에서 재생성이 필요한지 및 가능한지 확인한다. POST 본문을 GET으로 바꾸거나 URL만으로 요청을 재생성해 호환성을 주장하지 않는다. 실제 관측에서 호환성 문제가 드러나면 위 대안 비교와 선택을 갱신하며 이 ADR을 지원 완료로 해석하지 않는다.
