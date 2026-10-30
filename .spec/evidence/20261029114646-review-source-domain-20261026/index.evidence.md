---
id: evidence:kinekino/20261029114646
title: F1 source association / range draft domain checkpoint
status: active
created: 2026-10-29
updated: 2026-10-29
---
# F1 source association / range draft domain checkpoint

2026-10-26. canonical통합기준은SOURCE.md SHA8A051EB720E5C46BE51D43F385AFC932003E9164E5E3AADE22218BCB8B5A129D의190/bins통과입력이다. 이번변경은그기준에서추가한F1 domain후계이며이전190성공을새후계의전체기능성공으로전용하지않는다.

## 구현 범위

src/domain/review_source/{identity,association,draft,mod,tests}.rs와domain/mod.rs의모듈등록을추가했다. 원Media seal-r2 B47E991360A2CBB95279193A4D7968CA2455E2F759C234569DBA4EFD69B1D858 및격리mock9는수정하지않았다. r2값규칙을책임별모듈로옮기면서승인계약의독립association revision/초안의exact binding/공통DecimalU64/명시User작성자/strict serde에맞췄다.

- association case/id는수명중불변,media재연결은새association revision이다. 이전draft는원association revision에남으며같은draft에서binding을변경할수없다.
- revision0/overflow·stale successor,빈/64개초과range·잘못된구간·검증duration초과·note UTF8 8192byte한도를검사한다. 빈ranges와겹침/순서는보존한다. duration미상성공을검증된재생가능상태로승격하지않는다.
- wire시간/revision은기존DecimalU64십진문자열이다. unknown fields/숫자·음수/float/u64overflow/author=model/임의verified값을거부한다.
- 현재TimelineStatus는r2처럼Unverified만생성가능하고playback_ready=false다. Verified adapter/opaque proof와전이는전체요구에남았고구현완료로표시하지않는다. Unverified record에evidence_refs를주입해도거부한다.

호환범위: r2는비공개미등록checkpoint이므로공개API이관은없다. request_sha256은계약명source_request_sha256으로명시매핑한다. ID는r2범위ASCII alphanumeric/_/-128bytes를새namespace에유지하고기존SDK전역80byte/dot ID검사나Unicode request ID를변경하지않는다. SHA는기존SDK valid_sha256의canonical lowercase규칙을재사용한다. r2의대문자허용helper를공개API규칙으로도입하지않았고원seal은그대로보존한다. 실제membership/custody검사는application adapter가수행해야하며domain문자열검증만으로신뢰하지않는다.

## 실제 검증

92 cargo test -p kinekino-player --lib domain::review_source --features native-proposal-window --locked --offline -j1/Cnative:5pass/0fail/1412filtered,exit0/89760ms.93cargo fmt --check exit0.94 root/paint/native all-targets/native-proposal-window strictClippy exit0/35696ms. 기존paint deprecation warning은별도원stderr로보존한다. CPU2/BelowNormal/heavy1·TMP/TEMP C/tmp·기존toolchain/두lock불변. 신규crate/download/추론/실미디어/GUI/actual9write/stage/commit/push0.

| 파일 | SHA256 |
|---|---|
| src/domain/mod.rs |7CF432C2391BA2F3892BCAB3950F7B7FC7F1A51644A5A143ADB80E8449402197|
| review_source/mod.rs |AD825B7B222D2DAFFD494E155CC8009572E5015F8E5514DCECCD6F544EDECC5A|
| review_source/identity.rs |B7FA2FC070FC7F44E77D531F74209699F444180AEBA1ACBB91EA8B598CCD299F|
| review_source/association.rs |D783A77A6B2398F5CEFDF87BF1042234A5B76E79D8FE8A1F1B3A5004045D2733|
| review_source/draft.rs |0E5290EE776ADD84D24F421E8BE8F92549CD4B6D41C61384425213A9DC6AB644|
| review_source/tests.rs |709705713D7F627EB338BA7D9FFAB1BA7F168DC244B05E521B2F899906DE2C6E|

## 미완료와 다음 연결

이checkpoint는도메인규칙의실제시험이며독립association create/read/revise·draft read/save 명령을등록하지않는다. durable transaction/CAS/replay·실디스크복구·취소·CaseResolver/MediaLease·SDK/CLI/GUI·Verified PTS/실제AI품질은미완료다. 현재normalbins는8A기준이며이번F1기능지원으로보고하지않는다.

다음은기존ReviewStore.open의검증된package/item/source hash와세션guard를CaseResolver에유지하고,기존asset custody의retained lease를쓰는application port·단일immutable transaction envelope·durable adapter를연결한다. 신규media registry나actual9자동변경은하지않는다. RD01~RD12 전체인수는 [승인 계약](../../player/application/api/review-source-draft-contract.md)에남겨순차구현한다.

스킬: modoo.today domain검색의무관한synthetic/agent/frontend/legal결과는채택하지않고software-qa의test-fixture-and-mock-build본문·SKILL.md를읽었다. 정확출처 modootoday/software-qa/test-fixture-and-mock-build, SHAa393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100,부속참조없음. 계약기반fixture/격리와mock대실저장검증분리를F1에적용한다. 이번5시험은순수domain/wire검증이며mock/디스크성공을주장하지않는다. 속도개선작업/비교측정없음.
