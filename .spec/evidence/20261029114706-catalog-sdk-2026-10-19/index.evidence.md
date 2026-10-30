---
id: evidence:kinekino/20261029114706
title: 공통 카탈로그 SDK 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공통 카탈로그 SDK 검증

## 범위와 상태

명시 Windows --catalog-root의 experimental catalog.search(category-only)/episodes.list, 원문 artifact 보존과 공통 typed SDK 연결 범위다. 실제 사이트 성공·검색/genres/sort·media resolve·GUI·AI 기능 완료가 아니다. 검증 명령 27/60 집계는 유지한다. [계약](../../player/application/api/catalog.md), [공개 사이트 수집 관측](catalog-http-2026-10-19.md).

## 최초 root 통합 checkpoint

runtime/quality-review/catalog-sdk-r1/logs/check.log: workspace all-target offline check 종료0(61초). 신규 HTTP/HTML 의존 52개를 캐시에서 lock에 추가했고 기존 registry 버전 Updating 로그는 없다. 네트워크 다운로드는 없다. source 공개는 media-catalog manifest/src Rust만 최소 allowlist로 추가했다. 독립 workspace 선언을 제거하고 root의 여섯 번째 context로 등록했다.

fmt0, logs/test.log의 workspace test 종료0(컴파일4분23초), **196 passed/9 ignored**. 기존185에 provider6+application3+파일저장1+CLI1을 추가한 checkpoint다. synthetic complete HTTP error가 먼저 저장된 후 parse 실패에 같은 참조가 남고, 전송 실패/identity 불일치가 원문을 생성하지 않으며, 실제 일반 파일의 SHA·재오픈·손상 거부와 명시 CLI의 discovery/미지원 query 거부를 검증했다. 이 source에는 이후 상태/schema·전송진단 보완이 아직 포함되지 않았다.

## 최종 보완 검사

기본/명시 backend status의 unsupported/experimental 대응, maturity 분리, typed 응답 schema, 안전한 bounded source/get_ref 전송진단 회귀를 같은 source checkpoint로 검사했다. final-fmt0, workspace all-target strictClippy0(19.29초), final-test **201 passed/9 ignored**(컴파일2분40초), 두 bin build0(0.79초). source-final과 Cargo.lock SHA 0cf44760431b804cdf18afd50aa82f57b50579cfe1465f9f6d6e9ee7b1aa96cc를 보존했다. 이후 browser provider 소스는 이 checkpoint 검증에 포함하지 않는다.

고정 CLI SHA **23bfd6c5e1b6ff66240e0dc7e8579660ad9dc86256cc3680fa1f435badb6b64a**, GUI SHA **20d3fbc63c5327ff28b7f3f2bc7d3d213657ed5e24f1f2cf52c50b9fc82bfe3a**. GUI는 실행하지 않았다. 실제 CLI 기본/명시 capabilities와 명시 schema는 exit0, 미지원 query는 exit2다. backend=false status=unsupported/maturity=experimental, backend=true status=experimental/site_verified=false를 CLI 회귀와 raw metadata로 대조했다.

최종 진단의 단일 실제 방영중 요청은 **exit8/engine_failed**다. phase=request_send, connect=true/timeout=false, IO payload=ConnectionReset/OS10054를 보존했다. HTTP 응답·완전 body·status·SHA가 없으며 raw_artifact/capture는 null, store의 objects/captures도 비어 있다. 연결 리셋 관측이며 봇 탐지/TLS/DNS/proxy 중 어느 원인도 확정하지 않는다. actual-cli/requests·stdout/stderr·exit·binary-sha256.txt를 보존했다. 추가 반복은 없다.

실제 사이트 목록/회차 성공과 browser provider는 미검증이다. HTTP client 초기화 실패는 root 조립에서 일반 engine_failed를 반환하며 요청 중 상세 전송 graph와 구분한다. 초기화 graph의 root 전달은 후속 공통 오류 조립 정리 대상이다.
