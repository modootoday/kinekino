---
id: evidence:kinekino/20261029114449
title: 카드 묶음 준비와 공통 텍스트 렌더링 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 카드 묶음 준비와 공통 텍스트 렌더링 검증

최종1962 custody 종료0: current source content/identity unchanged files=2349. 고정 R106 입력과 선택 시험 이후 입력이 동일함을 확인했다.

2026-10-28 R105→R106 중앙 PM. [구현 계약](../../player/application/production/card-bundle-preparation.md). CardBundleRequest/PreparedCardBundle·정확한 저장 발행 핀·템플릿/프레임/새출력 port를 추가하고 media-card-render의 픽셀 엔진을 CardText/TextRenderRequest/render_text로 분리했다. 기존 accepted RenderRequest/render/RenderedCard는 원 SelectedCard를 보존한다. 원 root WIP·다른 담당 UI·독립 Meari 소스를 변경하지 않았다.

## 입력과 첫 실패

R105 SOURCE.md SHA256 A8952F2E2D91F2098044523C63C803E32FCFD52EF2117ECCD1FF049CC0E4A7DC, inventory05B34424A86A27C0B126F803162869E21F9CA5C1C80ED30C395D61B591CA7315/2349. 1952 카드 구성 strict는 기존 asset_contract 시험의 CardScene 텍스트 뷰와 SelectedCard 비교 타입 불일치 E0308로 종료1이다. first failure와 preimage-r105-text-port/asset_contract.rs를 보존하고 expected를 텍스트 뷰로 변환했다. front/back 포인터 검사는 유지하며 원 accepted identity는 기존 결과 시험으로 검증한다.

R106 SOURCE.md SHA256 7E245C33453F9EC1FA59567305414F0220E57D17D851AF3153E8CAF70FDEF781, inventory815BF6992D68E5C8361E207D13F893EE4048F338E65E56178F737A4AF60D8209/2349. mirror/runtime/verification/input-r106에 고정했다. 원 renderer source와 product application/mod preimage는 preimage-r104-card-render다. renderer tests.rs 사본은 원 root 파일의 SHA C76A9D4C1E9648999362CB385D6845C6B816EDFBBD349B17C240E550A4224AF4가 R104 inventory와 일치함을 확인하고 복사했다. 사본 FileID가 원 FileID와 같다고 주장하지 않는다.

## 실제 실행

로그 루트 runtime/verification/source375-datalab-b0-r1-logs. 1953fmt0,1954snapshot0,1955 카드 구성 all-target strict0,1956 root/package lib no-run0(1분56초),1957 renderer25통과/기존native opt-in1ignored,1958 bundle prepare4통과/1593filtered,1959 실제 native CPU buffer1통과/25filtered(0.33초),1960 default all-target strict0,1961fmtcheck0. strict/no-run은 -p kinekino-player -p media-card-render, 카드 구성은 kinekino-player/card-review-export 및 media-card-render/native를 명시했다. CPU2/BelowNormal/conservative2/heavy1·C-only/cache/tmp·offline/locked/-j1과 각 원 handle 종료 후 다음 작업 배정을 유지했다. transitive blitz-paint deprecation 경고는 별도다.

고정 exe/PDB는 runtime/build-cache/native/source-sdk-integration-r1/fixed-r106에 있다. root lib kinekino_player-a56df110f8c592d1.exe SHA256 0A1137C694AB2F582EA9827340078A338ED3DF5D1E90E3D597878CDFD467FD94, renderer media_card_render-7a6f693d2513b8a7.exe AE169D8D30D3CA2CCE78E237AA8E3A865CEEB11EFBCF4BB9DBABD3BBF4E9475A.

renderer25는 기존 PNG/자산/오류 cause/텍스트 escaping/레이아웃/제한과 새 neutral text 대 legacy pixel/hash/template/limitations 동등성을 검사했다. native opt-in은 기존 pinned Pretendard 1.3.9 fonts/licenses를 mirror/assets/fonts/pretendard에서 명시 읽어 CPU buffer PNG·8×8 합성 frame/320×320 output·선택 pixel/font delivery/limitations를 검증했다. 기존 accepted-entry fixture가 새 텍스트 픽셀 엔진으로 실제 rasterize됐으며, 이 fixture의 명시 가짜 accepted ID는 실제 store acceptance를 뜻하지 않는다. 새 프로젝트 경로에 가짜 accepted ID를 만든 것은 아니다. 해당 기존 시험은 buffer 관측이며 PNG 파일 발행/제품 CLI/수동 GUI/시각 승인을 입증하지 않는다. 다운로드/설치/모델 추론 없음.

preflight4는 합성 CardProjectStore/CardBundleAssets를 사용했다. 저장 발행 ID/revision/hash와 실제 직렬화 hash 대조, 순서/역할/게시문/alt/원문 보존, 중복/누락/미사용/변경 frame 거절, non-review와 기존 출력 거절, 원 저장소에 publish 호출이 없음을 확인했다. 실제 파일/template/font adapter 검증으로 계상하지 않는다. 30선택시험은 30CLI호출이나 제품 전체 기능 수가 아니다.

## 남은 범위

production.export_cards·실제 template/frame/asset adapter·Jobs·no-overwrite 묶음 발행·실패 부분결과/재시도·취소/정리·manifest/편집 project/문구/alt의 실제 저장은 아직 미연결이다. root StrictPng를 통한 새 제품 renderer 연결과 정상 CLI의 실제 묶음 검증은 후속이다. 빈 템플릿·여러 캔버스·역할별 디자인·glyph/overflow·GUI 시각/사용자 품질 요구를 유지한다. 공통 픽셀 엔진 성공으로 이를 완료 처리하지 않는다.

R106 전체190/정상2bins는 실행하지 않았다. 최신 완료 지정 gate는 R104이며 그 성공을 R106 입력으로 승계하지 않는다. 기본 카탈로그103/별도execution host10과 export_cards 미지원·원 실제9 NeedsRepair/사용자품질0/9 유지. 원 증거/WIP/소유권/private overlay·WARP/추론/다운로드/수동 GUI/push 보류를 유지한다.

적용 스킬: 기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B). typed fixture·실제 CPU raster·원 CLI/전체 gate·사용자 품질의 증명 범위를 분리했다.
