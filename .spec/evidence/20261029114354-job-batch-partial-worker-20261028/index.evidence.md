---
id: evidence:kinekino/20261029114354
title: 실제 원 worker 일괄 부분 실패 검증 — R132
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 원 worker 일괄 부분 실패 검증 — R132

최종gate:2224정상2bins build종료0(캐시재사용535ms),2225사후custody종료0,2380파일content/identity동일. fixed-r132/native-bins exe/PDB보존,GUI SHAB520A22D88B81EF94C57555913843E2DEED17F8313DCDB2D69D2AE796FF8C3C3,CLI SHA14F0981673EE4139ACA98FB5881A9E1002BAD6F004A8EA170D745D80455533BB. bins는R131과같은bytes이며새컴파일성공으로과장하지않는다. 실제자막원worker부분실패/재열기/실패만retry시험·기본strict/fmt·지정190·정상bins·입력동일까지완료했다. 이는카드일괄실제여정이나전체TI09/SDK·CLI지원의완료가아니다. 다음은host고정root조립과공통SDK·CLI연결이다. 아래진행표기는당시기록이다.

기본gate진행:2219fmt/2220Product145/2221Core18/2222Native26/2223OS1 종료0,190/190통과. 네group-return의실행/producer integrity Ok·같은R132 SOURCE.mdSHA확인. Meari는기존622/624 producer manifest로실제재실행이며새컴파일이아니다.2224정상bins build진행중,사후custody는남는다. 이번변경은시험코드이며제품로직을새로구현한것으로표시하지않는다.

실제시험통과:2216기본no-run종료0(76.1초),fixed-r132/native-core lib SHAFAE88D3A2D97BB932C00294AED42F297191106DE653C0EDCC4933FC0BDCAFFD4/PDB보존.2217부분실패시험1개종료0(0.42초). success Completed/재시도항목 Failed→같은JobID attempt2 Completed,성공JobRecord·출력bytes동일,작업2/실행호출3,첫/최종문서와실행호출기록·native재열기확인. 원host shutdown completed2/cancelled0/interrupted0,error=null,secondary_count0. 증거 runtime/verification/source-sdk-r132-batch-partial-r1. 명시발행전fault이며실제엔진자연발생실패/AI품질시험은아니다.

2218기본strict종료0. 전체190/정상bins/사후custody는아직미완료다. 아래미실행표기는당시기록이다.

입력고정:2215snapshot종료0,2380파일/inventory7940EF856A1E35C8B634F85D0FDE42D79C84C79FAF1F99C3971E833D04F92ED6,SOURCE.mdAD06AC5E6D9059F8D8C5F4DDC4029E2A81EB82224A0260A273EA07097A744E50.2216기본lib/Jobs/CLI no-run진행중. 새부분실패시험·후속gate는아직미완료다.

구현 중이다. preimage-r131-batch-worker에 이전 tests.rs를 보존하고 별도 partial 시험 모듈을 추가했다. 제품 orchestration/store/원Jobs 동작은 변경하지 않았다. 원 retaining worker와 실제 자막capture/변환/발행·native batch journal을 사용한다. 시험 adapter는 한 항목의 첫 실행에만 발행 전 EngineFailed를 주입하고 나머지 호출은 원 자막 executor로 위임한다. 실제 엔진의 자연발생 실패나 AI 모델 시험이 아니다.

검사 대상은 두 원작업 중성공1/실패1→native journal 재열기→실패항목만 원Jobs.retry CAS→동일Job ID/attempt2 완료다. 성공JobRecord/output bytes 보존·원source pin·재제출후작업2개/실행호출3회 유지·최종재열기·원host종료 영수증을 확인한다. 자동실행재개/새key/두번째worker를 만들지 않는다.

2214fmt 종료0,2215snapshot 진행 중. 아직 컴파일/시험 통과로 계상하지 않는다. 실제host/SDK·CLI연결·네이티브재시도 ACK미관측 fault·입력수정fork/cancel/UI·AI품질0/9는남는다. 최신완료기준선 R131/카드actual R127·원WIP/소유권/private overlay/보류 유지.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 명시fault와실제원worker/파일발행·미구현CLI인수를 구분했다.
