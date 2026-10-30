---
id: evidence:kinekino/20261029114556
title: 명시 JOB history 주입과 제품 통합 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 명시 JOB history 주입과 제품 통합 검증

채택 commit: `d7c8ea26166ace75342be85b213329d687ef1155`.

2026-10-23. 정상 desktop launch의 `--job-root`를 기존 guarded read-only jobs factory와 원 PlayerSdk에 연결했다. 신규5와 관련 회귀8을 실제 고정 BIN에서 먼저 통과했다. 이후 default54 target의 실제 등록1424개와 합성 실행 근거를 전수 대조하여 **1368 pass/56 ignored/미해결 실패0**, 별도 **compile-fail doctest7 pass**를 확인했다. workspace default all-target strict, root Native feature all-target strict, 최종 fmt도 통과했다.

중요한 범위 차이: 전체 `cargo test` 명령23/33은 기존 시험 fixture의 기대 누락으로 각각 실패했다. 원 실패를 고친 후 이미 성공한 prefix와 해당 실패 실명의 재실행, 미실행 suffix를 합쳐 coverage를 완성했다. **한 번의 전체 cargo test 명령이 exit0이었다고 표시하지 않는다.** Native는 컴파일/strict만 통과했고 창·렌더러·수동 시각·browser runtime 인수는0이다. 제품 전체/실제 ffprobe·provider·모델 인수도 아니다.

## 소스와 조립

기준 root `ec609170592e1098e3dddba933d8fc47ee53c2cb`, library `7fb53c99c1f550d651009d1f4fda6aa7a6a1a77b`, lock `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2`. gate는 `runtime/verification/sdk-job-read-integration-r1/`이다.

원 JOB source2 manifest `45AB26E0E71B8617F3F0F53E19E4CEFBBF7E89CB56D14523BABBD5C44D7B84AA`는 불변이다. 최종 `runtime/development/sdk-job-read-launch-r1/final-source5.json` SHA **6F95CB48519C89BFB5657032D655686A8EBE41675C9EE5B42A3F925F01ADA8F5**, before5 **0E6E3C4F7F2424009EF506248618C6BF115087E09654028E9FC9DDB156BDDFA4**다. source5는 launch2와 실패 기반 test3이며 시험명 추가는 원 신규5뿐이다. 최종 union24 SHA **E4185D4C242AEEE00B50B0684F3CA92C40C0AACA52B8FC29273EEE38368A2A4A**는 채택 library17 + 이미 채택된 product mirror2 + JOB/test5다. 기존18/new6 외 소스 합성0이다. manifest의 보호 pending 문구는 봉인 당시 상태이며 이후 보호 완료는 아래 원 관측과 STATE에 따른다.

- `src/application/launch.rs`: 기존 storage_root 검증을 재사용해 명시 absolute `--job-root VALUE`/`--job-root=VALUE`를 받는다. 미지정 시 기본 경로를 만들지 않는다. 중복·누락·상대 경로·design-preview 혼합을 거절한다. 공통 오류문구가 실제 option을 가리키도록만 정리했고 기존 검증 정책은 유지했다.
- `src/main.rs`: 같은 PlayerSdk가 Arc/LocalHost/ExecutionOwner로 넘어가기 전에 기존 `jobs::read_only(root)`/`with_jobs`를 주입한다. helper는 원 typed ApiError를 보존하며 main만 기존 초기화 오류 형식으로 표시한다. 새 task worker/endpoint/SDK/정책/registry/hot-swap은 없다.
- factory는 read capability만 제공하지만 `JobFiles::open`의 short writer lock과 job-events/job-sources 초기화 부작용은 있다. guarded 기존 root가 필요하며 잘못된 root를 임의 생성하지 않는다. read=true/write=false/자동 task 재실행=false다. UI는 root를 선택·생성하거나 실행 중인 unconfigured handle을 attach/release로 upgrade하지 않는다.

신규5는 actual list의 `application::launch::tests::job_root_reuses_explicit_local_storage_selection`과 player target의 `job_read_tests::{absent_job_root_preserves_unconfigured_sdk,configured_root_reads_original_store_without_job_worker,invalid_job_root_preserves_original_guard_error_without_creating_root,same_configured_client_flows_through_original_execution_owner}`다. 기본 Unsupported, 원 파일 factory와 capability·빈 page·missing get·write 거절·root 오류/파일 보존, 같은 configured Arc→원 SDK ExecutionOwner의 terminal/ACK/close를 확인했다. 이는 GUI route/handoff/render 인수가 아니다. nonempty journal의 실제 CLI 근거는 [기존 jobs 검증](jobs-2026-10-19.md)을 유지한다.

