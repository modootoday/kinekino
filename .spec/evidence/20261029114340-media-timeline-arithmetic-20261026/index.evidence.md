---
id: evidence:kinekino/20261029114340
title: 미디어 timebase 공통화·signed origin 계산 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 미디어 timebase 공통화·signed origin 계산 검증

2026-10-26. 기존 [실제미디어 provenance 공백](../ai/highlight-media-provenance-gap-2026-10-25.md)과 Media handoff를 읽고 현재 source/후보를 대조했다. actual9와같은원미디어/검증PTS를동시에충족한표본을새로확인한것은아니다. Sprite source102392846B와portrait932830B의현재경로/길이만확인했으며fullmediahash/FFprobe/재생/추론/다운로드/GUI실행0이다. stored해시는현재content검증으로승계하지않았다. 두후보를actual9에붙이지않았다.

기존infrastructure/frame_capture/seek.rs의유리수threshold계산을공통domain/media_timeline로옮겼다. [도메인계약](../../player/domain/media-timeline-arithmetic.md)에정규화/부호/반올림/overflow와proof경계를고정했다. signedorigin변환은새공통기반이며기존decoder의negativePTS지원이나ReviewSource Verified를완료하지않는다.

## 실제 검증

runtime/verification/source375-datalab-b0-r1-logs:

-145 media-timeline-tests:5pass/0fail,1434filtered,exit0/82560ms. canonicalratio·부호/0/잘못된format거절·90kHz음수origin/원점이전거절·소수microsecond상하반올림/exact표시·기존12288Hzthreshold·극단값overflow·여러timebase의방향반올림경계를검사했다.
-146 frame-capture-regression:4pass/0fail/1internal-child ignored,exit0/2505ms. 기존fractional/preroll/PNG/process-owner시험이며실미디어decode시험이아니다. 기존583333→7168/583334→7169·2^53표현한도assertion을변경하지않았다.
-147 최초strict는새transform의`value % divisor == 0`가manual_is_multiple_of lint로exit101이었다. 원로그/receipt보존후`value.is_multiple_of(divisor)`로수정했다. lint/검증조건억제0,toolchain/MSRV변경0.
-148 최종산술5pass/0fail exit0/56079ms.149fmt --check와150root all-targets/native-proposal-window strict exit0. 기존paint deprecation은원로그보존.

locked/offline/jobs1/CPU2/BelowNormal/C native/tmp·기존toolchain/두lock·원Media seal유지. 실제9/raw/decision/WIP·소유권·추론/다운로드/수동GUI/WARP/push보류유지. 이전9AB전체190은이새산술후계의전체회귀가아니다.

## 기존 주장과 호출 경로 점검

modoo.today 검색의OSS triage/성능·quantization/광고환산 결과는관련없어채택하지않았다. modootoday/software-qa/code-change-review-verification 본문과references/verdict-table.md(SHA9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9)를읽어기존지원주장만대조했다.

| 주장 | 판정·근거 |
|---|---|
| 기존capture의timebase가actual9 Verified를입증한다 | 반박. seek::threshold는숫자변환과double한도뿐이다. ReviewSource의`pub enum TimelineStatus { Unverified }`와`playback_ready=false`는그대로다. 실제9의content/PCM origin증명은없음 |
| 공통화후기존capture의다음tick선택/정밀도guard를보존한다 | 확인.기존`dividend.div_ceil(divisor)`를공통`ticks_at_or_after`에위임했고adapter의`ticks > (1u128 << 53)`거절을유지했다.146의기존경계시험통과 |
| 새signedorigin계산은실측origin을자동인증한다 | 반박. constructor의`Construction never authenticates that origin`과호출자가준origin만사용하는경로를확인했다. 새값은serde Verified/wire상태를제공하지않음 |

동작차이: timebase의정규화로동치표현을곱셈overflow없이비교하며,부호/공백표현은명시거절한다. API error는기존adapter의Unsupported매핑을유지한다. 기존공개capability/README가Verified지원으로바뀐것은없으며새산술계약에한계를명시했다.

## source pin

| 파일 | SHA256 |
|---|---|
| src/domain/media_timeline/mod.rs |4BFE2751886489AE12A28068F319BCAD7ACA7FFB14A71716790085850C91C08D|
| src/domain/media_timeline/time_base.rs |9E1874A28CFDB7308059881F9A6E3C3ACFF448A642A58534646C441BB24E6E4A|
| src/domain/media_timeline/transform.rs |8901B776C40C95BA5665C202CA07BCC3A5681A0E21352E83438D1180F0AE103E|
| src/domain/media_timeline/tests.rs |B00EEE00B646A3274E6F7F374E7009A343528F1C26C294606FE5803DCC6E7EB0|
| src/domain/mod.rs |D2585C294C1E9B016A9A5ADEEDEBDA338E64896D2BD214985788BDDC2C59F995|
| src/infrastructure/frame_capture/seek.rs |7EB7C3F536C168ED09B4C85F0BE4B846E6F6A746893FE833F00DA5EBF186B694|

다음은기존단일Sprite후보의content/custody와boundedstream/framePTS관측계약·자원배정이다. 기존PLAN의PCM17초/VTT시각을영상PTS로선언하지않으며actual9의새원영상/registry를만들지않는다. 새관측을수행하기전범위/도구pin·원점/priming/실seek근거와품질인수를구분한다.
