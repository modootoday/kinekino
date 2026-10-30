---
id: evidence:kinekino/20261029114544
title: R41 검토 저장소 discovery 분할 실행
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R41 검토 저장소 discovery 분할 실행

후속 [R44 명시도구구성](cli-review-pinned-discovery-20261028.md)은4분할capabilities동일·97schema정합을확인했다.아래R41자동탐색변동/미인수는그원실행사실로보존한다.

2026-10-28 중앙 PM. 원1308 시간 초과를 보존하고 Rust 시험만 변경했다. 제품 기능 코드·기존 WIP/소유권·추론/다운로드/수동GUI/WARP/push 보류를 유지한다.

## 입력과 변경

원 `tests/cli_discovery_inventory.rs` preimage는 mirror의 `runtime/verification/preimage-r40-review-discovery/cli_discovery_inventory.rs`, SHA B90CAF6ED09EFB13A31FD8E46CC66287F5036B38D520F302C0E71002119F9018. 변경 후 SHA914B6D1975F00B026D094332485E6DBA34B21D49C91650FB51F720CFC1C5CB4D.

네 명시 test가 같은 helper를 shard0..3으로 호출한다. 각 capabilities97개·중복없음·review storage9 supported/playback4 unsupported를 검사하고, 원 순서 index%4로 schema25/24/24/24개를 나눈다. 각 schema의 명령/지원 상태·원본 보존을 확인하고 관측 schema 이름·shard·CLI 핀을 기록한다. 기능 검증으로 계상하지 않는다.

새 input-r41 SOURCE SHA7E7E31E6E0DFBDA09FF9DB29825F47B59603D181E948BD885F1E30504DDCBE6D, inventory1F74F68366F9A581D13F53F5D9A6E21BD7DDCE47CA33CA7482CA08EA2CBEAF1D/2297. snapshot은 컴파일 후·첫 shard 진행 중 읽기 전용으로 생성했으며 source는 컴파일 시작 후 변경하지 않았다. 과거 R40 입력의190/bins를 R41 재실행으로 주장하지 않는다.

1309 no-run exit0/10416ms,1310 해당test strict exit0/804ms,1312 전체fmt exit0/6449ms,1316 Native all-target strict exit0/824ms,1317 default all-target strict exit0/798ms. 새 producer fixed-r41/cli-discovery-inventory.exe SHA55952086EEA12F5FB4A8705850AED92DB106E37C124F7AB2EB9E27A7AEADD874/1932800. 정상CLI ED1D/36718592는 원R40과 동일해시다. C2 baseline index는 기존R40 discovery 설명자료를 재사용한 operator snapshot이며 R41 컴파일 provenance는1309와 이 기록으로 연결한다.

## 실제 실행

후속 verify-current는 동일 R41 SOURCE 핀에서 terminal exit0·2297 content/identity unchanged를 반환했다(직접 실행 handle77919, 별도 runner receipt 없음).

|로그|shard|실제 선택 결과|runner terminal|
|---|---|---|---|
|1311|0|1pass/5filtered,30.96s|exit0/42321ms|
|1313|1|1pass/5filtered,27.51s|exit0/36105ms|
|1314|2|1pass/5filtered,27.93s|exit0/36512ms|
|1315|3|1pass/5filtered,28.53s|exit0/39824ms|

독립 `runtime/verification/source-sdk-r41-review-shard{0,1,2,3}-os1/outer` 및 각 TMP fixture가 원 증거다. 무거운 실행은 직렬이며 total60000/cleanup5000·CPU2/BelowNormal 유지. snapshot 읽기 및fmt만 첫shard와 중첩했다.

## 발견된 구성 불안정과 인수 한계

shard0/1/3의 capabilities 전체 응답 SHA는0F0DF980CB9560B395FCCC9810960832D5749B4FF387EF1BDA226DC79E12C669, shard2는7B9A2375C94F742E0F1591A29FA1C56DC1A5F05DDD51B0266942621E35EA442B다. native git word-diff는 지원 상태 supported→unsupported, frame capability true→false, FFmpeg8.1.1 version→null 및 unavailable_reason 발생을 보여준다. 따라서 네 결과를 단일한 안정 구성의 전체 schema 인수로 합치지 않는다. review9/4 경계와 각 shard의 실제 일치만 증명한다.

현재 frame_capture::discover는 exe 인접/Chocolatey/PATH에서 최초 도구를 선택하고 version/encoders/filters를 각각2초로 확인한다. 도구 해시가 입력으로 고정되지 않고 검사 오류도 일반 unavailable reason으로 합쳐진다. 실제 원인이 시간초과인지는 추가 증거가 필요하다. 다음은 현재 선택된 도구 경로/해시·원 실패 관측을 보존하고 명시 도구 구성 및 실패 진단 계약을 검토하는 일이다. 임의 시간 연장·도구 다운로드·전역PATH 변경으로 통과시키지 않는다.

fixture/mock skill SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 원 실패 보존·scope별 판정을 적용했다.
