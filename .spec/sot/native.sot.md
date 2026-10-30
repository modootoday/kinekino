---
id: sot:kinekino/native
title: 숏폼 프로젝트의 native 불변 저장소
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 프로젝트의 native 불변 저장소

R142입력고정:2302snapshot종료0,2396파일/inventory2EF27835CF985F49C55F2C57C42ECD6879296AAAF18B6B1DF71489DC1C9B6830,SOURCE.md06BA5E45D20A3AAFE3630E5A914E3A7A25C750CAB40F71DEB292CA56460BB8A3.2303기본lib/Jobs/CLI no-run진행중이며새 native 시험·strict/후속 검증은 미완료다.

R142 구현 중이다. ClipProjectFiles는 명시적 local root/ancestor의 원 directory handle을 보유하고 clip-writer.lock을 독점한다. 기존 head 검증부터 CAS·발행 준비·write/sync·최종 rename까지 같은 writer와 project directory guard를 유지한다. 요청이 파일명·명령·다른 root를 조립하지 않는다. 출처 권위는 실제 source preflight의 책임이며 이 저장소가 미디어 custody/Verified를 만들지는 않는다.

최초 directory는create-only,각발행은r-고정폭revision-SHA.json이다. 완성된ClipPublication을새stage에write/sync한뒤원write+DELETE파일핸들의ReplaceIfExists=false rename으로발행한다. 공유native_immutable_publication모듈은이기술책임만소유하며기존카드/batch발행코드는변경하지않았다. 실패stage는.pending-*로보존하고최신발행으로읽지않는다. 장치전원손실/디스크quota를검증한것으로표시하지않는다.

재열기는모든최종파일의연속revision·canonical bytes/SHA·parent·document/source불변성과application발행계약을검증한다. gap/중복/잘못된최종파일/foreign document ID는NeedsInput으로거절하며부분head를반환하지않는다. 명시historical조회에도현재전체이력을검증하고latest publication번호를함께반환한다. 실패시자동정리/이력삭제/소스교체를하지않는다. 원문서1MiB+header4096bytes,project directory최대4096항목한도를두며pending도항목예산에포함한다. 긴이력성능/최대한도실측은미검증이다.

실제native시험은합성ClipDocument로생성/저장/재열기·현재/과거revision·stale CAS·root이동거절/writer독점·원파일덮어쓰기거절·pending보존·partial/foreign최종파일거절을검사하고원root핸들의FileID를증거로남긴다. 미디어/encoder는실행하지않는다. preimage-r141-native-clip-store에infrastructure mod원본보존,2300/2301fmt종료0/2302snapshot진행중. 아직컴파일/실제시험통과로계상하지않는다. SDK·CLI/source preflight/encoder·실제렌더는남고render_short는미지원이다. 기존소유권/WIP/private overlay/AI품질0/9·보류유지.

기존modootoday/software-qa/code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B),test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)을적용해실제OS파일관측과합성미디어선언을구분한다.

