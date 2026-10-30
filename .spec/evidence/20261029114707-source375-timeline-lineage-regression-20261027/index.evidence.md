---
id: evidence:kinekino/20261029114707
title: PCM 계보 producer 후계 통합
status: active
created: 2026-10-29
updated: 2026-10-29
---
# PCM 계보 producer 후계 통합

2026-10-27 중앙 PM. [실제 원본·재생·클립 계보](timeline-pcm-lineage-20261027.md)와 같은 SOURCE `B1B46C92308C404C7756E67B45F76ABEDCA92C36780AA99FAA66F6738D1353E2`/2208파일을 고정해 소스 편집 없이 직렬 검증했다. inventory `5F4DE664EFD598635FB882E18243F7D32B8DB296385DA5CEA61515EFDC4ECAF6`. 원375 private overlay·이전 C27/CAFF/C06 및 canonical 승격과 구분한다.

| 로그 | 결과 |
|---|---|
|461/462/464/465|strict·정책21/5ignored·실제 no-run·실제1(영상/clock/원PCM/재생/클립/취소) 통과|
|466/467|native-proposal-window lib/jobs no-run177460ms·Product145/145 통과|
|468/469|독립 Meari lib/range_commit no-run·Core18/18 통과|
|470/471|Native window 관련5target no-run·26/26 통과|
|472|자동OS1/1 통과|
|473|정상 player/player-cli build exit0·두 artifact profile.test=false/fresh=true|
|474|저장/replay/CAS·combined·readonly CLI3/3 통과,시험24.75초|
|475|fmt --check 통과|
|476|verify-current exit0/39832ms·전후2208파일 content/identity 동일·stage 없음|

네 group-return은 `runtime/verification/source375-timeline-lineage-{product,core,native,os}-exact-r1`에 같은 SOURCE SHA·execution_result=Ok(())·producer_integrity=Ok(())를 기록했다. frozen exact190 SHA082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464의 실제 등록/ignored·원 EXE/PDB를 대조하고 시험별 원 process를 실행했다. 로그는 source375-datalab-b0-r1-logs 각 번호다.

473 정상 player SHA256 `3BC4B2E8A38CC02256226E7EDB346BB0A6629ECAD166275459DAD8533FC8DFF1`, CLI `DFB31240C3F174BB0D8367D774C07498783149FCF82C829525543007F3EFBCFE`. 이후 default-feature CLI 시험 산출물과 별개다. 실행 파일 생성과 자동OS1은 수동 GUI/시각 인수가 아니다.

root HEADd13cbfca140e41863b4d0bd27930be3f57a1123f/Meari0844f6c24caf224617ca8abbb2dd75695a971b99, 두 lock B2E3D540/23792484, custody78DD39F4의 전체 hash가 기존 pin과 일치했다. C native/meari-native/tmp·locked/offline/jobs1·CPU2/BelowNormal·무거운 작업1개를 유지했다. 소유권·WIP·원 증거/실패·보류는 보존했다.

origin/끝점 capture·결합 proof/custody/admission·제품 CLI startup·전체CLI 인수·실제9 AI품질/GUI 시각은 미완료다. 최신190은 등록된 회귀 집합의 결과이며 전체 제품 완료율이 아니다. QA fixture/code-change 및 postcondition 스킬 적용 출처·판단은 링크한 실제 기능 문서에 유지한다. 수동GUI·추론·다운로드·WARP·push 없음.
