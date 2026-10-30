---
id: evidence:kinekino/20261029114441
title: 카드 묶음의 실제 PNG·공통 자산·버퍼 렌더링
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 카드 묶음의 실제 PNG·공통 자산·버퍼 렌더링

최종1973 custody 종료0: current source content/identity unchanged files=2351. 고정 R107 입력과 실제 파일/CLI 시험·정상 bins 이후 입력이 동일하다.

1971 정상 native-proposal-window player/player-cli 명시 build 종료0(1분13초),1972 fmtcheck0. 카드 구성 CLI는 별도 fixed-r107에 보존하고 기본 debug 경로를 정상 구성으로 복원했다. fixed-r107/native-bins의 GUI SHA256 710AC575CE440313EC6489A484FCD99C22F4E24E4478720CEF8F263CEFAF3A38, CLI 2F64FD9196E9A7CB4EA2006AAE5D4EE45C5831EE66717B5C13EA43788D62D194. PDB도 보존했다. R107 전체190은 미실행이며 최신 완료 통합gate R104를 현재 입력의 전체 회귀로 승계하지 않는다.

2026-10-28 R107 중앙 PM. [파일/자산 어댑터 계약](../../player/application/production/card-bundle-file-adapter.md), [준비 계약](../../player/application/production/card-bundle-preparation.md). CardBundleFiles를 실제 CardProjectFiles/PNG·공통 템플릿/폰트에 연결하고 기존 accepted CardReview와 같은 render_content/TextRenderRequest 경로를 사용하게 했다. 프로젝트 픽셀 입력에 fake accepted ID를 만들지 않는다. 아직 Jobs·production.export_cards·묶음 발행은 미연결이다.

R107 SOURCE.md SHA256 AED3B45932DD2D9F3B194678C5E9276809A504529854E20ACF6E02B7C099D01F, inventory3E71C7E7E576D5AB6CE805ED685A47E119A003A5CF04F2BA45C0E1DDEA89E0BA/2351. snapshot은 mirror/runtime/verification/input-r107, 수정 원문은 preimage-r106-card-bundle-files에 보존했다. private mirror의 application card_bundle과 infrastructure card_export만 변경했다. root WIP·독립 Meari·타 UI/미디어 소스·원 실제9를 수정하지 않았다. 원 source375/canonical 채택으로 계상하지 않는다.

로그 루트 runtime/verification/source375-datalab-b0-r1-logs:1963/1964fmt0,1965snapshot0,1966 root/card package의 카드 구성 all-target strict0,1967 root lib/accepted_card_export_cli no-run0(3분02초),1968 실제 파일 adapter4시험 통과/1597filtered(4.77초),1969 기존 accepted 카드 실제CLI1시험 통과/8filtered(5.90초),1970 default all-target strict0. 이번 단계의 첫 실패는 없으며 이전 R105/E0308은 원 기록에 보존한다. transitive blitz-paint deprecated 경고는 별도다. 각 원 handle terminal 이후 다음 무거운 작업을 시작했고 offline/locked/-j1·CPU2/BelowNormal/heavy1·C-only/cache/tmp 유지.

카드 구성 고정 exe/PDB는 runtime/build-cache/native/source-sdk-integration-r1/fixed-r107에 있다. CLI SHA256 B542B25707CB4B62C68A0D4374A0E9283396DF2D4FC4E9D7F234DE7BCB4B68B4, root lib kinekino_player-a56df110f8c592d1.exe 4E10EC353603339AEC9ADE85B3A54235F96B632867D573F6E3F670913FC2E195, accepted_card_export_cli-18b05ec505233866.exe 97E0EB09A836D899C84CBDAED330785EDF6499099417BDDB535D6FF794F26E31.

## 실제 파일 시험 범위

증거는 runtime/verification/source-sdk-r107-card-bundle-files-r1의 bundle-{render,png-errors,template-output,changed}-6772다. render는 실제 새 카드 저장소에 합성 프로젝트를 발행하고 원 저장 상태를 읽어 비교했다. 저장 발행 pin·compiled PNG1 template declaration·Pretendard1.3.9의 실제 공통 자산/라이선스·8×8 frame을 사용해 서로 다른 두 카드의1080×1080 RGBA PNG를 만들었다. 독립 png decoder로 크기/색형식·선택 두 픽셀을 검사했다. PNG와 provenance/prepared JSON을 남겼으며 프로젝트 저장 상태는 불변이고 제품 output_directory는 생성되지 않았다.

observation-0.png SHA256 B8D2158D4B7B043B1D5A923973945A76C94A3A4FF87EFB85D6C66C7743E0762A, observation-1.png BD5F59C7E6D168975C41B0AF4C7F8C6F74711B887E91A539EDA28234CA3BEEF2. 이 파일은 시험의 관측 증거이며 Jobs가 발행한 최종 카드 묶음이 아니다. provenance는 card_project pin/edit revision/card ID/position 및 quality_approved=false를 가진다. accepted target/accepted_output_revision은 생성하지 않았다.

png-errors는 틀린 입력 SHA뿐 아니라 손상된 실제 PNG의 SHA를 다시 고정한 상태에서도 CRC/trailing을 엄격히 거절하는 것을 확인했다. template-output은 잘못된 선언 SHA/다른 canvas·기존 출력 충돌/상대 경로/상위 이동을 거절했다. 충돌 디렉터리는 시험이 명시 생성한 합성 항목이다. changed는 준비된 파일명/폰트 pin 변조·준비 이후 frame 변경을 거절하고 제품 출력이 없음을 확인했다. 변경 전 원 합성 frame도 별도 보존했다. render_card의 재검사는 source snapshot/미디어 권리/PTS 인증이나 사용자 품질 승인이 아니다.

## 기존 accepted Card CLI 회귀

runtime/verification/source-sdk-r107-accepted-card-cli-r1에 실제 새 고정 카드 CLI의 요청/응답/상태와 PNG·저장 전후 증거를 보존했다. 기존 exact accepted Card target/position/template/frame pin·새파일·거절/저장 불변 경계를 다시 실행해 통과했다. review.png SHA256 98F1395550589D060180BE9ED30E841A6371F9A0CF68EE85C64DAF336E2B13AC는 R97 관측과 같지만 R97 성공을 승계한 것이 아니라 R107 새 CLI로 재실행한 결과다. 이는 일반 export_cards·원 실제9·GUI 시각/AI 품질 인수가 아니다.

다음은 기존 Jobs의 prepared UTF8 source blob·원 execution control/검증/발행 gate·PNG/문구/alt/편집 project/manifest 묶음의 no-overwrite publication과 실제 CLI 인수다. 빈 템플릿/여러 canvas/역할별 디자인·glyph/overflow·취소/timeout·disk/permission/crash·부분실패 재시도·GUI/사용자 품질은 미완료다. 기본103/별도host10·export_cards 미지원·원9 NeedsRepair/승인0/9 유지. 원 증거/WIP/소유권/private overlay·WARP/추론/다운로드/수동 GUI/push 보류 유지.

스킬 적용: 기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B). 합성 port 시험에서 실제 파일/PNG/폰트와 실제 CLI로 증거를 보강하되 pixel 관측·품질·발행·전체 gate 범위를 구분했다. 모델 추론/다운로드/설치/수동GUI 실행 없음.