## 실제 실행과 coverage

fresh RAM22.71GiB/VRAM6226MiB/C119.92GiB/writer0에서 시작했다. canonical 제품 S/T, runner `EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D`, CPU2/mask3/BelowNormal/descendant Job/j1/thread1/offline/locked를 유지했다. source mirror21 후 실패 기반 test3만 별도 revision으로 추가했다. library lexical/UI 후보/Media notice10/새 target/clean/다운로드/GUI/모델/WSL은0이다.

| 단계 | 원 근거 | 결과/범위 |
| --- | --- | --- |
| 우선 빌드/list | 03/04/05 | root library710/player4 실제 등록, JSON fresh=false·고정 사본 |
| 신규5+회귀8 | 06..18 | 각각 exact1 pass; 전체 coverage에 중복 합산0 |
| default no-run/list | 21/22-list-* | test BIN54/normal BIN2, 실제 목록 모두 저장 |
| 원 전체 실행 | 23 | root lib704 pass/1 fail/5 ignored; 다른 target 완료 아님 |
| 원 parity 단독 | 25 | 빈 export payload에서 exit2 vs 기대3, 원 실패 보존 |
| 수리 우선 확인 | 31/32 | root malformed matrix와 실제 SDK/CLI parity 각각 pass |
| 새 전체 실행 prefix | 33 | 앞 target들을 통과하고 jobs37 pass/1 fail/14 ignored에서 종료 |
| provider 수리 확인 | 38 | 원 catalog job/파일/실제 CLI readback 포함 같은 실패 실명1 pass |
| 미실행 suffix | 40-remaining-01..43 | 원 JSON member cwd/고정 BIN, 모두 exit0; 이미32에서 통과한 parity 첫 case만 중복 skip |
| doctest | 41 | workspace12 crate, compile-fail7 pass |
| strict/fmt | 42/44/45 | default workspace all-target strict/root native-proposal-window all-target strict/최종 fmt exit0 |

`default-effective-test-names.json`의 package|target|실명1424개와 실제 list inventory를 비교해 missing0/unexpected0이다. prefix의 실패 provider row는 같은 실명의38 pass로 대체하는 관측 관계를 남겼고 원 row는 raw observation에 보존했다. 합성1368 pass/56 ignored와 doctest7을 구분한다. 공유 helper가 여러 target에 등록된 것은 해당 등록의 실행 수이며 새 제품 기능 수가 아니다. 이전 download23/Meari76/priority13/failedcase 재실행을 추가 pass로 합산하지 않았다.

138 runner receipt 모두 terminal이며135 exit0, 원23/25/33의 exit101 세 개를 보존했다. 우선 빌드133907ms, default no-run307478ms, default strict6698ms, Native compile strict68523ms는 이 자원 제한 실행의 관측이며 성능 benchmark가 아니다. Native/browser 접근성·focus/selection/IME·UI 기본 JOB route는 별도 인수가 필요하다.

## 기존 시험 수리의 근거

1. r2 `src/application/api/tests.rs`: UnavailableSkipHost는 export_preflight에서 strict payload→context/revision을 검사한 뒤 Unsupported를 반환한다. 빈 export4는 InvalidArgument가 정상이다. malformed matrix의 이 네 기대와 이유만 수정했다. production guard0.
2. r3 `tests/sdk_capability_parity.rs`: 원25가 `segments.export_consume`의 빈 payload로 actual exit2/기대3 실패함을 확인했다. malformed4의 SDK/CLI InvalidArgument/exit2 비교를 먼저 보존하고, 기존 public typed DTO·canonical lookup codec으로 형식이 유효한4의 Unsupported/exit3 비교를 추가했다. lookup parse는 issuer/control 권한 발급이 아니다. fake backend나 같은 빈 요청의 성공 위장0, 새 시험명0.
3. r4 `tests/jobs/provider.rs`: BrowserCatalogJobs의 catalog.search/episodes.list2와 SubtitleJobExecutor의 convert/export_range2가 실제 registry4다. 오래된 len3을 len4로 치환하는 대신 명시 등록 집합·schema discriminant 동일성과 중복 부재를 확인했다. 미지원 operation을 지원으로 가장하지 않았고 실제 catalog job/immutable file/CLI readback은 유지했다. 새 시험명/제품 정책 변경0.

