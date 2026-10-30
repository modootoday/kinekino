---
id: evidence:kinekino/20261029114529
title: 원 자산 입력과 application 수명 — R213~R215
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 자산 입력과 application 수명 — R213~R215

[공통 계약](../../player/application/production/asset-generation.md)의 image/mask/review admission을 domain·application·고정 native PNG catalog에 연결했다. 이 작업은 production.build_assets의 프로젝트/원 Jobs/SDKCLI/발행 또는 GUI 인수 완료가 아니다. 원 Media/Review resolver의 소유권과 미완료를 유지한다.

## 입력·경로·수명

수정 전 domain/app/infrastructure 모듈 및 raster/기존 snapshot 진입점은 mirror `runtime/verification/preimage-r212-asset-inputs/`에 보존했다. lock/dependency를 변경하지 않았다. R2132474파일 inventory SHA256 `3CD96D00A252D88CB48DAE7229EBC45EBCFC3D161882FA2234AD41618CB43185`, SOURCE.md SHA256 `1C8E4627704B65F7A1C10AC708D3A9FADFA9F805B8082D2A1A9B88ADC71C2F1E`.

로그 루트 `runtime/verification/source375-datalab-b0-r1-logs/`,2889/2890fmt·2891snapshot·2892native strict·2893default strict·2894native lib no-run 종료0. 고정 lib exe/PDB는 mirror fixed-r213/native-core, exe SHA256 `590A587166660DACF98173887239B646BB07ACB4C00A79DDEEB732B53C435957`.2895첫 실행은9통과/1실패/기존PNG1 ignored로 실제 종료1이었다. untagged gAMA가 거절되지 않아 원 assert가 실패했다. PNG0.18.1의 실제 decoder raw field는 gama_chunk/chrm_chunk이며 source_gamma/source_chromaticities를 대신 읽은 것이 원인이었다. 원 입력/로그/실행 파일과 preimage-r213-png-profile을 보존하고 assert를 유지한 채 검사 대상을 수정했다.

R2142474파일 inventory SHA256 `A0416E3A1B60AA4CC050128E48DAAF0CDEF919725FBDA4ED5C38AE039E94C065`, SOURCE SHA256 `A00C75E2BC59184D0E50EB73136FF270F3A61AD73D34DB4756A03E166B276E2D`.2896fmt/2897snapshot/2899native strict 종료0이며 실제 시험은 하지 않았다. 원 library가 invalid iCCP payload를 무시하는 경로도 확인해 preimage-r214-png-chunks를 보존하고 원 chunk 존재 검사·추가 negative 시험을 R215로 고정했다. 부분 검사 성공을 전체 시험으로 계상하지 않는다.

R2152474파일 inventory SHA256 `CDBD960D5E0F0E934C5D57EF0EF0E9083F12133DCE7AF98345CF77D77A973877`, SOURCE SHA256 `3D585E157021221549B775104B97A36D53903225733B08339B3B217BFCA70EB0`.2900fmt/2901snapshot/2902native strict/2903default strict/2904native lib no-run/2909fmt check 종료0. 고정 lib exe/PDB는 mirror fixed-r215/native-core, exe SHA256 `1A1D717DFBF29C0DAA8B5F435E2412B872DD80B5982D78474F995B3F6F92976C`. 원 handle로 종료를 확인했다.

2905입력 경계11시험 통과, 별도 기존PNG 시험1은 그 묶음에서 ignored로 유지했다. actual 파일 pin/guard와 RGB tRNS·gray/RGBA alpha·full-frame foreign binding·가짜 SHA/미검토 ack·strict PNG/encoding/profile·최초 취소 오류를 확인했다. 그중 review resolver1은 명시 synthetic 포트 계약이며 실제 사용자 검토가 아니다. PTS>2^53 보존도 값 검증과 native Unsupported 확인이지 실제 영상 관측 인수가 아니다. 신규 invalid iCCP negative는 원 decoder가 무시한 payload라도 원 chunk 존재로 Unsupported가 됨을 검사했다. 단위 파일 fixture TMP는 `runtime/build-cache/tmp/source-sdk-r215-asset-inputs-r1/`로 분리했다.

