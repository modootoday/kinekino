---
id: evidence:kinekino/20261029114357
title: 현재 카드 검토 PNG의 CLI 선택 검증 — R97
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 현재 카드 검토 PNG의 CLI 선택 검증 — R97

최종 1900-source-sdk-r97-custody는 종료0, current source content/identity unchanged files=2336이다. R96 입력 그대로이며 R97에서 전체190을 다시 실행하지 않았다.

2026-10-28 중앙 PM. 소스 변경 없이 R96 입력 D9111DC774A224299B111179B63CC0135B0A0F8330F8521DB6AE155C57138F88을 사용했다. card-review-export 구성의 공통 SDK/정상 CLI production.export_card_review_png를 실행했다. 일반 production.export_cards와 구별한다.

로그 루트는 runtime/verification/source375-datalab-b0-r1-logs다. 1896 카드 구성 all-target strict 0, 1897 선택 test no-run 0, 1898-source-sdk-r97-card-actual은 accepted_card_review_cli_publishes_real_png_and_preserves_binding_boundaries 1통과/8filtered, 시험 5.35초다. 1899 native-proposal-window 정상 player/player-cli build는 종료0이다. 원 로그와 fixed-r97 exe/PDB를 보존했다.

고정 카드 CLI SHA256 F5E04D12D41BD20B615640C9EEB8F518C5265A9929121490B3F0C64066CDC487, 시험 exe DCAEF39C5287D124B2E5F8A56856F31AA130CFE893CEDA916D3C7B6DF8B94916. 정상 구성 재빌드 뒤 debug CLI는 B966A8D2E666653BCFD6BA0E25551F8D1FDA635B09500F685A4A14D697300E38로 관측됐다. R92 고정 정상 CLI와 다른 바이트이며 동일하다고 계상하지 않는다. R92 원 고정 산출물과 검증 귀속은 보존한다.

실제 요청·응답·상태·PNG·저장 전후 증거는 runtime/verification/source-sdk-r97-card-review-r1에 있다. 합성 8×8 RGBA 입력으로 1080×1080 PNG 36847바이트를 생성했다. 출력 SHA256 98f1395550589d060180be9ed30e841a6371f9a0cf68ee85c64daf336e2b13ac. 선택 픽셀·여백·문자 영역의 잉크 존재·alpha·원본 참조·accepted target·템플릿/프레임 핀·새 출력/거절 경계를 검사했다. 원 accepted/store의 변경 방지도 검사했다.

결과는 review_artifact이며 quality_approved=false/template_approved=false다. available_unverified를 지원/승인으로 승격하지 않는다. glyph coverage·글자 가독성·overflow·fallback·전체 색/시각 일치·프레임 권한/PTS 인증·취소·deadline·제품 템플릿 승인은 미완료다. 선택 합성 픽셀 관측은 renderer의 일반적 배치 보증을 대체하지 않는다. 실제 AI9·수동 GUI·사용자 품질 검토를 실행하지 않았다. 다운로드/설치 없음. transitive blitz-paint deprecation 경고는 제품 strict 결과와 별도다.

적용 스킬: 기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B). 합성 픽셀·실제 CLI 실행·시각 인수·사용자 품질의 증명 범위를 분리했다. 기존 WIP/소유권/원 실패/private overlay 및 C-only/heavy1 보류를 유지한다.
