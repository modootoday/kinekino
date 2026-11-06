---
id: adr:kinekino/20261106022549
title: Linux Secret Service CredentialStore adapter
status: active
created: 2026-11-06T02:25:49Z
updated: 2026-11-06T02:25:49Z
rel:
  governs: [crates/kinekino-platform-unix/src/secret_service.rs, crates/kinekino-platform-unix/Cargo.toml, crates/kinekino-platform/src/lib.rs, Cargo.toml]
  depends_on: [adr:kinekino/20261106001220]
---

# Linux Secret Service CredentialStore adapter

## Context

로드맵 M3-3(ST-2)다. M3-2가 `check_credential_store`와 Session/File 어댑터 패리티를 고정했다. M3 완료 기준 (2)는 Linux Secret Service 어댑터와, 실패 시 메모리 저장소로의 **명시적** 대체를 요구한다. 이번 조각은 OS 어댑터와 계약 통과만 다루며, 제품 조립에서 Session 폴백을 배선하지 않는다.

권고안(루틴 진행):
1. 잘 알려진 `keyring` 크레이트는 기본(login) 컬렉션이 잠겨 있으면 프롬프트 대기로 멈출 수 있어, 같은 zbus 계열의 `secret-service` blocking API로 org.freedesktop.Secret.Service에 붙는다.
2. 이미 잠금 해제된 컬렉션만 고른다(잠긴 login에 `unlock` 프롬프트를 걸지 않음). 세션 컬렉션이 열려 있으면 그것을 쓴다.
3. `try_open`/`open`은 D-Bus·Secret Service 부재 또는 사용 가능한 컬렉션이 없으면 재시도 가능 `Unavailable`을 반환한다.
4. `(service, account)`는 `require_credential_key` 후 Secret Service 속성으로 매핑하고, 등록 테스트에서 `check_credential_store`를 산다.
5. Windows Credential Manager는 이번 범위 밖(자리표시자 유지).

skill: 조회 결과 무관(자격증명 포트·Secret Service와 직접 맞는 스킬 없음).

## Decision

1. **모듈.** `kinekino-platform-unix`에 `secret_service::SecretServiceCredentialStore`를 둔다. `secret-service`는 workspace.dependencies에 두고(features `rt-async-io-crypto-rust`), Linux target에서만 끌어온다. 비-Linux Unix는 `try_open`이 `Unavailable`이다.
2. **연결.** `secret_service::blocking::SecretService::connect(EncryptionType::Dh)`로 세션 버스에 붙는다. 호출마다 짧게 연결·해제해도 된다(수명·프롬프트 회피).
3. **컬렉션.** 모든 컬렉션을 나열해 **이미 unlocked**인 첫 컬렉션을 고른다. 없으면 `Unavailable`. 잠긴 기본 컬렉션에 대화형 unlock을 요청하지 않는다.
4. **키 매핑.** 속성 `service` / `account`, 라벨 `{service}/{account}`. `set`은 `replace=true`로 덮어쓴다.
5. **계약.** 라이브 테스트가 `check_credential_store`를 호출한다. 서비스가 없으면 해당 테스트를 건너뛰고, `try_open`의 Unavailable은 별도 테스트로 단언한다.
6. **파사드.** `kinekino-platform::credential`에서 Unix 빌드에 `SecretServiceCredentialStore`를 재수출한다. 앱 설정은 계속 `SessionCredentialStore`를 쓴다.
7. **제품 폴백(설계만).** Secret Service `try_open` 실패 시 제품이 `SessionCredentialStore`(또는 동등한 메모리 저장소)로 **명시적으로** 대체할 수 있다. 이 ADR은 그 배선을 구현하지 않는다(M3 완료 기준 (2)의 조립은 후속).

## Rationale

- keyring 기본 경로는 잠긴 login에서 헤드리스 환경이 멈출 수 있어, unlocked 컬렉션만 고르는 thin `secret-service` 경로가 더 작과 안전하다.
- 동일 `check_credential_store`를 통과해야 Session/File과 의미가 어긋나지 않는다.
- 제품 폴백을 어댑터 안에 숨기지 않으면 호출부가 OS 실패를 관측할 수 있다.

## Consequences

- Linux에서 Secret Service가 열려 있으면 ST-2 OS 어댑터 계약이 통과한다. Windows Credential Manager는 여전히 자리표시자라 ST-2는 **부분**으로 남는다.
- 앱은 이 조각 후에도 Session 저장소를 쓴다. M3 기준 (2)의 “명시적 대체” 배선은 후속 조각이다.
- macOS 등 Secret Service가 없는 Unix는 `Unavailable`이다.

## Record

