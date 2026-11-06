---
id: adr:kinekino/20261106001220
title: CredentialStore contract for empty keys and adapter parity
status: active
created: 2026-11-06T00:12:20Z
updated: 2026-11-06T00:13:00Z
rel:
  governs: [crates/kinekino-core/src/port/credential.rs, crates/kinekino-core/src/port/contract/mod.rs]
---

# CredentialStore contract for empty keys and adapter parity

## Context

로드맵 M3-2(ST-2)다. `CredentialStore` 포트(`get`/`set`/`delete`)와 `Secret`(Debug는 값을 찍지 않음)은 이미 있고, Unix `SessionCredentialStore`·`FileCredentialStore`는 기본 왕복만 `check_credential_store`로 검증한다. Windows 세션 저장소는 `Unavailable` 자리표시자다. M3 완료 기준 (2)는 비밀이 OS 자격증명 포트를 거치도록 요구하며, Linux Secret Service 어댑터는 M3-3이다. M3-2는 **계약 하네스만** 키우고 OS keyring·D-Bus 의존은 넣지 않는다.

권고안(루틴 진행):
1. `check_credential_store`를 확장해 키 격리·덮어쓰기·부재 delete·빈 service/account → `InvalidInput`(재시도 불가)·왕복을 같은 규칙으로 강제한다.
2. `Secret` Debug 적색화는 core 계약 테스트로 단언한다.
3. Session(Unix)·File 어댑터가 빈 키를 거부해 하네스를 통과한다. Windows 자리표시자는 그대로 둔다.
4. M3-3 Secret Service 어댑터도 같은 `check_credential_store`를 호출한다. Secret Service 실패 시 제품이 메모리 저장소로 명시적으로 대체할 수 있다는 설계 메모만 남기고, 이번 조각에서 대체 배선은 하지 않는다.

skill: 조회 결과 무관(자격증명 포트 계약과 직접 맞는 스킬 없음).

## Decision

1. **`check_credential_store` 계약.** 빈 저장소에서 (a) 기본 왕복 get/set/delete, (b) 서로 다른 `(service, account)` 쌍은 충돌하지 않음, (c) 같은 키에 두 번째 `set`은 값을 교체, (d) 없는 키 `delete`는 `Ok(false)`이고 다른 키는 유지, (e) 빈 `service` 또는 빈 `account`의 get/set/delete는 `ErrorCode::InvalidInput`이며 재시도 불가.
2. **`require_credential_key`.** core `credential` 모듈이 어댑터가 재사용할 빈 키 검사를 제공한다.
3. **`Secret` Debug.** `format!("{:?}", Secret::new(…))`에 평문이 없어야 하며 core 계약 테스트로 고정한다.
4. **어댑터 범위.** Unix Session·File만 빈 키 거부를 추가한다. Windows Session은 `Unavailable` 자리표시자라 계약 하네스를 호출하지 않는다.
5. **M3-3.** Secret Service(및 이후 Credential Manager) 어댑터는 등록 테스트에서 같은 `check_credential_store`를 통과해야 한다. OS 저장소 실패 시 메모리 폴백은 제품 조립에서 명시적으로 드러내며, 이 ADR은 폴백 배선을 구현하지 않는다.

## Rationale

- 모든 어댑터에 같은 규칙을 걸면 M3-3에서 OS 어댑터가 조용히 다른 의미를 갖지 못한다.
- 빈 키를 `InvalidInput`으로 막는 것은 키링/파일 경로에서 모호한 항목을 만들지 않기 위함이다.
- Debug 적색화는 로그·패닉에 비밀이 새지 않게 하는 최소 계약이다.
- OS 어댑터와 폴백 배선은 ST-2 “구현”에 해당하므로 M3-3으로 미룬다. 이 조각 후 ST-2는 **부분**.

## Consequences

- Session·File·(향후) Secret Service가 동일 하네스를 통과해야 한다.
- 빈 service/account 호출은 성공하지 않는다.
- ST-2는 계약·메모리/파일 통과까지 **부분**; OS 어댑터는 M3-3.
- M3 완료 기준 (2)는 M3-3 이후에야 충족된다.

## Record

- 2026-11-06: M3-2 — CredentialStore contract harness expanded; Session/File enforce empty-key InvalidInput (pending merge hash).
