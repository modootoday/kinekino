---
id: evidence:kinekino/20261029114300
title: 기존 PNG crop SDK·CLI 실행 기록
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기존 PNG crop SDK·CLI 실행 기록

2026-10-20 Windows 실행 관측이다. [스냅샷 기능 정본](../../player/application/production/snapshots.md)의 명시 pixel crop 일부와 [후속 계획](../../../plans/player/snapshot-export.md)을 연결한다. PNG 편집 UI·전체 제품 완성 인수는 아니다.

## 입력과 책임

base `a611f791bad209f156a95d1f2e26cc2dce8499ec`의 SDKsurface4 r2·자막 range 통합본에 backend A75/API9077/evidence fixture46933 순서로 정확 diff를 병합했다. root 공유 파일을 옛 full-file source로 덮지 않고 자막 `export_subtitle_range`와 PNG `export_snapshot` typed method를 모두 유지했다.

원16 pin은 `25CFE6A7C47E48A3E90693ABC79480B63B64369B1B5A310B27D4D85F74B857BB`다. 기존 parity fixture의 CLI-equivalent SDK에 실제 PNG port를 주입하는 PM 승인 시험1을 별도 pin `AD9C8EABA716C3163ACCEB9BE3F834BC46580C7DF56FB8894C661E01A21C7494`로 추가했다. 이는 실행 전 읽기에서 확인한 adapter 차이이며 실패 시험으로 재분류하지 않는다. 기존 자막 검증/모든 assert·malformed·Unsupported·schema 루프와 desktop status 비교 의도를 보존했다.

최종17 pin `558614AFB9D06B29EC5803F33B2C27EA317820463AA73F6DF89BC796442D5F99`. 원 bytes/신규 부재와 원 모든 pin은 `runtime/quality-review/png-export-integrated-r1/` 및 coordination 각 폴더에 보존했다. SDK/API/CLI는 같은 application port·domain bounds·PNG infrastructure·atomic no-overwrite publication을 사용한다. 새로운 PTS·revision·모델 결과를 만들지 않는다.

## 실제 결과

| 범위 | 실제 결과 | 원 receipt 시간 |
|---|---|---|
| PNG backend | 14pass/0fail/0ignore | 98480ms exit0 |
| 실제 typed SDK/CLI/endpoint | 7pass/0fail/0ignore | 76917ms exit0 |
| 기존 capture lib | 5pass/기존 ignored1 | 3538ms exit0 |
| 공유 CLI contract/identity/capture/parity/자막 range | 4+1+2+2+4=13pass/기존 ignored6 | 107244ms exit0 |
| root default all-target clippy `-D warnings` | 통과 | 44045ms exit0 |
| root native-feature all-target clippy `-D warnings` | 통과 | 32939ms exit0 |
| root fmt check | 통과 | 1635ms exit0 |

실제39pass와 ignored7을 구분한다. FFmpeg/실제 미디어 의존 ignored는 실행하지 않았고 미디어 캡처 성공으로 세지 않았다. 정상 backend/API 시험을 반복하지 않았다. conservative2/-j1/RUST_TEST_THREADS1, 고정 runner r3 SHA `EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D`를 직접 사용했고 build4 재시도0이다.

PNG static8/16bit numeric/alpha·palette/subbyte crop, bounds/overflow·공통 read/pixel/decoded 예산, APNG Unsupported, IHDR/IDAT/IEND 및 ancillary CRC·zlib Adler32 matching-SHA 반례, 원본 불변·새 파일 no-overwrite를 실제 검사했다. metadata를 복사하지 않는 정책과 입력 checksum 검증은 별개다.

## 실제 CLI와 원 증거

`runtime/quality-review/png-export-integrated-r1/artifact/player-cli.exe`는 실제 사용한 dev/debug/default-feature binary다. SHA256 `BF6CFB8874E688CB98ACE638325DC53CC68486F3B6C88D2489EEAF47C50C8DB1`, 28678656bytes이며 cache/preserved bytes hash가 같다. 실제7개 test binary의 path/SHA/bytes는 `test-binaries.json`에 있다.

실제 CLI 호출11회의 stdin 원 JSON/stdout/stderr, source/output before/after 사본과 SHA·존재 여부·PID/exit·endpoint 관측은 `cli-evidence/`와 `cli-evidence-index.json`에 있다. 보존 artifact hash 대조 차이0, terminal11/fixture timeout0/I/O worker33 joined다. endpoint 미주입은 Unsupported·출력 absent→absent, 주입은 실제 SDK counter0→1·새 출력 present다. capability/schema 원 응답, context identity 원 bytes, 자막 range 결과도 각각 별도 evidence 폴더에 보존했다.

## 마감과 한계

원 실행 handle7 종료, receipt7 terminal0, 기록PID25 부재를 확인했다. final-source/owned/protected JSON은 실제 `[]` 파일로 저장했다. root/snapshot17 SHA delta0, 보호 root865/snapshot594 delta0, root index empty다. heavy/snapshot 소유권을 PM에 명시 반환했고 stage/commit은 하지 않았다.

Windows file adapter/API/CLI만 검증했다. non-Windows Unsupported는 소스 계약이며 다른 플랫폼 실행 결과는 아니다. GUI·기본 Player 전환·native video/OS slot·모델/다운로드·release 배포는0이다. numeric sample/alpha 보존은 ICC/EXIF 등 metadata나 color-managed 시각 동등성 보존이 아니다. 출력64MiB는 encoding 후 publish 전 한도이며 total peak memory 한도가 아니다. 같은16bit encoder/decoder 비교를 독립 golden/endian 검증으로 확대하지 않는다. 정상 child/I/O cleanup 관측을 모든 timeout/kill/reap/metadata 오류 분기의 fault injection 검증으로 확대하지 않는다.
