---
id: evidence:kinekino/20261029114333
title: 제품 Native 등록 1단계 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 Native 등록 1단계 검증

2026-10-20 승인된 다섯 파일 범위에서 root의 optional Windows 기능을 등록했다. `Cargo.toml`/`Cargo.lock`, `src/lib.rs`, `src/infrastructure/mod.rs`, `src/assembly/meari/mod.rs`만 제품 등록 대상으로 변경했다. [조립 계획](../../../plans/player/sdk-proposal-window-startup.md)의 1단계이며, UI façade·launch DTO·main 연결은 후속 단계다.

## 변경과 경계

- `native-proposal-window` → optional `meari-native`, `default-features=false`, `window` 기능.
- 공통 `blitz-paint` 로컬 patch 및 Windows/기능 조건부 self alias.
- 기존 notifier와 startup 모듈의 조건부 export. 기존 SDK application/domain 정책과 Native/Core/UI 구현은 변경하지 않았다.
- 기본 GUI backend와 CLI 명령 계약은 유지했다. 실제 Native 창 실행·시각 검토·제품 전체 완성은 이 검증의 결론이 아니다.

## 관측 결과

원 로그·receipt·이전/최종 사본은 [검증 디렉터리](../../../../runtime/quality-review/product-native-registration-step1-r1)에 있다. 모든 Cargo 작업은 오프라인이며, heavy 실행을 직렬화하고 CPU affinity 3/BelowNormal 및 공통 C 드라이브 target을 사용했다.

| 검사 | 결과 | 소요 ms |
| --- | --- | ---: |
| 최초 fmt | 부모 모듈 선언 정렬 실패, 원 로그 보존 | 2487 |
| 부모 두 파일만 정렬한 fmt-r2 | exit 0 | 4065 |
| 기본 기능 cargo test --offline --locked -j1 | exit 0, 426 통과·23 ignored·실패 0 | 348318 |
| cargo clippy --offline --locked -j1 --all-targets -- -D warnings | exit 0 | 38836 |
| root --lib --features native-proposal-window check | exit 0 | 59225 |
| Rust lock identity 감사 | 기존 555개 보존, 신규 136개, 제거 0 | 160 |
| normal dependency tree 기능 off/on | 양쪽 exit 0 | 1634 / 1355 |
| UI19 순서별 SHA 감사 | 19개 모두 일치, exit 0 | 108 |

lock identity는 name/version/source/checksum 집합을 비교한다. diff에서 이동처럼 보인 dtoa/dtoa-short/oxc_allocator도 기존 식별자를 유지한다. dependency 문자열의 변경과 활성 기능은 이 집합 비교와 구별한다. 저장한 `tree-off.stdout`에는 meari-native/로컬 blitz-paint/winit 0.31이 없고, `tree-on.stdout`에는 해당 Native 그래프와 승인된 로컬 paint 경로가 있다. 과거 CLI73 실행 결과를 이번 실행 횟수에 합산하지 않았다.

## 핀과 인계

등록 다섯 파일의 `final-registration-pins.json` SHA256은 `36914414B13F3C05D0B1EE32F3983203DEE533BB85071CF9D7DB3201B7545A8B`다. 동결 wrapper 다섯 파일 관측 inventory는 `22E1F7C92A17A6B23C36184F4158A51CCA88F15AA70239F109A76BEE8E2453ED`로 이전과 같다. UI19 기준 inventory `26335921B555B8732B3609F9D22A7AD7DDA93B9F5E3D4998111381818774F35C`의 정확 목록만 해시 관측하고 Rust 감사로 일치 확인했다.

과거 SDK 전체 inventory E195…에는 이번에 승인 변경한 Cargo/부모 모듈도 포함되므로 전체 핀이 그대로라고 주장하지 않는다. 현재 단계의 변경·동결 관측을 별도 inventory로 보존한다. Native 316개 기준 2827…은 인수 기준이며 이번 작업에서 새 전체 Native 감사를 수행했다고 주장하지 않는다.

모든 실행 receipt는 terminal이다. 담당 실행 프로세스의 종료 관측과 슬롯 반환은 같은 증거 디렉터리의 인계 기록에 남긴다. 다음 담당은 canonical UI façade를 등록하고, 동일 source를 중복 path module로 컴파일하는 타입 혼합을 방지해야 한다.
