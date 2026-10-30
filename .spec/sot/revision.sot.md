---
id: sot:kinekino/revision
title: 숏폼 편집 revision과 불변 프로젝트 발행
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 편집 revision과 불변 프로젝트 발행

R145 현재 구현: native ClipProjectFiles와 optional PlayerSdk store·typed ApplicationClient 3메서드·clip_projects.create/inspect/save_revision dispatch/schema/capabilities 및 local --clip-project-root를 연결했다. 실제21CLI/3typed동등·발행head CAS·재열기·과거view·원발행보존을 검증했다. endpoint/serve host와 root를 혼합하지 않으며 timeout/idempotency/자동재시도·렌더·media provenance/품질승인은 지원하지 않는다. 기존 source 선언은 그대로 고정하고 실제 source admission은 후속 preflight 책임이다. [검증 근거](../../../verification/application/clip-project-sdk-cli-20261028.md). 아래 R141 구현 중 표시는 역사 기록이다.

R141입력고정:2294snapshot종료0,2392파일/inventoryC3810CDA8C5A93A661D96DB41F4415C4DEBFAA3B614390CE3CCBC8C54B198F7E,SOURCE.mdABCC35D7F067865571D36E757954A995A6E0046AA85C0A310B27065B1749DF05.2295기본lib/Jobs/CLI no-run진행중이며새시험/strict/후속gate는미완료다.

R141 구현 중이다. ClipEditor는[ClipDocument](clip-document.md)의source배열/clock/관측핀과document ID를고정하고segments/preset/cut mode만편집한다. add/duplicate/delete/replace/reorder/preset/cut mode변경은현재edit revision CAS를통과한사본을검증한뒤한번에적용한다. 빈최종목록/중복ID/foreign참조/잘못된구간/stale CAS/overflow는원문서·undo이력을변경하지않는다. undo는세션내최대64개이며복구할때도수정번호는증가한다. 저장된문서를다시열어세션undo를꾸며내지않는다. 실제다중클립렌더지원은아직없다.

application/clip_projects는create/inspect/save와ClipProjectStore포트·ClipPublication을정의한다. 최초발행은edit revision1/publication revision1/parent없음이며이후발행은원본선언/ID동일·edit revision전진·publication counter연속·이전발행SHA를parent로보존한다. 편집여러회를한번에저장할수있어두revision은동일하지않다. source/clock선언을갱신하려면명시적인다른프로젝트/fork계약이필요하며조용히최신원본으로바꾸지않는다.

native store는원writer lease아래전체이력과head를검증하고expected publication revision CAS·prepare_publication·create-only 발행을이어야한다. application은잘못된header/hash·과거revision view를head로사용하는것을거절하지만이값검증이파일origin/custody를만들지는않는다. 실제native 저장소·SDK/CLI등록·source preflight·encoder·작업발행은후속구현이다. production.render_short는계속미지원이며이미지를생성하거나CLI성공으로계상하지않는다.

preimage-r140-clip-editor에기존domain mod/tests와application mod보존.2293fmt종료0/2294snapshot진행중. 새editor4시험과publication3시험은합성값의원자성/CAS/undo/원본핀/발행이력을검사하며실제파일재열기인수가아니다. 기존modootoday/software-qa/code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B),test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)을적용해포트계약과실제저장·렌더를구분한다. 최신완료전체gateR138/AI품질0/9·기존WIP/소유권/private overlay/보류유지.