원 source2, 각 before/source revision, 고정 구 BIN과 raw request/response/evidence를 보존했다. 무관 target을 skip해 성공으로 처리하지 않았다. default single-command green의 미증명은 위와 같이 그대로 남는다.

## 고정 BIN과 보호

- 원 priority library `CAA9304BE316DFFD89DCA320C7C8268F256B7B054C2F10686107E94153531C7E`/100694016B와 player test `B148BFE6BBBDFB3B344B09AE0FCCD2B81F845E4106B472B2B32D2B443A5DC802`/9124864B를 유지했다.
- 수리 library `EAE6409A085DFA051697361BAE84049C24C07D2BDC1FF438A107047C06B71C8D`/100694528B, parity `3B88DE30E5FC8344A0381890AEABB6AB7CD2220F891804797CAEB0906683DD78`/9677312B는 새 JSON/list/사본이다. provider는36-fixed-jobs-r4.json이 정확한 pin이다.
- root tracked1320/library tracked485/index `2C96F544D280964F06EFACB72C57C97B88226137284FDF66BBB2EF4E02A69B47`는 변경0이다. S 기존4662 중 byte 변경18은 union24의 기존 입력뿐, 신규 소스6도 정확히 일치한다. 추가190개는 broad Rust 시험이 만든 runtime evidence이며 공개/컴파일 소스로 승격하지 않았다.
- byte 동일 S4644의 UTC ticks mtime 차0이다. 최초 DateTime/string 비교 false-positive JSON도 보존하고 normalized observation으로 정정했다. touch/복구0이다.
- 기존 T exe531 중 변경14/신규40을 기록했다. 52개는 Cargo JSON producer에 직접 대응하며 normal player/CLI의 deps alias2는 각각 같은 volume/FileID와 hash로 대응한다. 새 normal GUI SHA `6818E174484B22767E2DE207D78D326471E64C102A0E8DB00C3E3D29A0E6C144`, CLI `4B241437D24CED0832BBCCDA9BD87F0AA0F24DB748A44D93A120B9D8332555CB`. normal 전후 원본/alias4의 실제 preimage 사본도 보존했다. CLI는 실제 parity/회귀에 사용됐지만 정상 GUI 창 실행은0이다.
- fixed test/normal pin61과 child2, 이전 owned-probe 고정 사본5를 유지했다. known4709 미귀속 alias2도 원 bytes 그대로이며 attribution 미해결/all-cache-protected=false를 유지한다. 전 캐시 안전성을 주장하지 않는다.

근거는 protection-summary, mtime-normalized-observation, final-input24-observed, new-S-observed, binary-producer-records/binary-attribution-observation/binary-alias-fileids, normal-preimage-copy-observation, fixed-bin-protection, known-cache4709-observed, final-process-observation이다. 마지막 gate/T 실행 프로세스 및 build 프로세스는 부재였고 추가 실행 없이 STATE/slot-return으로 중앙 자원을 반환한다. 소스 채택은 중앙 PM의 별도 단계다.

## 남은 범위와 적용 스킬

단일 전체 cargo test green, UI default JOB route의 원 attachment 인계·오류 표시·renderer/runtime 인수와 수동 시각 검토는 구분해 남긴다. 실제 pinned ffprobe의 protocol/license/media, original provider/header/auth/HTTP host 수명, HLS VOD·remux, Range/동일 객체 재개·정식 재리졸브·재시작 복구, download modes/batch와 실제 product CLI 성공·오류·취소·저장, AI 입력/출력/모델 품질 전체 목표도 계속 남는다. 기존 [download owned 검증](../production/download-owned-probe-2026-10-23.md)을 제품 전체 완료로 확대하지 않는다.

이미 읽은 modootoday/software-qa/test-fixture-and-mock-build SKILL SHA `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`을 원 factory/typed shape/고립 evidence/teardown에 적용했다. code-change-review-verification SKILL SHA `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`, verdict-table reference `9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9`를 원 실패·실제 BIN/list·고유 명단·합성 coverage/명령 성공·보호 한계 구분에 적용했다.
