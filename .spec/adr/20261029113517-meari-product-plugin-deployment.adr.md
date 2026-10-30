---
id: adr:kinekino/20261029113517
title: Meari 제품 플러그인의 초기 배포 방식
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Meari 제품 플러그인의 초기 배포 방식

2026-10-19. 상태: 제안. [제품 계약](../../sots/player/presentation/shared/meari-product-plugins.md). Meari SDK 담당과 합의·실제 검사 후 채택 상태로 갱신한다.

## 문제와 선택

Native와 React/TSX→WASM Web UI에서 파일·창·재생·공통 SDK 기능을 같은 typed capability로 소비해야 한다. 모든 제품 기능을 Meari 본체에 넣을 수 없고 임의 Rust DLL ABI/별도 JS 정책 실행기를 도입할 이유도 아직 없다.

초기 제품은 **빌드 시 연결한 Rust 플러그인과 Meari 공통 static registry**를 제안한다. Native는 host가 주입한 OS/SDK adapter, Web은 같은 Rust capability binding을 WASM으로 빌드하고 browser API adapter를 사용한다. 브라우저에서 제공할 수 없는 native 기능은 명시 Unsupported다. SDK IPC가 필요하면 기존 공통 API의 typed transport로 제한하고 임의 shell/subprocess 실행을 capability에 노출하지 않는다.

| 방식 | 장점 | 비용·제약 | 판단 |
| --- | --- | --- | --- |
| static Rust registry | 동일 타입 검사·명시 의존성·기존 Cargo/target 검증 활용 | native 기능 변경 시 재빌드, process sandbox는 아님 | 초기 제안 |
| 동적 Rust DLL | 독립 파일 교체 가능 | Rust ABI 안정 보장 없음, 수명/allocator/호환/배포 계약 추가 필요 | 초기 도입 제외 |
| 독립 WASM component/WIT plugin | 경계와 제한된 capability를 명시 가능 | 별도 host/runtime/WIT·browser bridge 검증 필요, 현재 Meari 소비 API와 연결 미검증 | 독립 배포·격리 필요가 입증되면 재평가 |
| 별도 process IPC | SDK/worker 격리 가능 | transport 인증·버전·취소·backpressure·프로세스 수명 비용 | 기존 SDK transport 소비에 한정, 신규 범용 plugin daemon은 보류 |

Rust Reference는 Rust ABI에 안정성 보장을 제공하지 않는다고 명시한다. 이 사실과 현재 typed Rust 경계를 근거로 동적 DLL을 기본값으로 선택하지 않았다. [공식 ABI 문서](https://doc.rust-lang.org/reference/items/external-blocks.html#abi).

`wasm-bindgen`의 browser 대상은 `wasm32-unknown-unknown`이며 `web-sys`는 browser WebIDL API binding을 제공한다. 따라서 Web adapter는 Rust/WASM과 명시 browser API 경계로 설계할 수 있다. 이것이 native OS 기능과 동등한 권한/동작을 보장한다는 뜻은 아니다. [공식 target 문서](https://wasm-bindgen.github.io/wasm-bindgen/reference/rust-targets.html), [공식 web-sys API](https://wasm-bindgen.github.io/wasm-bindgen/api/web_sys/).

## 결과와 재검토 조건

plugin 코드 교체를 TSX HMR과 분리한다. schema/manifest/permission 변경은 재협상하고 native binary 변경은 정상 재시작한다. SDK 작업의 idempotency와 수명을 presentation reload가 소유하지 않는다. static registry는 보안 sandbox가 아니므로 비신뢰 외부 plugin 설치 지원을 주장하지 않는다.

제3자 독립 배포, 장애 격리, 서로 다른 compiler/SDK 수명, 실제 browser 지원 요구가 생기면 WASM component 또는 versioned IPC 방식을 별도 spike로 비교한다. 현재 단계에서 새 장시간 시험·VM·DPI·추론 실행을 추가하지 않는다. Meari 본체 확장 SDK 인계와 실제 CLI 계약 검증이 다음 gate다.
