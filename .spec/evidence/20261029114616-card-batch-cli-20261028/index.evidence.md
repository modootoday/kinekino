---
id: evidence:kinekino/20261029114616
title: 카드 일괄 CLI의 부분 admission 오류 해소 — R137
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 카드 일괄 CLI의 부분 admission 오류 해소 — R137

최종gate:2265정상2bins build종료0(캐시재사용517ms),2266사후custody종료0,2384파일내용·식별동일. fixed-r137/native-bins exe/PDB보존,GUI SHA149BD7F1B21720562C3E5951D8D9933C321DF7AFED565CA179C7ED18499A2917,CLI SHA73F244180BA698A920F7D3FE6D29926CF584863934CADA087191CD2AD0E5CF0E. 카드일괄실제7CLI·원EOF·양strict/fmt·지정190/정상bins·입력동일까지완료. 기본일괄15CLI의실제근거는R136이며R137에서재실행한것으로계상하지않는다. 전체discovery재검증·카드workerFailed/ACK미관측·기타TI09/제작/GUI·사용자품질0/9는남는다. 아래미완료표기는당시기록이다.

기본gate진행:2258기본no-run/2259기본strict/2260fmt종료0.2261Product145/2262Core18/2263Native26/2264OS1 종료0으로190/190통과. 네group-return실행/producer integrity Ok·동일R137 SOURCE.mdSHA확인. Meari는기존622/624 manifest실제재실행이며새컴파일이아니다.2265정상bins build진행중,사후custody남음. 기존카드19CLI는소스변경전R136근거로구분하며이번R137의새카드일괄7CLI와합쳐재실행한것으로표시하지않는다.

실제통과:2255카드no-run종료0(14.593초),fixed-r137/card Jobs SHA275477D1DA68D43C862B8A0C976F8CA115EF7737EA5D732B63D3CB9F6E15EBEC,CLI SHA7B385AF24CE1D2E7AB109A1470B52801951AB6D02BDDA1A139144C72243AC584/PDB보존.2256실제시험종료0(8.97초),CLI요청7개. 첫카드Completed/두번째SubmitError→충돌파일원bytes보존→동일key Submit해소→최종Completed2·첫JobRecord/manifest/모든파일SHA보존·두번째source pin동일/attempt1·CLI/typed최종조회동일·Job2확인. 원EOF dispatch completed/pipe joined/sdk joined·completed2/cancelled0/interrupted0 확인. 증거source-sdk-r137-card-batch-cli-r1/card-batches.

2257카드strict종료0. workerFailed→attempt2시험으로계상하지않으며실제자막원workerR132와구분한다. 기본gate/전체discovery/사후custody는아직미완료다. 아래미실행은당시기록이다.

입력고정:2254snapshot종료0,2384파일/inventory48DC2A04AA97F47306D5C0015314B3CA3662539A4D72D9089BB6F3C69CD665B5,SOURCE.mdB6924F1EE8137DD9CC7768D7AD4B3C809150173CE60228E63881D3ADEB2933AC.2255card-review-export lib/Jobs/CLI no-run진행중. 실제시험/후속strict·기본gate·사후custody는미완료다.

구현 중이다. 기존 카드 fixture/manifest 검증과 원OwnedHost/요청증거도구를 재사용한다. preimage-r136-card-batch-cli에이전card_bundle.rs를보존하고별도card_bundle/batches.rs시험을추가했다. 제품정책·executor·저장소·선언핀은변경하지않았다.

명시카드host에서두고정입력을capture/create한뒤두번째출력경로에실제충돌파일을생성한다. 원첫카드발행은Completed,두번째prepare는RevisionConflict/SubmitError가되어야한다. SubmitError는unknown admission도가능하므로retry_failed로임의Job을만들지않는다. 충돌파일을증거위치로rename해원bytes를보존한뒤Submit명령으로동일입력/키를재제출해해소한다. 성공한첫JobRecord·manifest/모든산출물SHA·source pin을보존하고최종Job2/두번째attempt1·CLI/typed조회동일·원EOF정상종료를검사한다.

이시험은카드의실제파일충돌/admission실패이며worker Failed→attempt2나모델품질검증이아니다. 원worker Failed→attempt2의실제자막근거는R132에보존한다.2253fmt종료0,2254snapshot진행중이며아직컴파일/실행전이다. 전체discovery회귀·카드worker실패/ACK미관측·전체TI09/나머지제작·GUI·사용자품질0/9는남는다. 최신완료기준선R136·기존WIP/소유권/private overlay/보류유지.

기존modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을적용해실제admission충돌과worker실패·미실행범위를구분한다.
