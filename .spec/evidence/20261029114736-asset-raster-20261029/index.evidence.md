---
id: evidence:kinekino/20261029114736
title: 2D 자산 픽셀 변환 검증 — R211/R212
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 2D 자산 픽셀 변환 검증 — R211/R212

구현 계약은 [공통 픽셀 변환](../../player/application/production/asset-generation.md)이다. domain 값/geometry와 infrastructure 픽셀 kernel을 추가했다. application 원 입력·mask/review admission·project/sequence·Jobs·SDKCLI·발행·GUI는 미완료이며 production.build_assets 지원으로 계상하지 않는다.

수정 전 domain/infrastructure mod는 mirror `runtime/verification/preimage-r210-asset-raster/`, 첫 lint 수정 전 raster/tests는 `preimage-r211-asset-chunks/`에 보존했다. R2112467파일 inventory SHA256 `EB1658695354A4E65796DBC4FBEE433CB70DE052D4A1DBD86AB302DAD7C3CBF7`, SOURCE.md SHA256 `2F9383DEDBEA0810054FBA1DCB28286E76FE81AF827089A622C1FB02089B1439`. 로그2876/2877fmt·2878snapshot 종료0,2879native strict는 chunks_exact_to_as_chunks lint로 실제 종료1. 고정 크기4의 iterator 두 곳을 as_chunks로 고쳤으며 원 실패/입력을 보존했다. R211을 성공 입력으로 계상하지 않는다.

R2122467파일 inventory SHA256 `61177267CFC3AF5EC8FDAA99C481899468735E6579EFBBCAA552C040230B8681`, SOURCE.md SHA256 `87FFD94EC72E31749C6059BE61ADD8545F2A769A74330B93E408D1F4B0DD3556`. 로그2880fmt/2881snapshot/2882native strict/2883default strict/2884native lib no-run/2887fmt check 종료0. 외부 blitz-paint의 기존 deprecated 경고1은 소유 WIP를 수정하지 않고 구별했다. 고정 lib exe/PDB는 mirror `fixed-r212/native-core/`, exe SHA256 `67D815145F77F305D23A27817FB30BEC73697015FC3E47E0BAB9A46FDC9CFB2D`.

2885픽셀8시험 통과: source alpha·trim offset/UV·바깥 pivot의 정확한 유리수, straight/premultiplied, foreground/background complementary matte, transparent hidden red의 blue edge 오염 방지, linear-light 흑백 축소, source-pixel threshold→feather, identity 경로 전 source alpha, budget/empty/shape 거절, 노출된 각 checkpoint의 최초 오류·원본 불변을 확인했다. 합성 픽셀 시험이며 실제 source/검토/SDKCLI 인수로 올리지 않는다.

2886기존 PNG 실제1시험 통과. root `runtime/verification/source-sdk-r212-asset-png-r1/`.1280×544의 고정 PNG와 명시 synthetic manual rectangle mask에서 새321×137 straight RGBA/mask PNG를 만들었다. 엄격 static PNG 검사·디코드 pixel equality·고정 ffprobe의 PNG codec/dimensions/rgba·gray pix_fmt 및 원본 SHA 불변을 확인했다. foreground SHA256 `0A787BB7B0D47B8AC2F1B46932A7C6F048084334D0B6719150D8E0B2513AA220`, mask SHA256 `77793436ACDB3EF415642EE84E0B1FF82E298C22FA1E4FCAEE41C32AA368502C`. 원 입력 mask pixel SHA·transform·source bounds/UV/정확한 pivot·raw probe와 verification.json을 보존했다. PTS null/declared sRGB·quality_approved=false/sdk_cli_verified=false/gui_visual_verified=false를 유지한다.2888사후 custody 원 실행 종료0,2467파일 내용·식별 동일성을 확인했다.

기존 direct image0.25.10/PNG0.18.1과 lock을 사용하고 새 의존성을 설치하지 않았다. lock SHA256 `6616F5A1688488000D2C6BFECFAFBF931BE5AC38C7778AA288F54ACDEA4D0556`. 현재 actual PNG 시험 입력은 기존 `runtime/ai-spike/assets/frame.png`, SHA256 `691C0C6224FE1AB4BA2E3F38EFAC75761951386E2B30670584791535E2C7B9C7`,1144177바이트다. 별도의 명시 synthetic manual rectangle mask를 사용하며 AI 마스크/검토 승인이 아니다. 원 PNG PTS는 이 시험에서 관측하지 않으므로 null, 색은 declared_srgb_not_verified_colorimetry로 구분한다. 원 Media 자료를 수정하지 않는다.

기존 ffprobe pin을 별도로 검사한 뒤 PNG codec/dimensions/pix_fmt를 원 owned process로 관측하는 시험을 준비했다. 새 추론·모델/미디어 다운로드·수동 GUI/품질 승인으로 확대하지 않는다. 적용 modoo.today skill의 정확한 출처/해시와 판단은 위 구현 계약에 기록했다. 최신 전체190/정상 bins는 R210이고 이 부분 변경의 시험 성공으로 대체하지 않는다. 전체 goal·제작 인수0/5·AI 사용자 품질0/9·기존 보류/소유권을 유지한다.
