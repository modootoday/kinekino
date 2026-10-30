---
id: evidence:kinekino/20261029113626
title: Runtime notice bounds default gate
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Runtime notice bounds default gate

2026-10-23 PM 명시 sole-heavy/shared 배정. root `d7c8ea26166ace75342be85b213329d687ef1155`, library `7fb53c99c1f550d651009d1f4fda6aa7a6a1a77b`, lock `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2`. 실행 근거는 `runtime/verification/vad-runtime-notice-bounds-r1`이다. 기존 제품 S/T, runner EEA9DEE6, conservative2/CPU2/BelowNormal/descendant Job/offline/locked/j1/thread1을 사용했다. RAM29.16GiB/GPU6304MiB/C113.51GiB/writer0에서 시작했다.

## 입력과 실제 결과

원 source10 `5E83E0DEE3C511D26C4961EC989E7660A5087EA71621A6BAEF6105695B5F3E1D`와 before10 `B9C9EFFCACE63333765CBB8BC56FFFA48717811909C9C5B2EA0CEC3C89902DE3`를 보존했다. existing8/new2의 root/S preimage와 기존 S tracked root/library 정합 차0을 확인했다. fmt01이 path-mounted files.rs의 tests 경로 해석에 실패해 r2에서 명시 path 한 줄만 추가했다. strict03은 신규 test의 Instant import 누락으로 실패해 r3에서 import만 추가했다. 두 원 실패와 각 revision은 보존했고 제품 guard는 변경하지 않았다.

최종 source10 r3 `42D72ABD735CA0C6FA5A687A2E71F8BDB94BA90AE78D3AAF1848BB07B3104455`는 같은 exact10이다. fmt02/final06·관련4개 package all-target strict04·library Cargo JSON no-run05가 통과했다. 실제 고정 BIN4의 등록은 root712/vad5/executor66/worker41=824다. 그중 신규17+기존14의 exact31을 실행해 각각1pass를 확인했다(root2/vad5/executor17/worker7). 나머지793 등록은 이 gate에서 실행하지 않았다. 전체 cargo test 성공이나 native 지원으로 확대하지 않는다.

공통 notice 정책은 개별1MiB·합계1MiB·최대16개이며 optional 빈 목록과 native 필수 목록을 구별한다. 실제 TPN338088B/ORT license1073B 크기와 경계·합계·개수·원 파일 guard·SHA 변경·원 cancel/cutoff를 검증했다. 시험 바이트는 synthetic fixture다. 모델 license64KiB·manifest64KiB·child wire8192B는 유지한다.

## 보호와 반환

root1320/library485 bytes 차0, root index stage0이다. S 기존4861 중 입력8만 바뀌고 신규2만 추가됐다. 비입력4853의 bytes/UTC ticks 차0이다. 초기 JSON datetime/string 비교의 mtime 오탐은 원 관측을 보존한 채 protection-S-normalized.json에서 UTC ticks로 대조했다. 전역 touch/clean/새 target은 없다.

T 기존571 중 변경4/new0이며 각 변경은 build05의 Cargo JSON executable/SHA와 일치한다. 원 normal GUI/CLI preimage 복사와 FileID, 기존 fixed61 및 knowncache4709 두 bytes가 보존됐다. 기존 캐시 producer 미귀속2는 여전히 미해결이며 all-cache 보호를 선언하지 않는다.

runner41 모두 terminal(39 exit0/원 실패2)이다. 실제 시험은 소유 runner의 terminal receipt와 원 handle 종료를 기준으로 마감하며 보조 프로세스 관측만으로 정리를 대신하지 않는다. 명시 반환은 gate의 slot-return.json을 따른다. root 적용·commit은 PM 소유이며 push0이다.

ORT/native feature·제품 record 활성화·worker native 빌드·모델·추론·다운로드·GUI0, PRODUCT_RELEASE=None을 유지했다. [후속 체크포인트](../../../plans/player/inference/vad-native-product-checkpoints-2026-10-23.md)의 native compile/load·bundle/audio2·일반 CLI·사용자 품질은 미완료다.

modoo.today `modootoday/software-qa/code-change-review-verification` 본문 SHA256 `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`와 필수 `references/verdict-table.md` `9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9`의 원 실패 보존·예상/등록/실측 판정 분리를 적용했다. lint-and-validate 스킬은 프로젝트 Rust fmt/test/clippy 규칙으로 적용했으며 타 언어 개발 도구는 도입하지 않았다.
