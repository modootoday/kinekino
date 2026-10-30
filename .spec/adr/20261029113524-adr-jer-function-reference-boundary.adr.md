---
id: adr:kinekino/20261029113524
title: JER 함수 참조와 전체 scorer 실행을 구분
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JER 함수 참조와 전체 scorer 실행을 구분

상태: 추가 검증 경계 채택. Rust C-API 어댑터 구현·런타임 취득·실제 함수 실행은 미완료.

후속 상태: [실제 함수6개 대조](../../sots/verification/ai/diarization-jer-functions-actual.md)를 완료했다. 사설 runtime의 실제 flags/path/DLL 관측과 원본 함수 실행으로 RTTM onset+duration 경계 오류를 찾아 수정했다. 이 추가 경로만 통과했으며 전체CLI와 품질 승인은 여전히 미완료다. 위 문장은 결정 당시의 준비 상태로 보존한다.

## 결정

Windows 전체 dscore CLI는 원문 DER 함수가 Perl script를 직접 실행하는 문제를 유지한다. 원문을 고치거나 shim·새 Python 검증 코드를 작성하지 않는다. 추가 독립 검증은 Rust에서 CPython C-API를 통해 고정 원문 load_rttm/load_uem/turns_to_frames/contingency_matrix/jer를 호출하는 방식으로 구현한다. 새 application/검증 정책과 실행·보존·비교는 모두 Rust에 둔다. Python은 변경 없는 외부 reference runtime이며 제품의 AI 엔진이나 SDK 의존성으로 도입하지 않는다.

이 경로는 원문 framing·NumPy/SciPy JER 함수와의 추가 실제 대조다. 기존 전체 CLI 교차 검증을 완료한 것으로 대체하지 않는다. 함수별 관측과 full CLI/DER/global parity, 모델 품질 승인은 별도 필드로 기록한다. 기존 사전 JER 오차 허용치0.00000051 percentage points를 넓혀 grid 차이를 감추지 않는다.

## 실행 전 조건

- 원문13개 및 실제6개 입력의 [확보 근거](../../sots/verification/ai/diarization-jer-source-inputs.md)를 사용하고 다시 해시 검증한다.
- 사설 런타임과 dependency는 workspace 아래에만 취득한다. 버전·라이선스·원본 archive/wheel·추출 파일·DLL을 고정하고 기존 사용자 Python/앱/환경을 변경하지 않는다.
- [PyO3 설정](https://pyo3.rs/v0.27.2/building-and-distribution)의 PYO3_CONFIG_FILE을 사용해 빌드 중 기본 Python site startup probe도 피한다. 설정은 host ABI와 실제 라이브러리 관측으로 확인한다.
- [CPython3.13 isolated configuration](https://docs.python.org/3.13/c-api/init_config.html)을 Rust 어댑터에서 적용하고 site/user site/environment/current-directory import·bytecode 쓰기를 끈다. 실제 flags·sys.path와 불필요한 초기화 hook 부재를 관측하기 전 사용 가능으로 표시하지 않는다.
- 실제 interpreter 버전·module 원문 경로/hash·numeric frame counts·contingency matrix·JER·원시 stream·60초 process-tree deadline/각1MiB cap을 보존한다. 모델 추론 없이 root 단일 worker로 실행한다.

선언한 config나 합성 시험만으로 격리·호환성을 승인하지 않는다. 원문을 Rust로 재구현한 수학 참조와도 구분하며 실제 외부 함수를 호출하기 전 independent parity는 false다.
