---
id: evidence:kinekino/20261029113729
title: JER 참조 실행기 보존 묶음 중앙 무결성 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JER 참조 실행기 보존 묶음 중앙 무결성 검사

2026-10-18. 중앙 Rust 검사기에 JER 참조 실행기 묶음을 추가했다. 실제 보존4개 버전에서 **manifest에 나열된 소스 항목186개와 실행 파일4개가 모두 일치**했다. 소스 수는 버전별 목록의 합계이며 중복 제거한 고유 파일 수가 아니다. 모델 재실행0회이며 JER 수치 대조·모델 품질 승인과 별도다.

## 추가한 검사 범위

기존 검사기는 model-spike 등의 보존 묶음을 이미 검사했지만 `.spec/evidence/player/jer-function-reference-builds/`는 검사하지 않았다. 새 책임 모듈 `application/jer_build_evidence.rs`가 각 manifest의 상대경로·크기·SHA256, canonical 경로의 버전 디렉터리 내부성, 중복 target, 필수 Rust/Cargo/ABI 항목 및 디렉터리 ID와 EXE SHA256 연결을 확인한다. 실행 파일에는 manifest의 기대 크기가 없어 SHA256으로 확인하며 소스 크기 검사와 구분한다.

검사한 버전은 `f6ad6ab3…`, `65b07bb7…`, `db4e7964…`, `79a876e9…`다. 과거 작업본의 현재 소스로 교체하지 않고 각 보존 디렉터리만 읽었다. 선언한 목록의 무결성 검사이며 모든 컴파일 입력의 독립적인 완전성 증명이나 source→EXE 재현 빌드 증명은 아니다. 원본 dscore13파일·사설 CPython/native DLL·모델 가중치는 이 추가 build 검사 범위와 구분한다.

기존 공통 artifact 검사는 파일 전체를 Vec로 읽는 대신64KiB buffer로 streaming SHA256과 실제 bytes를 계산하도록 바꿨다. 전체 검사기 메모리 상한이나 성능 향상을 실측한 결과가 아니라 이 해시 입력 buffer의 범위다. 파일 크기·해시 검증 조건은 유지한다.

## 실제 검사와 부정 테스트

```text
cargo fmt --manifest-path tools/spec-maintenance/Cargo.toml --check
cargo test --manifest-path tools/spec-maintenance/Cargo.toml --locked --offline -j 2
cargo clippy --manifest-path tools/spec-maintenance/Cargo.toml --locked --offline -j 2 --all-targets -- -D warnings
cargo build --manifest-path tools/spec-maintenance/Cargo.toml --release --locked --offline -j 2
tools/spec-maintenance/target/release/player-spec-maintenance.exe check
```

Rust6tests와 clippy/release build를 통과했다. 새 합성 시험은 원본 소스와 같은 길이의 변조, 중복 canonical target, 필수 항목 누락, parent 경로 탈출, 실행 파일 변조를 거부했다. 합성 binary는 실행하지 않았다. 실제 archive 검사는 별도로4/186/4를 확인했다.

첫 실제 중앙 검사는295문서, 정본 링크 오류0, 기존 보존 upstream 링크 문제2를 보고했다. 새 출력 `jer_reference_builds=4 jer_reference_sources=186 jer_reference_binaries=4`가 범위를 드러낸다. 기존 `build_evidence_manifests=2 build_evidence_files=51`은 Qwen ASR 묶음의 카운터이며 전체 build archive 수가 아니다. 다른 기존 범위는 JSON의 각 하위 필드에 있다.

## 보존 근거와 한계

[검사 보고서](../../../evidence/verification/jer-build-integrity-2026-10-18-r1/build-evidence.json)의 SHA256은 `e97eab55bb62c186bc59c5f51447c23f47322d894de666082974901f1de82566`다. 검사에 사용하고 복사한 [검사기 EXE](../../../evidence/verification/jer-build-integrity-2026-10-18-r1/player-spec-maintenance.exe)는 `0258dea80b8f7162260b0a1ef29815d5198c034816d83b98cde1304d949a9475`다. [검사 시점 소스](../../../evidence/verification/jer-build-integrity-2026-10-18-r1/source/src/main.rs)와 Cargo.toml/lock도 함께 보존했다. 이 소스 복사는 검사기 자체에 내장한 컴파일 입력 attestation이 아니다.

해시는 보존 목록의 일관성 근거이며 publisher 인증, native/transitive dependency 전체 고지·패키징, interpreter 실행, 수치 정확도나 모델 의미 품질을 대신하지 않는다. 실제 JER 근거는 [함수 대조](diarization-jer-functions-actual.md)와 [78원소 대조](diarization-jer-matrix-actual.md)를 따른다. 품질 승인0개 및 남은 [품질 게이트](../../../plans/player/ai-quality-gates.md)는 유지한다.
