---
id: evidence:kinekino/20261029114807
title: 자산 출력 manifest와 원 보유자 — 2026-10-29
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자산 출력 manifest와 원 보유자 — 2026-10-29

[출력 계약](../../player/application/production/asset-bundle-output.md)의 private overlay 구현이다. 기존 source375·소유권/WIP·원 증거와 WARP/추론/다운로드/수동GUI/push 보류를 유지한다. AGENTS SHA `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C` 재확인. 새 코드/codec/시험은 Rust이며 의존성 변경 없음.

## 입력과 실패

preimage-r223-asset-output, preimage-r224-owned-output, preimage-r225-codec-lint에 변경 전 파일을 보존했다. R224는 출력/plan2의 fmt2973/2974 및 snapshot2975만 수행했다. R225는 borrow 결과를 원 Arc 보유자 기반 owned 값으로 전환한 별도 입력이며2976fmt/2977snapshot 후2978native strict가 Clippy chunks_exact_to_as_chunks/manual_checked_ops 두 건으로 실패했다. 실제 기능 시험은 그 입력에서 실행하지 않았다. 원 실패를 유지하고 as_chunks_mut/checked_div로 고친 R226을 고정했다.

R226 inventory2489개 SHA `5B65304F6C69BB0E2F8289804413B341EE6EC785B482301E3B4045F674504C8D`, SOURCE.md SHA `E755E41A29398C8DB9D5A145F558898DF7D9427886FC68EF7E48D7A6FDD69667`.2979fmt·2980snapshot·2981native strict·2982default strict·2983lib no-run·2987fmt check 종료0. 원 lib exe SHA `29D57EA053F3070336B00AC9D6DCA9A4030C8C615E98C29743EE9B70B643D9A5`는 fixed-r226/native-core에 보존했고 cache PDB는 유지했다. 기존 Meari deprecation 경고는 원 소유 범위로 보존했다.

## 실제 결과

2984 codec2통과: synthetic2pixel premultiplied raw 보존과 실제 straight PNG 픽셀, Gray8 mask 무 sRGB/gAMA, 잘못된 길이·encode 전 취소를 확인했다. 색/사용자 품질 인수가 아니다.

2985 기존 PNG application1통과: root `runtime/verification/source-sdk-r226-asset-output-r1`. 기존321×137 static source/mask와 고정 R215 request SHA `5411b336f1f3d3a58589183d53acb19a3c3a46ea4be4937e994566eba2477a80`를 실제 admission/raster/encoder/builder로 소비했다. foreground straight와 background premultiplied의 RAW/PNG/mask 및 project/manifest 총8파일을 create_new·write/sync·readback hash/byte 비교했다. manifest SHA `3cae1115fd8292ff4437d7c500df7fb6c65d90dc4d6faf0401a7ffc85a9a2f8c`, project SHA `d0f9c1b3c68347324698af7360ed2d1d821693452a9ddef914081a987a9e655a`. plan/2에 missing policy reject를 명시했다. 실제 파일 내용은 품질 승인 없이 review_artifact/colorimetry_verified=false다. 원 PTS는 null이다.

같은 시험에서 순서1을 먼저 push하면 SourceExpired였고 정상0→1은 성공했다. original catalog와 외부 CapturedAssetProject를 모두 해제한 뒤에도 PreparedAssetBundleOutput이 원 project/image/mask 쓰기를 거절하게 유지했으며 결과 해제 후 쓰기 열기가 허용됐다. 원 source SHA도 보존됐다. concrete 결과의 Send+'static 경계를 컴파일로 확인했으며 실제 Jobs worker 이동/게시를 했다는 뜻은 아니다.

2986 project 저장/typed SDK/capture13통과·기존PNG2ignored. 여기에는 incomplete builder의 finish 거절과 explicit gap policy 보존이 포함된다. codec2 및 명시 PNG1과 이13개를 전체 제작 인수로 합산하지 않는다.

## 남은 범위

2988 사후 custody 종료0: R226 실제2489개 source content/identity unchanged. 원 source375/이전 성공·첫 실패와 private overlay 경계를 그대로 유지한다.

이 파일 작성은 새 evidence 디렉터리의 시험이며 native staging/원 Jobs publication/CAS/완료 proof가 아니다. 단일 capture 공유와 별개 capture 결과의 혼합·512MiB/4096unit 실경계·PNG 작업 중 취소·각 발행/복구/ACK/release fault는 미검증이다. 원 source 파일 묶음·자급 재편집·sequence/영상 admission·마스크 편집/undo·projection·실제 SDKCLI 렌더·GUI·AI 사용자 품질도 남는다. 최신 지정190/정상 bins 통합 기준 R221을 이 구성요소 결과로 바꾸지 않는다. 원 Jobs capture/owned session과 디스크 발행 연결을 다음 구현으로 유지한다.

적용한 기존 QA fixture/변경 검증/verdict 및 image-postprocess의 원본 불변/출력 재검증 출처·SHA/판단은 [원 입력 검증](asset-input-admission-20261029.md)과 공통 계약에 연결한다.
