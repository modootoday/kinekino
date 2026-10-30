---
id: adr:kinekino/20261029113536
title: 자체 개발 언어 Rust 통일
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 자체 개발 언어 Rust 통일

사용자 최상위 지시로 제품·새 테스트·벤치마크·개발 도구의 자체 코드를 Rust로 통일한다. SRP와 domain/application/infrastructure/presentation 계층은 언어 통일과 함께 적용한다. main은 조립만 담당한다.

외부 엔진/OS 도구는 infrastructure의 고정 인수·출처/해시 검사 어댑터로 호출한다. 외부 엔진 구현 언어를 자체 비 Rust 개발의 근거로 삼지 않는다. 웹 데이터는 실행 인수를 임의 확장할 수 없다.

새 검증 경로는 tools/validation의 Rust CLI다. 이전 Python/PowerShell 검증 도구와 WebView 주입 JavaScript는 기존 부채로 명시하고 대체 검증 후 이관·최소화한다. 언어 통일 선언이 기존 코드 전체의 이관 완료를 뜻하지 않는다. 자체 Python AI 벤치 코드는 제거했고 Python 추론은 실행하지 않았다.

루트 및 독립 Rust 도구에서 fmt/test/clippy를 모두 수행한다. 후속 작업은 기존 비 Rust 목록 작성, Rust 대체 도구와 동일 시나리오 검증, 사용 경로 교체, 잔여 파일 정리 순서다.
