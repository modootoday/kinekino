---
id: evidence:kinekino/20261029114437
title: F1 association / draft application checkpoint
status: active
created: 2026-10-29
updated: 2026-10-29
---
# F1 association / draft application checkpoint

2026-10-26. 이전도메인checkpoint와canonical8A 통합190/bins를보존한추가구현이다. 원Media seal-r2 B47E9913…/mock9/actual9원본·decisions·모델은수정하지않았다. SDK/CLI명령등록·실디스크·사용자품질완료로계상하지않는다.

## 구현

application/review_source에MutationRequest/단일TransactionBody/Receipt,CaseLease·MediaLease·Control·CaseTransaction ports,mutate/inspect_association/read_draft를추가했다. application/mod.rs에는모듈등록1행만추가해기존WIP/route_media를보존했다. domain AssociationRevision에는계획create/revise의note보존을추가하고8192UTF8byte한도를적용했다. draft shape검증은공통domain메서드로옮겨application에서정책을중복하지않는다.

입력·next revision검증→검증caselease 유지→case단위write예약→committed exact replay→control→현재association/draft CAS·exact binding검증→media lease→단일body·출판직전guard→출판/receipt순서다. replay는현재media unavailable/후속cancel보다우선하지만case권한은건너뛰지않는다. draft는명시된과거association revision으로읽으며현재media부재로숨기지않는다. read값으로현재media/PTS가검증됐다고주장하지않는다.

동일case/operation/object/key의다른request는RevisionConflict다. 별도association create/revise와combined create+draft 모두단일body로표현한다. writer port는예약동안최신revision/출판을직렬화하고출판직전PublicationGuard를실제원자경계에서호출해야한다. 출판후에는취소check로성공을Cancelled로바꾸지않는다. 새출판receipt가body와다르거나committed receipt형태가손상됐으면EngineFailed/details.delivery=execution_unknown/publication_may_exist=true/automatic_retry=false를보존한다. 같은key조회로복구하며새key자동재시도0이다.

현재port는실제파일어댑터가아니다. Windowsdirectory/fileguard와writer예약의실물보장,정확body SHA와파일복구는아래다음단계다. MediaResolver도기존custodyadapter에연결되기전제품지원으로등록하지않는다. Verified proof/PTS와SDK cancellation wiring도전체목표에남아있다.

## 실제 검증

- 95 review_source 범위:domain5+application4=9pass/1412filtered,exit0/88929ms.96fmt/97strict exit0.
- 이후committed손상receipt가호출자InvalidArgument로오인되지않도록검사하고실제service경로의손상replay회귀를추가했다.98동일범위최종10pass/0fail/1412filtered,exit0/64793ms.99fmt exit0/100root·paint·native all-targets/native-proposal-window strictClippy exit0/18327ms.
- fixture는매시험독립Arc/Mutex상태이며case/medialeaseDrop count,출판직전취소fault,ACK유실·손상응답fault를명시제어한다. 재연결후과거draft,combined단일commit,미디어부재/cancel후same-key replay,권한재검증,changed-payload conflict,early/latecancel no-commit,postpublish unknown/같은key복구를확인했다. 실제concurrent filesystem/프로세스중단·재시작·디스크원자성/NTFS전원손실보장은입증하지않는다.
- C/native·C/tmp·CPU2/BelowNormal/jobs1·locked/offline·기존toolchain/두lock불변이다. 기존paint deprecationwarning은원로그로보존했다. 신규의존성/다운로드/추론/GUI/stage/commit/push0.

## checkpoint SHA256

| 파일 | SHA256 |
|---|---|
| application/mod.rs |CA93866D660E9782B87308301E0451170ADC53DCBCA77930F9860E494092A6D8|
| application/review_source/mod.rs |478A6A91134A915B9EF76E68061D5B7E4A6C00B11DCE20F75F20F8690560BE7C|
| application/review_source/ports.rs |5FF4736E6E1DFC2CA1B7894186C6B59B1F7CC2448DA0E3908CDC5CEE8AA0A0AA|
| application/review_source/requests.rs |771CCB59D33AB01E29B217B8E4EDE73C9E4445D9AF2A7C83012F8D94CCDBBCDC|
| application/review_source/service.rs |481CF6E9A953FAF1ED1C7FA20F6B8D605276DD31408288A8FF7464038AC3665F|
| application/review_source/tests.rs |814ED0014321936BC248DBF1C4310BF9B2B3F98AC01593291DEDCA4A6EF4E886|
| domain/review_source/association.rs |75AC8F280B379D2ABF74143ABADA9275BEBBFD8CDE432438BDD555F1E7EDD628|
| domain/review_source/draft.rs |F968002A0D65AA340A5E88A7EFAFCB81933860B131BAAFDA5E2D91F2D5F6C18C|
| domain/review_source/tests.rs |F2FB4AA83D322A80FCE5EEB5541BE60569A9F0EC7D0E9E21441FA1DF9B7BC7AC|

다음은existing ReviewStore.open의검증된package/item/source membership+sessionguard를CaseResolver에연결하고기존custody lease를MediaResolver에연결한다. 저장은기존immutable_storage_paths의root/directory/file/lock와기존new_file의단일파일publication을재사용한다. 현재publish_before는deadline검사만하므로출판직전lease/cancel callback의checked publication경계가추가로필요하다. generichelper를multi-filetransaction으로부르지않고단일완료envelope를권위로삼는다. 실제파일·원예약충돌·중단/손상·reopen/same-key replay·읽기전용·cancel·ACK유실을RD01~RD12에맞춰시험한뒤API/schema/capability/CLI를같이등록한다. 실제AI/GUI시각/사용자품질목표와보류는유지한다.

스킬은같은goal에서직접읽은modootoday/software-qa/test-fixture-and-mock-build SHAa393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100을적용했다. 계약의정의된상태/형태·명시fault·격리·release확인과mock/실어댑터보장분리를유지한다. fixture성능개선이나측정하지않은가속주장0이다.
