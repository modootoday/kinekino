---
id: sot:kinekino/card-frame-free-rendering
title: 프레임 없는 카드의 공통 픽셀 렌더
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 프레임 없는 카드의 공통 픽셀 렌더

R117 [실제 renderer 검증](../../../verification/application/card-frame-free-renderer-20261028.md): 공통26시험·새프레임없는 실제CPU1·기존프레임 실제CPU1·native all-target strict/fmt 통과. 사후custody 진행 중이며 제품 빈템플릿/Jobs/SDK·CLI 연결은 후속이다. 아래 미실행은 당시 기록이다.

R116 구현 중. 기존 프레임 필수 RenderRequest/TextRenderRequest와 결과를 유지하고,OptionalFrameRenderRequest/RenderedOptionalFrameCard 및 render_optional_frame을 추가한다. 기존 API는 새 공통 엔진의 Some(frame) 경로로 연결하며 기존 식별자를 바꾸지 않는다. 프레임 없음은 frame=None이며 결과 frame_sha256/frame_size=None이다. 가짜 이미지/해시/투명 PNG를 생산하지 않는다.

프레임이 없으면 image descriptor·PNG검사·전달 관측을 요구하지 않고,img 요소도 생성하지 않는다. 공통 폰트의 라이선스/해시,텍스트 크기·제어문자·레이아웃 overflow,RGBA/인코딩/출력PNG 검사와 한계 고지는 그대로 유지한다. 현재 CardScene의 frame_url 빈 값은 명시적 이미지 부재이며 외부 URL fallback이 아니다.

이 단계는 공통 renderer 기반이다. 제품의 빈 템플릿 catalog/admission·선택 프레임 optional 준비·Jobs bundle/manifest·SDK/CLI 실제 연결은 아직 남는다. 전체 카드 지원으로 계상하지 않는다. 최초 소스7파일을 preimage-r115-optional-frame에 보존했다.2065fmt종료0·2066input-r116 snapshot 진행 중. 정적 점검에서 새 실제native 시험의 decode_rgba 반환형은 Vec인데 tuple로 읽은 부분을 발견했다. snapshot 완료 후 원 입력을 보존하고 수정할 예정이며 컴파일 실패로 계상하지 않는다. 아직 컴파일/실제시험/strict/회귀/bins/custody 미완료,최신 완료gate R115.

원9 NeedsRepair/사용자승인0/9,소유권/WIP/private overlay·WARP/추론/다운로드/수동GUI/push보류 유지. 기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 stub와 실제 CPU 렌더를 구분한다.
