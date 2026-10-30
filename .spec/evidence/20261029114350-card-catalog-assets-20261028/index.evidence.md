---
id: evidence:kinekino/20261029114350
title: 카드 카탈로그 공통 자산 중복 검증 개선 — R127
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 카드 카탈로그 공통 자산 중복 검증 개선 — R127

최종 gate:2178 정상2bins build 종료0(캐시 재사용,585ms),2179 사후 custody 종료0으로2373파일 content/identity 동일. fixed-r127/native-bins exe/PDB 보존,GUI SHA618A0E95E8F500A8B0E8C89F641E8935A3037A6E798ADCA995613A64A85F5E49,CLI SHA8AB93DE752CFD22039BC48292A6DE1C5B23B2DDC24C8AEEF87FDC5319FD1448E. 실제19CLI/원EOF·양strict/fmt·지정190·정상bins·입력동일까지 완료했으며 R126첫실패2회는 역사증거로 보존한다. 다음은 실제BatchLease orchestration/native event store/SDK·CLI/부분실패 인수다. 아래 미완료 표기는 각 실행 당시 상태다.

기본 gate 진행:2171 기본 lib/Jobs/CLI no-run,2172 기본 strict,2173 fmt 종료0.2174 Product145/2175 Core18/2176 Native26/2177 OS1 종료0으로190/190 통과했다. 네 group-return execution_result/producer_integrity Ok 및 동일 R127 SOURCE.md SHA를 확인했다. Meari는 기존622/624 manifest로 실제시험 재실행이며 새 컴파일이 아니다.2178 정상2bins build 진행 중이며 사후 custody는 남는다.

추가 확인:2170 카드 strict 종료0(외부 blitz-paint 기존경고1). 원host receipt completed_total=12,cancelled/interrupted=0,dispatch completed/pipe joined/sdk joined 확인. fixed-r127/card lib SHA130CD182A72A661B6F62B2AC5EA16F7600E87038381931A4282E491A8F425E76,Jobs SHACFB7D98EFA59612947660921F7C30CB4F3CAB787629BD6B5335B942C0C391C83.2171 기본구성 no-run 진행 중이다. 아래 진행상태는 당시 기록이다.

R126의2155/2156 실제CLI는 첫 capabilities 응답의 기본5초 client_deadline/read_response timeout으로 실패했다. 원 로그와 고정exe는 보존했다. 카탈로그가13개 템플릿마다 동일한 공통 스타일과 폰트 자산을 다시 준비·검증하는 것을 확인해 조회당 한 번 검증한 메타데이터를 공유하도록 수정했다. 전역 캐시·검증 생략·제한시간 증가는 없으며 템플릿 선언/핀/13종 카탈로그/반환 계약을 유지한다. 개별 검증도 공통 helper로 핀을 생성한다.

원본은 preimage-r126-catalog-assets/bundle.rs에 보존했다.2166fmt/2167snapshot 종료0. input-r127은2373파일,inventory F8952C69A6A826DFC1869CBE1ADE099FEBADE19B4F43B710765B17A2B5DDBE71,SOURCE.md B60D35EBB14FF1AE6157321FBBAD3DE7A9980D065C4E8517A6A3983496256C95다.2168 카드lib/Jobs/CLI no-run 종료0,exe/PDB를 fixed-r127/card에 보존했다. CLI SHA EF261873DE730288900F484EF2B51597112F9B3B1A0233AFB48E10A010E327C6.

2169 같은 제한시간의 실제CLI 전체시험이48.37초에 종료0이다.19요청/완료12작업/6family×3role의18새PNG 및 원 host 종료를 검사했다. capabilities 요청은472ms/exit0이며 이전 첫 timeout을 재현하지 않았다. 이번 변경과 회복의 연관을 실제경로로 확인했으나 일반적인 최대지연이나 모든호스트 성능을 보장하지 않는다.

2170 카드 strict 진행 중. R127 기본전체190/정상bins/사후custody는 아직 미완료다. 최신 완료 기본gate는 R126이며 전체 일괄저장/orchestration/CLI·GUI·AI사용자승인0/9·기존소유권/WIP/보류를 유지한다. source375 이후 변경은 private overlay다.

기존 modootoday/software-qa/code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 첫 실패와 수정입력·동일 제한의 실제회복·미검증 범위를 구분했다.
