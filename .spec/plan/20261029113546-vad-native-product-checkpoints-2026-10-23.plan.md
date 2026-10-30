---
id: plan:kinekino/20261029113546
title: VAD native 제품 연결 후속 체크포인트
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VAD native 제품 연결 후속 체크포인트

2026-10-23 중앙 조율. notice10 채택 후 현재 기준 root89c0fa0da2ea63aa82cd36b79ad7994e128ac06a/library7fb이며, 실행 권한은 [중앙 배정](../execution-leases.md)만 따른다. notice default gate는 반환됐고 Media는 source-only다. [출처·정밀도·모듈·release 잔여](../../../sots/player/infrastructure/inference/research-followups/vad-native-release-gaps-2026-10-23.md)를 기준으로 다음 입력을 구분한다. 아래 native worker 빌드·추론·다운로드·GUI 실행은 아직 배정하지 않았다. PRODUCT_RELEASE=None을 유지한다.

## 1. Notice 정책 후보 검증

후속 중앙 배정으로 default 단계는 [실제31 정본](../../../sots/verification/ai/vad-runtime-notice-bounds-2026-10-23.md)으로 마감했다. 아래 source-only는 최초 작성 이력이다. 최종 r3는 같은 exact10 내 시험 경로/import만 수리했으며 source10 SHA256 `42D72ABD735CA0C6FA5A687A2E71F8BDB94BA90AE78D3AAF1848BB07B3104455`, fmt/관련 strict 및 신규17 포함 exact31pass다. Native 빌드 이후 단계는 여전히 별도 배정 전 미완료다.

진행 관측: 최초 fmt의 path-mounted tests 해석 실패를 r2의 명시적 `#[path="files/tests.rs"]`로, strict03의 시험 Instant import 누락을 r3로 수리했다. fmt02-r2와 관련 root/media-vad/executor/worker all-target strict04-r3는 통과했다. Cargo JSON의 고정4 BIN에서 예상31개 등록을 확인했고 `runtime/verification/vad-runtime-notice-bounds-r1/actual-results.json` 및 exact31 원 로그는 모두 exit0·각1pass다(신규17 포함, 별도 합산하지 않음). 원 실패와 후보 revision은 보존하며 정리·최종 보호 확인·명시 반환·소스 채택은 아직 미완료다. 아래 원 source10 해시와 actual0은 동결 당시 이력이다.

`runtime/development/vad-runtime-notice-bounds-r1/HANDOFF.md`의 exact10을 동결했다. source10 SHA256 `5E83E0DEE3C511D26C4961EC989E7660A5087EA71621A6BAEF6105695B5F3E1D`, before10 `B9C9EFFCACE63333765CBB8BC56FFFA48717811909C9C5B2EA0CEC3C89902DE3`. 기존8/new2, leaf rustfmt/check10 통과, 신규 작성17/상속 포함31이며 컴파일·타입·actual0이다. 중앙 독립 검토에 확정 P1/P2 없음은 실행 통과를 의미하지 않는다.

공통 Rust domain 정책은 notice 개별1MiB·합계1MiB·최대16개다. 공통 빈 목록은 허용하고 기존 native 필수 목록 검사는 호출자가 유지한다. 실제 확보 TPN338088B와 ORT license1073B를 원문 그대로 수용한다. 모델 license64KiB·manifest64KiB·child wire8192B 및 원 SHA/ordinary-file/owner/deadline/취소/정리 계약은 유지한다.

SDK 슬롯 반환 뒤 PM이 명시 입력·S/T·runner를 배정해야 한다. 먼저 default fmt, 관련 root/media-vad/executor/worker strict와 library test를 offline/locked/j1로 수행한다. 실제 시험 목록을 authored-tests17.json과 대조하고 직접 영향 받은 기존 시험을 함께 실행한다. 중복 소비자는 한 번만 실행하며 첫 실패를 보존한다. 이 단계에서 ORT feature·모델·제품 issuer를 활성화하지 않는다. root 적용·명시 경로 stage/commit은 PM 소유다.

## 2. Native worker 별도 빌드

기존 worker의 `native-ort`는 ort=2.0.0-rc.14, default-features=false, std/api-31이다. build.rs는 x86_64-pc-windows-msvc와 ORT_LIB_PATH를 요구하고 DirectML을 링크한다. API 소스 확인은 native 타입·링크·ABI 통과가 아니다. 기존 private spike 및 default worker 시험을 제품 native worker 증거로 대체하지 않는다.

별도 배정에서만 process-scoped CARGO_NET_OFFLINE=true/ORT_OFFLINE=1/ORT_SKIP_DOWNLOAD=1과 고정 ORT_LIB_PATH를 사용한다. native-ort strict 후 `cargo build --offline --locked -j1 -p media-vad-worker --no-default-features --features native-ort --bin media-vad-worker --message-format=json`으로 test=false artifact와 SHA/bytes/features를 확보한다. test-control-process·다운로드 fallback·전역 PATH/RUSTFLAGS 변경·cache clean은 포함하지 않는다. SDK lib-search 부족을 확인된 원인으로 단정하지 않는다.

