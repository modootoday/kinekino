---
id: evidence:kinekino/20261029114150
title: Case에 결합한 증거 resolver
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Case에 결합한 증거 resolver

2026-10-26 중앙 PM. application/review_source의 ReviewCaseEvidence는 기존 retain_case/ReviewSession을 보유하는 RetainedReviewEvidence를 반환한다. read(reference,bound)는 읽기 전후 case를 확인하고 선택한 item의 명시 asset ID/SHA에 대해서만 기존 binary reader를 호출한다. 해당 handle이 source/package와 모든 asset guard를 소비 완료까지 유지한다. file path/별도media registry를 만들지 않는다.

domain ReviewItem::assets가 source·candidate·baseline·automated check evidence의 목록을 소유한다. ReviewPackage::assets는 이를 모아 기존 전체 목록 의미를 유지한다. 새 case evidence 읽기에서는 package의 다른 item에만 선언된 asset을 거절한다. 기존 media association의 지원 범위를 이 변경으로 새로 정의하지 않는다.

196은 실제filesystem fixture에 두 item을 넣어 검사 중이다. 같은package에서첫item은다른item전용증거를NotFound로거절하고다른itemcase에서는원bytes를읽는다.올바른binary읽기·ID/SHA혼합·path형ID·guard보유와기존저장/복구/capacity영향을함께확인한다.실제미디어/품질인수로계상하지않는다.

## 확정 결과

196 review_source영향25pass/1내부process helper ignored,종료0/시험2.05초.새두item실파일scenario에서foreign item asset거절·소유item읽기·혼합SHA/path형ID거절·원guard유지와기존복구/capacity통과.197 model_review필터의기존presentation검토상태5pass/시험0.74초.이는순수상태회귀이며실GUI/시각검토나domain assets 전체시험분모로확대하지않는다.

fmt --all -- --check 종료0,198 all-targets strict 종료0/31.98초.소스 SHA review_evidence.rs5B188BF3DE581CC0E68F2AB50A59A7D53F6C690D0E0873BB0C27EA624A9A14C1,domain/model_review/package.rsFB309C784A50F1C277A5E858C37F7629EBEC8BC81BF9AC207CFDB6154EF67B11,case_tests.rs23F424DF4A51CBDA2CBB78036E42643BAD4482BF7A7830E5CFEB3968648AC040.시험/strict동안코드변경없음.기존의존deprecation로그는보존했다.

기존case에결합한증거입력resolver는완료했다.전체timeline resolver는이handle을보유한채raw audio/video/PCM·source/tool/process/capture를교차검증해야한다.아직TimelineEvidenceResolver로지원등록하거나Verified를반환하지않는다.396기준190/정상bins는이후계의전체회귀가아니다.

이resolver는기존case의증거입력수명과membership을연결한다.원관측별source/tool/process/capture를교차검증해TimelineObservations를만드는 TimelineEvidenceResolver 전체 구현·Verified·SDKclock은다음범위다.이입력handle을완성된timeline권한으로표시하지않는다.기존WIP·소유권·원증거·heavy1/C캐시·추론/다운로드/수동GUI/WARP/push보류유지.

기존modoo.today test-fixture-and-mock-build/code-change-review-verification의foreign binding·실파일/합성분리를적용했다.[binary reader](review-package-evidence-reader-20261026.md),[기존case](review-source-case-custody-20261026.md),[timeline admission](timeline-evidence-admission-20261026.md)에연결한다.