2906기존 PNG 쌍 실제 application1시험 통과. root `runtime/verification/source-sdk-r215-asset-inputs-r1/`, request/verification·새 rendered.png를 보존했다. 원 R212 foreground PNG SHA `0A787BB7B0D47B8AC2F1B46932A7C6F048084334D0B6719150D8E0B2513AA220`, mask PNG SHA `77793436ACDB3EF415642EE84E0B1FF82E298C22FA1E4FCAEE41C32AA368502C`를321×137 static 입력으로 새로 pin했다. catalog drop 전후 원 source/mask guard, actual mask×source alpha·픽셀 재열기·원 SHA 보존·결과 drop 후 guard 해제를 확인했다. 출력 SHA `B3EEA47C4D9CD25B1D92912DF53019BB9123D2AB0F3032514A46C46E2E77CC1D`. 원1280×544 영상으로의 계보/PTS를 만들어내지 않았고 source_pts=null/unreviewed_mask_warning=true/quality_approved=false/sdk_cli_verified=false/gui_visual_verified=false다.

2907기존 자산 픽셀8·2908기존 snapshot PNG13 회귀도 통과했다.2910사후 custody 원 실행 종료0,2474파일 내용·식별 동일성을 확인했다. 새 고정 exe/PDB 디렉터리에는 복사 전 NTFS 압축을 설정했고 복사 후 파일의 compressed 속성을 확인해 원 cache와 기존 증거를 삭제하지 않았다.

native 입력은 host가 고정한 전체 image/mask pin과 로컬 일반 PNG만 읽는다. mask pin은 전체 source image id/revision/SHA/원본 좌표·clock을 포함하고 실제 PNG bytes SHA와 coverage/alpha encoding을 검사한다. unreviewed acknowledgement는 정확한 mask blob pin을 요구한다. Prepared/RenderedAssetFrame은 원 file/ancestor guard를 포함한 lease를 보유한다. 원 catalog가 drop돼도 결과의 원 guard가 남는다.

PNG static CRC/stream 검사는 기존 snapshot 공통 decoder를 재사용한다. 새 sample adapter는 RGB tRNS를 확장하고16bit를 조용히8bit로 축소하지 않는다. [PNG 공식 규격](https://www.w3.org/TR/png-3/)의 sRGB 우선순위로 sRGB tag를 소비하고, 지원 변환이 없는 ICC/cICP와 untagged gAMA/cHRM을 거절한다. 입력 색 해석을 원 색 품질 승인으로 올리지 않는다. 기존 snapshot decode 동작은 변경하지 않고 내부 static 검사 진입점의 소비 범위를 확장했다.

## 검증 경계

- 생성 PNG fixture의 원 파일 guard/정확한 해시·full frame foreign binding/미검토 ack/CRC/encoding/tRNS/profile/취소 및 output alpha를 위2905 범위로 검사했다.
- synthetic review resolver의 usable·record·mask 검사는 포트 계약 시험이며 실제 user review로 계상하지 않는다. production native review resolver는 Unsupported다.
- 기존 R212 foreground/mask PNG를 새 static 입력으로 소비하는 별도 실제 시험2906을 수행했다. 원1280×544 영상의 PTS를 주장하지 않으며,321×137의 두 PNG 파일에 각각 pin을 부여한다. 원본 PNG와 mask의 보존/lease 수명·공통 application 렌더·픽셀 재열기만 입증한다.

기존 skill의 원본 불변/출력 재검증·QA fixture와 actual/synthetic 구별을 적용하며 정확한 출처·해시는 공통 계약 및 이전 raster 검증에 연결한다. Source/model/frame-observation·authenticated review resolver, 프로젝트/sequence·불변 출력·원 Jobs·SDKCLI·GUI는 남는다. 최신 전체190/정상 bins는 R210이며 제작 전체0/5·AI 사용자 품질0/9·private overlay·기존 보류/소유권을 유지한다.
