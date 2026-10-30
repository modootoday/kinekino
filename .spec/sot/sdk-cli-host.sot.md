---
id: sot:kinekino/sdk-cli-host
title: 일괄 작업의 공통 SDK와 명시 CLI host
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 일괄 작업의 공통 SDK와 명시 CLI host

R134 고정입력:2230snapshot종료0,2382파일/inventory EB075716547B809204526CF95CF8FC2B3F48F72FA201E21162C2B47F1901D43E,SOURCE.md1C3DE68397C432B7DAE8CA83C63BDB52995AD2BCDC6B717B5FDFE633A824A38D.2231기본lib/Jobs/CLI no-run진행중이며실제CLI/typed동등성·strict/190/bins/custody는미완료다.

R134 정적보강: R133 snapshot2382/inventoryF8E42AC38AF0F7B5CBBD0BE48DA2720183D83310370360B4E2BCCBD9AC990CF4,SOURCE.mdD37483F83667D4ACA0A0043891F15C41399CE7C82B7722F09939C1943496D207을보존했다. 컴파일전preimage-r133-batch-history-schema를보존하고이력inspect입력schema가현재host operation목록에제한되지않도록보강했다. create는현재원jobs.submit입력schema를공유한다. unavailable discovery/CAS/schema·과거operation표현회귀시험추가,2228/2229fmt종료0,2230새snapshot진행중. 실제SDK·CLI/후속gate는미완료다.

R133 구현 중이다. 공통 BatchService가 create/inspect/submit/refresh/retry_failed를 소유하고 PlayerSdk 및 ApplicationClient의 typed method가 같은 계약을 사용한다. 명령 이름은 job_batches.create/inspect/submit/refresh/retry_failed다. 기본 discovery는103→108명령이며 backend 미연결 명령은 unsupported다. 기본 CLI에서 실행 기능을 임의로 활성화하지 않는다.

명시 --serve-jobs host는 원 retaining worker를 시작한 동일 --job-root에서 BatchFiles를 열고 같은 jobs client와 BatchService를 조립한다. 저장소 열기가 실패하면 기존 dependency shutdown 경로로 원worker를 정리하며 ready를 내보내지 않는다. batch root/scope를 wire 입력으로 받지 않는다. PlayerSdk.with_jobs로 Jobs를 바꾸면 기존 batch 포트를 해제해 이전조립을 새Jobs에 승계하지 않는다. frame/기타 generic builder는 기존 batch 포트를 보존한다.

생성은1..128개·512KiB·유효/고유item ID를 먼저 검사하고 원Jobs.capture로만 안전한 source pin/정규화 입력을 만든다. create-only batch ID 중복은 RevisionConflict이며 자동 재제출/교체하지 않는다. ACK미관측이면 먼저 원ID를 inspect한다. capture 중 source blob이 생길 수 있고 이후 오류시 보존한다. 생성/열기만으로 원Job을 실행하지 않는다.

변경 명령은 positive expected_revision을 요구하고 idempotency_key/context timeout은 받지 않는다. inspect는선택expected_revision과offset/limit1..8을 사용한다. 반환페이지는768KiB 제한이고 큰단일항목은ResourceExhausted로 거절하며 원journal을 보존한다. 입력·원오류를 잘라서 성공응답으로 만들지 않는다. 생성/변경은compact 영수증·항목상태별수만 반환하고 inspect에서원JobRecord/오류를읽는다. 여러페이지는같은revision으로핀해야한다. 최대이력/최대batch의host지연은아직미검증이다.

schema는원jobs.submit payload/JobRecord를공유하고batch input/progress/CAS/페이지상한을설명한다. 기존discovery시험의명령수기대를108로바꾸며 원preimage-r132-batch-api를보존했다.2226fmt완료/2227snapshot진행중. 아직컴파일·실제CLI·typed동등성·후속gate를통과로계상하지않는다. 최신완료기준선R132/카드actualR127·전체GUI/사용자품질0/9·소유권/WIP/보류유지.

기존modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을적용해discovery변경/실제CLI미검증/원worker근거를구분한다.
