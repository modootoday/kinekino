---
id: plan:kinekino/20261029113918
title: 제품 SDK parent 등록 읽기 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 SDK parent 등록 읽기 검토

2026-10-20. Meari 담당이 공개 registry 계약과 제품 등록 소스를 읽었다. 실행 재현·수리 완료와 구별하며 구현은 SDK 소유다. [등록 계획](sdk-product-host-registration.md)을 보완한다.

## 공개 요청 문맥 검증 후보

scoped_context는 원 request_id와 ticket으로 상관 ID를 만든다. 직접 등록·registry dispatch 경로에서는 빈 ID/제어문자 ID가 정상 ui-hash로 바뀌어 공통 Request 검증이 원래 값을 보지 못할 가능성이 있다. 기존 GUI RegistryProposalHost와 도구 ProposalBridge는 선행 검증하므로 그 경로 전체가 실패한다고 단정하지 않는다.

격리 선택20 통과 뒤 직접 SDK 호출과 제품 lease 호출의 invalid-ID 동등성 시험을 실행했다. context-red-first.receipt.json은45553ms/101이며 1시험이 실제 실패했다. 첫 invalid ID에서 직접 SDK는 InvalidArgument, 제품 등록 dispatch는 Unsupported를 반환했다. 이는 상관 ID 변환 전에 원 문맥 검증이 누락된다는 관측 근거이며, 실제 저장/채택 성공이나 전체 invalid 입력의 결과까지 뜻하지 않는다. PM은 receipt와 원 stdout을 직접 대조했다. 공통 SDK 검증을 재사용해 상관 ID 변환 전에 원 문맥을 확인하며 Meari나 제품 adapter에 ID 규칙을 복제하지 않는다. 유효 문맥의 옵션 보존·동일 ticket 안정성도 회귀로 확인한다. 수리 후 통과는 아직 미검증이다.

PM은 SDK의 최소안을 승인했다. 기존 Request.validate의 문맥 조건을 additive RequestContext::validate로 분리하고 Request는 위임하되 payload 검사는 유지한다. assembly helper는 hash 전에 공통 메서드를 호출한다. commands.rs/request_context.rs/lease 회귀 범위이며 오류 문구·안전한 요청 ID 처리·wire를 유지한다. 현재 격리 실행 종료 뒤 red 재현→최소 수리→새 입력 회귀로 인수한다.

수리 후 context-green-first는43857ms/0이며 root 집중6pass/0fail/0ignored·282filtered였다. PM은 실제 시험명을 대조하여 invalid 원 ID 동등성·동일 ticket 안정성/옵션 보존 및 기존 scope/cleanup 회귀를 확인했다. 소스에서도 Request가 공통 validate에 위임하고 scoped_context가 hash 전에 호출함을 확인했다. 이 근거는 집중 회귀이며 root 전체·수리 후 consumer 전체·strict/fmt 최종 인수는 남아 있다.

후속 동일 수정본의 root full313867ms/0·385pass/19ignored, root strict55270ms/0·fmt1294ms/0, 격리 consumer full207462ms/0·47pass/0ignored·strict382830ms/0·fmt309ms/0을 PM이 원 receipt와 결과행으로 확인했다. 앞 문단의 전체 검사 잔여는 이 결과로 해소됐으며, 최종 계약/pin 대조·소유 자원 정리와 명시 반환은 담당 인계 뒤 인수한다. 실제 창·fresh CLI/provenance·품질 검토 잔여는 그대로다.

## fingerprint의 범위

현 request/reply pin은 각각 proposals 모듈 파일 하나이며 별도 document_source/stored DTO 파일까지 포함한 canonical schema 해시가 아니다. provider pin도 adapter 파일 하나로 문맥 helper·typed plugin 전체를 포괄하지 않는다. 역할별 prefix와 Native capability 분리는 확인됐으나 이를 전체 provider attestation으로 안내하지 않는다. 현재 범위와 한계를 정본에 명시하고 생산 계약을 고정할 때 canonical schema 또는 책임별 source inventory를 별도 인수한다.

## 소유와 종료의 확인 범위

실제 Rc registry/context/scope와 주입된 SDK를 사용하며 도구는 제품 구현을 재수출한다. unregister 성공 뒤에만 handle을 지우고 실패 시 owner를 보존한다. 현재 고정 typed adapter는 Immediate이며 실제 provider Pending을 증명한 것은 아니다. Drop은 best effort다.

consumer의 held-registry close 실패는 UI session 정리에서 먼저 발생할 수 있으므로 제품 parent close 도달의 단독 증거가 아니다. 제품 lease 자체 borrow 실패는 해당 단위 회귀와 구분한다. serialized handle에서 생략되는 owner identity 때문에 상관 ID를 검증 없이 전역 유일하다고 표현하지 않는다. 실제 창·OS wake·fresh CLI·provenance 저장 결합의 잔여는 유지한다.
