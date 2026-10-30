---
id: evidence:kinekino/20261029114230
title: F1 user-review file transaction adapter
status: active
created: 2026-10-29
updated: 2026-10-29
---
# F1 user-review file transaction adapter

2026-10-26. 전제는 [application 계약 checkpoint](review-source-application-20261026.md)다. 이번 실제 파일 검증과 SDK/CLI 등록·사용자 품질 승인은 별도다. 원 Media seal/actual9/raw/decision·기존 WIP·역사 실패는 변경하지 않는다.

## 구현과 보장 경계

Windows UserReviewFiles는trusted assembly가지정한기존ordinary local root의별도cases namespace만사용한다. caller request에path/URL을받지않고case canonical JSON의SHA로디렉터리를선택한다. 기존immutable_storage_paths의root/directory/file/writer lock를재사용한다. shared helper는원내용으로보존하고이adapter의읽기예약만로컬함수로둔다. readonly open은기존reservation을읽어잠그며디렉터리/기록/lock파일을새로만들지않는다. case writer는OS배타file handle을조회/CAS/출판종료까지유지한다.

출판권위는schema/request/association/draft를함께담은한개의immutable JSON envelope다. compact canonical JSON≤262144bytes, SHA filename/hash/strict decode/shape/case일치를확인하고원파일handle을유지한다. load는filesystem열거순서에의존하지않고association/draft revision chain·중복key/같은revision·과거association snapshot을검증한다. 별도index/receipt파일완료로판정하지않는다.

기존media-file-publication에publish_guarded를추가했다. 기존publish/publish_before의신규파일/overwrite금지/기한동작을보존하면서write+sync뒤create-only hard link직전에원lease/cancel callback을호출한다. 출판후readback/receipt문제는execution_unknown/publication_may_exist를유지한다. publish완료후cancel로바꾸지않는다. crash상태의tmp가이미committed envelope와동일SHA이면비권위alias로보존/허용하고,일치하지않는partial/orphan은NeedsInput으로fail-closed하며원파일을지우거나빈history로되돌리지않는다.

현재자원상한은case당4096 committed transaction/한envelope262144bytes다. 가득차도기존key를만료/삭제하지않고새publication만ResourceExhausted로거절한다. committed exact replay는capacity검사보다먼저다. 추가history/복구정책변경은명시적migration이필요하다. OS전원손실·NTFSdirectory-flush내구성은측정하지않았으며완전보장을주장하지않는다.

## 실제 시험과 수정 이력

- 101공통publication helper:완성된temp bytes를확인한guard에서cancel→output없음/정리후재출판성공/기존output overwrite거절,1pass.106은기존deadline경로의Timeout/nooutput도추가확인해1pass.107공통crate all-targets strict exit0.
- 102 review_source16시험(domain5/application5/실파일6)모두pass였으나shared rooted helper에추가한read_lock이기존path-include fixture2곳에서unused경고를만들었다. 새읽기예약은adapter내부로옮기고shared helper를원bytes로복원했다. 경고억제/기존fixture변경0. 원102로그는보존한다.
- 103최종16pass/0fail/1412filtered,exit0/57425ms. 실파일6은readonly drop/reopen·exact replay/과거draft·실Windowsreader/writer충돌/해제후재접근·CAS/key충돌·latecancel no-publication·ACK유실후새store조회·committed tmp alias허용/orphan보존거절·손상JSON보존을확인했다. 기존models/review package를사용하지않는독립합성root다.
- 104fmt exit0.105root+publication/subtitles/jobs/VAD executor/worker/product plugin/paint/native all-targets strict exit0/24763ms. 마지막test-only deadline회귀추가후106test/107package strict/108fmt 모두exit0. 기존paint deprecation warning은원로그로보존한다.
- CPU2/BelowNormal/jobs1/locked/offline/C native·C tmp·기존toolchain/lock유지. rootindex/stage/commit/push0. 파일fixture는runtime/build-cache/tmp/review-source-files-{22292,24800}-{0..5}로보존됐다. 이는102/103의실제synthetic fixture 집합이며각테스트이름과개별디렉터리매핑을별도로그로직렬화한것은아니다.

## 현재 한계와 다음 작업

같은프로세스에서drop/reopen·OS공유잠금과물리파일은검증했지만실제프로세스강제중단/다른프로세스간경합·디스크부족/권한변경/root교체·전원손실은아직미검증이다. alias/partial상태는실제파일로구성해검사했으며원프로세스의그순간을kill한실측이라고하지않는다. fault lose_ack은test-only이며실제SDK/IPC응답유실과별도다.

CaseResolver/MediaResolver port는아직실제ReviewStore/asset custody에연결하지않았다. 다음은원ReviewSession guard를유지하는case membership bridge와기존custody media lease,프로세스중단/재조회시험,공통DTO/API/schema/capability/CLI등록을연결한다. 해당어댑터전제품명령지원은등록하지않는다. 현재TimelineStatus는Unverified이며Verified/PTS·실AI결과·GUI시각·사용자품질은전체요구에남긴다. 다운로드/추론/수동GUI/WARP/push보류유지.

## 현재 source SHA256

| 파일 | SHA256 |
|---|---|
| packages/media-file-publication/src/lib.rs |66713C79C0D3481C811201A52E8B8550090B81F7B1FD63FD976030F66799D6D8|
| src/infrastructure/mod.rs |2B040FB9CB38DF0CB0FB92D40D9EC98BA762A8AF5C4F4D6381ABB16C01F0D283|
| src/infrastructure/new_file.rs |7E7BB1D1B7749C33B44E0271B1604D0B546E69374104FF2CD8DA5008E7CB6C24|
| review_source/mod.rs |D84027121A52E3876CEFC0AFDD6D49BCED18DE98CF353FEFFF0113F356C92812|
| review_source/records.rs |366A97D7F15A856DD54AB69A0E12C4DE4E292732BE51A98BED639CFD3CC93BBE|
| review_source/transaction.rs |90956343BED983A3A088E2E41D3851BD423FD162125A1E02A0B159488E4E7D0A|
| review_source/tests.rs |0A1BFDA90D98889D590988ABEA346DA340CB1FDE52A0EC0BADB3F9B7F2FF14CC|

스킬: 직접읽은modootoday/software-qa/test-fixture-and-mock-build SHAa393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100. 실제파일fixture격리·원OS예약취득/RAII반환·반환후두번째open성공을적용했다. mock/실파일/프로세스/전원손실의증거범위를구별하며미측정속도개선주장0이다.
