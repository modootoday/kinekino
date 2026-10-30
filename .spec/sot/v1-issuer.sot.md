---
id: sot:kinekino/v1-issuer
title: v1 식별자와 원 issuer 결속
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# v1 식별자와 원 issuer 결속

상태·인수 한계는 [C0 index](index.md)를 따른다.

| 값 | 유일한 정규 형식 | ASCII bytes |
|---|---|---:|
| issuer scope | `nsi1.` + 원 scope nonce의 소문자 hex32 | 37 |
| result reference | `nsr1.` + 같은 nonce hex32 + `.` + 원 row id hex16 | 54 |

nonce는 16bytes/nonzero, row id는 기존 checked nonzero u64다. v1 prefix·폭·소문자·구분자·ASCII를 정확히 검사한다. 공백/Unicode/대문자/가변 폭/0 nonce·0 row/다른 version/추가 부분은 InvalidArgument다. `parse→encode`는 입력과 byte 동일해야 한다. row id는 기존 grant id가 command row로 전환된 원 번호이며 새 counter/독자 ID를 만들지 않는다. Debug.id·receipt/request_id·표시 제품명으로 codec을 구현하지 않는다.

scope nonce는 infrastructure scope가 OS instance GUID를 한 번 얻어 보존한다. 현 코드의 CoCreateGuid 어댑터 패턴을 사용하되 skip registry generation 값을 재사용하지 않는다. scope 내부 OnceLock에 nonce 또는 고정된 실패를 캐시하고 첫 공식 issuer 획득 때 준비한다. 실패/zero는 bounded EngineFailed로 admission/Observe 전에 거절하고 암묵 재시도하지 않는다. 기존 infallible HostScope 생성자와 non-wire A/B 동작은 바꾸지 않는다. GUID를 암호 인증 토큰이라고 주장하지 않는다.

공개 parser는 bounded lookup 값만 만든다. 원 issuer 권한은 신뢰된 조립이 캡처한 `NativeExportIssuer` opaque Rust handle이 보존한다. 이 handle은 원 Weak scope/read port와 고정 scope id만 가지며 serde/public Weak mint를 제공하지 않는다. endpoint의 고정 handle에 payload scope/ref의 nonce를 비교한 뒤 원 Registry에서 row를 찾는다. locator를 추측/파싱하는 것으로 guard나 제출 권한이 생기지 않는다.

## router 원 issuer query

Native backend의 issuer 획득은 Active 없이 원 scope를 캡처한다. Deferred는 고정 binding/epoch의 원 scope를 캡처한다. router는 `current_route()`를 골라 문자열 scope를 해석하지 않는다. 신뢰된 조립이 이미 소유한 **명시 ParentRouteHandle**을 주어 belongs/epoch를 확인하고 원 issuer handle을 캡처한다. 반환 뒤에는 원 Weak scope로 list/read하며 현재 route를 재선택하지 않는다.

CLI host는 기존 동일 Sessions와 함께 이 고정 issuer handle 하나를 주입한다. 원 scope와 다른 새 scope를 같은 고정 handle의 현재값으로 바꾸지 않는다. 새 issuer 노출은 명시 조립/새 host lifetime의 책임이다. 원 issuer를 지정하지 않은 router 전체 list는 B의 Unsupported를 유지한다. retired scope directory·외부 map을 추가하지 않는다. 식별된 Weak owner 만료는 SourceExpired이고, 죽은 endpoint의 transport 실패를 API SourceExpired 응답으로 꾸미지 않는다.

새 제출은 typed `observe_native_guard_for_export(expected_issuer)`에서 원 admission의 scope가 기대 handle과 일치하는지 **Observe 전에** 확인한다. foreign/stale scope를 확인하려고 native 관측을 먼저 실행하지 않는다. 기존 observe/control로 fallback0이다. parser 값과 opaque issuer handle은 별도 타입이며 nonce equality만으로 native authority를 발급하지 않는다.
