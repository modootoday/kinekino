---
id: evidence:kinekino/20261029114634
title: 공개 카탈로그 HTTP 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공개 카탈로그 HTTP 관측

## 검사 범위

독립 packages/media-catalog의 fmt·test·all-target strict Clippy를 실행했다. catalog-r1은 synthetic5회귀, 원문 분리 후 catalog-r2와 진단 보완 catalog-r3/r4는6회귀가 통과했다. 로그는 runtime/quality-review/catalog-r{1,2,3,4}/logs/에 보존한다. 실제 관측 종료1은 아래 전송 실패이며 테스트 실패가 아니다.

URL allowlist/query, synthetic 목록/회차 역순, challenge/알 수 없는 화면/명시 빈목록 구분, 외부 행·잘못된 pagination, HTTP오류/challenge/parser실패 후 exact raw 보존을 검증했다. 실제 사이트 DOM·redirect 서버·취소·네트워크 timeout까지 검증한 것은 아니다.

## 실제 읽기 요청

Rust example observe는 제품 CLI에 등록되지 않았다. r2는 세 카테고리 각각 한 요청, 진단 보완 r3도 세 요청, r4는 반복을 줄여 방영중 한 요청만 수행했다. observation JSON과 stdout/stderr를 checkpoint별 보존했다.

| 요청 | 결과 | HTTP status·완전 원문·SHA |
|---|---|---|
| https://source.example.test/방영중 | transport, connection failed | 수신 자료 없음 |
| https://source.example.test/종영 | transport, connection failed | 수신 자료 없음 |
| https://source.example.test/극장판 | transport, connection failed | 수신 자료 없음 |

r4 구조화 진단은 connect=true/timeout=false/io_kind=Other/os_code=null이다. TLS/DNS/프록시 원인은 미확정이다. 인증서 검증 해제·신뢰 설정·인증서·프록시 환경 변경은 없었다. 환경변수 전체·오류 원문·프록시 자격증명을 출력하지 않았다.

raw_artifact=null/provenance=null은 완전 HTTP body를 받지 못했다는 뜻이다. 빈 카탈로그·challenge 응답·성공 status·원문 SHA를 생성하지 않았다. 브라우저·GUI·미디어·AI 실행도 없다.

## 인계와 미검증

CatalogFetchPort::fetch는 수신 완료한 bounded RawDocument(status/content type/encoding/exact bytes/provenance)를 반환한다. SDK는 별도 artifact로 보존한 뒤 parse_document(&raw)를 호출할 수 있다. 기본 RPC raw inline을 요구하지 않는다. CatalogReadPort는 fetch/parse 편의 조합이다.

실제 category selector·빈목록 marker·작품/회차·pagination은 미검증이다. selector 가설과 synthetic 회귀를 사이트 지원으로 승격하지 않는다. 전송 원인 확인 후 실제 read-only 수직 검증이 필요하다. 검색/genres[]/sort·frame.example resolve·미디어 재생·제품 CLI 등록은 현재 미지원이다.
