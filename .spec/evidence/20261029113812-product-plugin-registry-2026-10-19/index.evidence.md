---
id: evidence:kinekino/20261029113812
title: 제품 자막 플러그인 실제 registry 검사 r2
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 자막 플러그인 실제 registry 검사 r2

2026-10-19. PM으로부터 Meari 공개 PluginAdapter 서명과 단일 검사 슬롯을 인계받아 기존 canonical codec을 실제 PluginRegistry에 연결했다. [r1](product-plugin-codec-2026-10-19.md), [host 계약](../../player/presentation/shared/plugin-conversion-host.md)을 따른다.

## 명령과 결과

공통 manifest는 packages/player-ui-plugins/Cargo.toml이다. BelowNormal/-j1/기존 공유 target/오프라인을 사용했다. root Cargo/lock/SDK와 Meari 본체는 수정하지 않았다.

| 검사 | 결과 |
| --- | --- |
| cargo fmt --check | exit0 |
| cargo test --offline -j1 | exit0, 기존3+등록3=6회귀, compile1분10초 |
| cargo clippy --offline --locked --all-targets -j1 -- -D warnings | exit0,29.68초 |
| cargo check --offline --locked --target wasm32-unknown-unknown -j1 --lib | exit0,1분14초 |

첫 test에서 독립 lock에78개가 추가됐다. 정상 build를 중단·정리·재시작하지 않았고 전체 종료 후 슬롯을 반환했다. `runtime/quality-review/player-ui-plugins-r2/`에 logs/source snapshot/source-hashes/core-api-hashes를 보존했다. adapter SHA256은 `05926e1d7138ec4407a34a854647da17e44e1a4ada29632f198db3a4ec07269e`, 등록 시험은 `2c7e60f5656de2106e6d4699522fc46c76c4616b7959912f8638ac2007baeb02`, lock은 `37a6ce7b1e641699fe56d54d653f11b4f2b16c484f2dec21d503bbbb1c5a5e32`다.

## 입증한 범위

MeariSubtitleConversion은 실제 PluginAdapter를 구현한다. host가 provider/contract/request/reply/error hash와 target 목록을 주입하고 registry가 grant를 검사한다. invocation ticket을 host closure에 전달하며 기존 codec/canonical DTO를 사용한다. 결과는 Immediate이며 SDK 오류는 Adapter error의 error-schema details로 ApiError 전체를 보존한다. pending 작업·progress event·OS 자원을 소유하지 않으며 없는 취소를 성공으로 위장하지 않는다.

등록 시험3개 중 첫 성공 경로는 각 Native/Web target 설정에서 canonical convert::execute+메모리 저장소의 SRT→VTT 성공·새 출력 경로·overwrite=false·cue1·한글 본문을 실제 registry로 읽었다. 오류 시험은 canonical UnavailableStorage 오류가 Adapter/details/ApiError Unsupported로 전달되고 ticket 기반 host request ID가 유지되며 완료 ticket cancel=AlreadyTerminal·unregister 뒤 Stale임을 확인했다. 거부 시험은 grant 없음과 schema hash 불일치가 SDK 호출 전에 거절되고 호출 수가0임을 확인했다. 기존 codec 시험3개도 다시 통과했다.

provider/contract pin은 명시 테스트 식별 hash다. metadata 공급이 provider 진위 검증을 의미하지 않고 capability grant가 OS permission 승인을 뜻하지 않는다. target enum을 Web으로 선택한 Rust host 시험은 browser/WASM 실행이 아니다. WASM check는 library 타입/컴파일 경계까지만 입증한다.

## 남은 경계와 source binding 진단

root ApplicationClient/RequestContext 조립, Native OS 파일 publication, Web transport/권한/resource, renderer/브라우저·폰트/SVG·시각 인수는 남아 있다. 일반 registry의 manifest 버전/target/pending/HMR 전체 회귀는 본체 담당 범위이며 이번6회귀가 전체 인수를 대신하지 않는다.

최소 source는 verification/subtitle-convert-intent.tsx다. 기존 고정 consumer-r2 binary(SHA `10344ad407e4d168d631cb175976ef127bea526df06b8b6138b7b466430ba074`)의 check는 exit1, file=verification/subtitle-convert-intent.tsx, code=unsupported, span85..107, message=`prop type must be string/boolean`이었다. source/manifest/stderr를 같은 checkpoint에 보존했다. 고정 binary의 baseline이며 최신 parser 검사를 대신하지 않는다. Meari 담당에게 typed callback/source IR 요구와 같은 consumer 재검증을 인계했다. Rust 등록 성공을 TSX 버튼→SDK 호출 성공으로 보고하지 않는다.
