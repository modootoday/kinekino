---
id: adr:kinekino/20261029113525
title: "ADR: static VAD release와 전용 worker의 결속"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR: static VAD release와 전용 worker의 결속

2026-10-21 Media 소유 계약 선택. 제품 활성화·배포 승인은 미완료다. 중앙 지시로 architecture 스킬의 trade-off/대안/전환 기준과 modoo code-change-review-verification의 기존 guard 주장 대조를 적용했다.

## 배경과 관측

ArtifactPin의 PartialEq는 id/SHA/bytes 전체를 비교한다. VadProfile.runtime은 최대64MiB이며 그 값 자체는 admission credential이 아니다. 현재 NativeTransportFactory는 trusted ExecutableGuard로 연 파일의 실제 bytes/SHA를 spawn 전에 검사하고 binary_guard를 원 transport에 보유한다. SDK attempt identity와 process identity/cleanup 소유권은 이미 별도다. 새로운 identity나 registry를 만들지 않는다.

## 선택

static ORT 제품 release의 files.runtime은 ORT native code가 링크된 전용 worker executable을 뜻한다. build-only onnxruntime.lib는 포함하지 않는다. 승인된 complete VadProfile의 runtime과 configuration.worker_pin은 ArtifactPin 전체가 같아야 한다. 승인 profile과 입력 descriptor의 complete profile도 같아야 한다. 이 비교는 trusted ProfileAdmission/기존 release lookup 뒤, attempt resource 생성 및 spawn 전에 수행한다. caller JSON이나 worker 자기보고가 승인 주체가 되지 않는다.

id는 logical artifact identity이고 PID/creation time은 process identity다. 같은 id만으로 승인하지 않으며 content만 같고 id가 다르면 현재 계약에서는 거절한다. 실제 파일은 기존 ExecutableGuard의 원 파일/ancestor guard와 실제 SHA/length 검사로 이 pin에 결속한다. 실행 family의 원 identity/ACK/EOF/cleanup은 기존 NativeTransport가 계속 소유한다. 순수 pin 비교는 실제 executable guard나 cleanup을 증명하지 않는다.

DirectML/필요 native DLL과 notices는 runtime_dependencies/runtime_notices로 분리한다. native ABI·최소 OS·build provenance·권리/고지 인수는 별도 선행 조건이며 위 비교로 대신하지 않는다. 같은 worker를 공유하는 두 release라도 complete profile 승인은 요청의 정확 release에 대해 수행해야 한다. 새 승인 저장소를 도입하지 않는다.

## 대안과 비용

별도 engine pin + worker pin release는 dynamic engine 교체에 유연하지만 두 payload의 호환성과 공동 승인을 추가로 표현해야 한다. 현재 static native code는 executable에 포함되므로 중복 역할을 만들지 않는다. content만 비교하면 alias가 섞일 수 있으므로 기존 ArtifactPin 전체 비교를 선택한다. 이는 byte-identical alias도 재승인이 필요하다는 비용을 수용한다.

## 호환·전환

현재 제품 operation은 비활성이므로 기존 실제 사용자 실행을 성공으로 승계하지 않는다. schema parser의 runtime 의미를 모든 과거 fixture/실험에 소급해 바꾸지 않는다. 다음 제품 조립 revision에서 static worker 역할을 명시한 trusted release를 만들고 exact worker pin을 동일 경로로 공급한다. 독립 synthetic fixture의 임의 runtime/worker 불일치는 해당 revision에서 명시 수정하고 원 실패/구판은 보존한다. union18/43에는 이 변경을 넣지 않는다. dynamic engine 배포를 선택할 때는 schema/호환 계약을 새 ADR로 재검토한다.

## 인수 조건

정상 complete profile/runtime/worker 일치, 같은 크기 다른 digest, 다른 승인 release와 요청 profile의 교차 조합, 동일 content의 다른 artifact id를 순수 fixture로 확인한다. 실패3은 resource 준비/spawn 전에 끝나야 한다. 별도 actual guard 시험에서 파일변경·reparse·원 handle 유지와 기존 process identity/cleanup을 검증한다. source fixture 작성은 통과가 아니며 현재 Cargo·native load0이다.
