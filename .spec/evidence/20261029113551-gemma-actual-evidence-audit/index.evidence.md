---
id: evidence:kinekino/20261029113551
title: Gemma 실제 결과 저장 근거 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma 실제 결과 저장 근거 검사

2026-10-19 중앙 Rust checker 실제 실행 session27355 exit0을 확인했다. [저장 보고서](../../../evidence/verification/gemma-actual-integrity-2026-10-19-r1/build-evidence.json) SHA256은 `48f8460f8aadbf24dbddd1dbd7545edd2a255b9f2f59b15ae739e14e0062f0d1`, [링크 보고서](../../../evidence/verification/gemma-actual-integrity-2026-10-19-r1/links.json)는 `b6a017e7f006e32e68186a4d394189b73f702a71c4bdad5ca581d5b97910dabf`다. 해당 시점 문서386개, canonical broken links0개, 원본 자료의 보존 미해결 링크3개다. 이후 새 문서와 수정판 엔진은 이 과거 검사로 검증했다고 주장하지 않는다.

원본 Gemma CLI11파일, 경계 실제 count 응답109개와 generation 응답3개·측정, 자막3요청의 manifest·측정·평가3종을 고정 SHA로 검사했다. source boundary 수정11쌍과 설정/라이선스4개, macro fork53파일 및 원본 crate archive도 검사했다. engine-lab 보존 빌드는12개다. 원문 [stdout](../../../evidence/verification/gemma-actual-integrity-2026-10-19-r1/check.stdout)과 [stderr](../../../evidence/verification/gemma-actual-integrity-2026-10-19-r1/check.stderr)를 함께 보존했다.

검사는 저장한 바이트의 일관성이다. 새 추론·모델 품질 승인·원본 프레임워크 수치 대조·내부 token/forward/stop trace·완전한 native 및 전이 의존성 compiler attestation을 수행한 것이 아니다. 수정판 API 재실행과 관련 보존/검사는 후속 게이트다. [원본 실제 경계](generation-gemma270-context-boundary-actual.md), [원본 실제 자막](generation-gemma270-structured-smoke.md), [수정 CLI 보존](generation-gemma270-cli-build.md)을 구분한다.