실제 진입점은 기존22 인자→guarded parent configuration→native deployment admission→OrtVadEngine→공통 released request다. legacy16 및 feature 없는 Unsupported 경로를 유지한다. CPU EP, profile thread 수, session 생성과 module guard 수명은 기존 어댑터를 검증하며 새 정책을 CLI/GUI에 복제하지 않는다.

## 3. Bundle와 독립 제품 record

순서는 소스 검증→native worker BIN 고정→정확 artifact/manifest 고정→독립 root 제품 record→normal CLI 빌드다. worker에 자신의 SHA를 내장하지 않는다. manifest는 실제 model/license/worker/modules/notices의 정확 pin 및 원문 provenance를 사용한다. 알려지지 않은 변환 recipe·precision·package→DLL 출처·최소 Windows 지원을 문법 통과용 선언으로 채우지 않는다. 기존 공식 문서·cache 조사 근거를 재사용하고 일반 검색을 반복하지 않는다.

제품 root의 ConfiguredVadRelease, manifest/profile 결속, immutable file guards, ParentSelectedRelease 및 ChildNativeDeployment가 같은 자료를 소비해야 한다. 향후 Rust bundle 작성 도구도 독립 issuer 권한을 발급하거나 PRODUCT_RELEASE를 자동 변경해서는 안 된다. 현재 None 경로의 실제73은 이력이며 positive bundle 지원 근거가 아니다. record 활성화 시 unconfigured 시험의 의미도 명시적으로 갱신한다.

## 4. Normal SDK·CLI audio2

기존 ProductInputFactory는 절대 workspace의 audio/outputs/cache/evidence 디렉터리와 `audio/audio-{opaque_id}.wav`, `outputs/result-{destination}.json`을 사용한다. 새 workspace/미디어 복사는 아직 수행하지 않는다. dialogue/music 후보는 기존 WAV와 full-source SHA를 재사용하고 실제 PCM/sample 범위는 Rust로 확인한다. 예상400000/320000 sample이나 과거 spike score 수를 새 제품 결과로 기록하지 않는다.

공통 SubmitJob/VadJobParameters의 profile/manifest pin, audio ID, Decimal start/end, destination, expected_source_sha256와 execution_timeout_ms를 사용한다. jobs.submit idempotency_key 및 기존 watch/get/cancel/retry 계약을 유지한다. context.timeout_ms나 새 우회 CLI flag를 발명하지 않는다. 결과·오류·원 취소·재시도·원자 게시를 SDK와 일반 CLI에서 입증하고 GUI 기능 검사로 대체하지 않는다.

정상 host 생존에는 기존 owned-process의 `PreparedEvidence::create(root, None, limit)`가 제공하는 held stdin/control pipe와 원 Supervisor를 재사용할 수 있다. ready stdout을 원 cursor/deadline 안에서 확인하고 일반 CLI RPC client를 실행한 뒤 원 finish로 stdin을 닫아 terminal/family/EOF를 수거한다. finite stdin EOF용 기존 CLI3 helper를 positive host로 오인하지 않는다. 새 acceptance consumer는 별도 소유권·실행 배정 뒤 작성하며 현재 exact10에 합성하지 않는다.

## 5. 실제 runtime 관측과 완료 범위

현재 worker response encoder는 actual_runtime=null을 출력한다. native loader의 실제 module 검사는 존재하지만 이를 protocol 관측으로 보존하는 typed 생산 경로는 별도 후속이다. 설정 CPU/precision/thread 값을 실제 관측으로 채우지 않는다. 필요 시 실제 측정 snapshot의 제한된 공통 encoder를 별도 설계하고 기존 None/default/fixture 의미를 보존한다. 현재 wire에 없는 OS build를 다른 필드에 넣거나 parent 신뢰·cleanup ACK를 발명하지 않는다.

native compile/link, 실제 guarded load, 승인 bundle audio2, 오류·취소·정리 및 사용자 품질 인수는 미완료다. buildscript4709 별칭2의 producer 미해결을 보존하고 전체 cache 안전을 선언하지 않는다. 신규 실행·추론·다운로드·worker 빌드는 중앙 재배정 전 계속 보류한다.

## 적용 근거

modoo.today `modootoday/software-qa/code-change-review-verification` 본문 SHA256 `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`, 필수 `references/verdict-table.md` `9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9`를 읽고 notice 호출 경로·원 제한·작성/실행 판정을 분리했다. `modootoday/operations-cs/version-pinned-procedure-from-vendor-docs` 본문 `eb3f771f822a329bc1794cdb71aa3f49bff1f63bd82eb7ee0a36dd0c171ebb23`의 고정 버전/문서 원문/실제 payload 구분을 적용했다. 이는 배포 권한이나 추가 사용자 승인 절차를 만드는 근거가 아니다.
