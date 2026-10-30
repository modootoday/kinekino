---
id: evidence:kinekino/20261029113645
title: 현재 일치하는 외부 실행 파일 보존
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 현재 일치하는 외부 실행 파일 보존

2026-10-18 root. [독립 감사](provenance-audit.md)에서 `limited-live-binary`였던 실행 파일3개를 별도 로컬 evidence에 복사했다. 모델 추론·재빌드·원본 변경은 하지 않았다. [manifest](../../../evidence/ai/live-engine-binaries-2026-10-18/manifest.json)에 크기·SHA·과거 측정 label을 연결한다.

| 실행 파일 | 크기 bytes | 연결 측정 | SHA256 |
|---|---:|---:|---|
| bounded Whisper |11,236,352|6|e980948855b7a552f85a6f15690486060c9e5a07cd3d5fe59e3c915d9175720a|
| SAM |8,706,048|2|dfcfff6f7d6d72de6a75277fe5d53eca21b72def3f0407557dd5324767df9d05|
| portable mistralrs |98,571,776|12|59ee8ff009a06cee8c39890cf6fd90d74f4617e46e88ac9b46a28a11538a9d73|

현재 소스와 해당 실행 파일의 대응은 미증명이다. 파일 보관으로 과거 실행 당시 소스를 복구했다고 해석하지 않는다.

이 보존은 실행 파일 byte identity만 강화한다. 당시 전체 소스·native 종속성·컴파일 옵션·가중치/환경 패키지가 복구됐다고 주장하지 않는다. 과거 다른 해시의 Whisper18개와 mutable model-tool historical hash 누락은 그대로 남는다. 별도 Rust 검사기가 사본3개의 크기/SHA와20개 과거 measurement.executable_sha256 일치를 확인하도록 추가했다.

실제 [무결성 검사](../../../evidence/ai/live-engine-integrity-2026-10-18.json)에서3파일·20개 측정 identity가 일치했다. manifest SHA는 `d8992ba1d601ca7083ea505fc4a30c359cf6139f190bc416f5628faec2d84f0a`다. 검사 도구5시험·fmt·Clippy가 통과했다.

## 새 관측 도구 인수와 링크 범위

raw 관측 도구 EXE SHA719dc70921c568afd283413966665508e2e7cfb0d80c47143bd98548182c8b89는 frozen manifest에서 root 실행 전용 경로에 복사해 같은 해시를 확인했다. 아직 실추론은 하지 않았다. 원래 baseline과 raw 관측판의 동일 입력 최종값 bit 비교가 먼저 필요하다.

새 upstream source README의 SERVE_COMPATIBILITY_MATRIX.md 링크는 고정 Git commit7984967에도 대상이 없는 기존 오류였다. 보존 README SHA6bd8c0276062851df9cd93ff99a6b8031a69530f53e2857719b20f57f402bc03와 정확한 경로·대상 한 건만 `preserved_upstream_broken_links`에 분리한다. 원문은 수정하지 않으며 이 링크가 정상이라고 표시하지 않는다. 다른 경로·내용 hash·대상 오류는 계속 실패한다. 프로젝트 정본 링크0과 upstream 보존 오류1을 구별해 보고한다.

후속186문서 검사에서 canonical_broken_links=0, preserved_upstream_broken_links=1을 확인했다. 전체 링크가 모두 정상이라는 뜻은 아니다. 새 분류의 정확한 경로/hash/대상 일치와 다른 사례 거부를 회귀 시험으로 확인했다.
