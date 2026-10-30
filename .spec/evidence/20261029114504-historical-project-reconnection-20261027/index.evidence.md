---
id: evidence:kinekino/20261029114504
title: historical 프로젝트 locator 재연결 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# historical 프로젝트 locator 재연결 — 2026-10-27

공통 SDK의 [locator 재연결](project-locator-reconnection-20261027.md)이 historical evidence 검증을 우회하지 않는지 실제9의 기존 오프라인 표본으로 확인했다. 새 모델·미디어 다운로드/추론·수동 GUI는 실행하지 않았다. 원 후보9 NeedsRepair·사용자 품질 승인0/9·private overlay·소유권/WIP/보류를 유지한다.

## 실제 근거

763 전후 검증은 exit0/42287ms로 종료했다. 전체2244 입력 content/identity 동일을 확인했으며 아래 진행 중 표기는 종료 전 이력이다.

762 입력 고정은 exit0/42614ms였다. SOURCE.md SHA256=8ACEEE6FFAD5685375EFB4CD3C9CDC9AEDDCB2CBB83CAC0D43D035AFC01329E2, inventory SHA256=E6DA0CC0E688BEC50E55E72B228AB32BF4FBF008B4AD96D4B1375E798003A362, files2244. `runtime/verification/source375-historical-reconnect-input-r1/`에 보존했다. 763 전후 동일 확인은 진행 중이며 아래 snapshot 진행 중 문장은 고정 전 기록이다.

제품 코드는 유지하고 `tests/historical_provenance_cli.rs`의 명시 ignored 실제9 시험을 확장했다. preimage는 `runtime/verification/historical-reconnect-preimage-r1/historical_provenance_cli.rs`, SHA256=3B4E8F0765E3227DA7E280EE68EBC3D7454BE9D00FFCDF8AA23EDEB36E084F7F다. 새 시험 SHA256=01F32FCB5E6EB1C80A80F7DBAA64A46309CA499B41D8E679A2C24ED06A0B2C71.

로그는 `runtime/verification/source375-datalab-b0-r1-logs/758~762-historical-reconnect-*`, 실제 receipt103개와 격리 복사본은 `runtime/verification/actual9-historical-reconnect-cli-r1/`에 있다. 758 no-run exit0, 759 actual1/1·CLI103회 exit0/22350ms, 760 해당 Native test strict exit0, 761 cargo fmt --check exit0다. 원 inputs 전후 SHA 확인은 시험 끝의 assert_original_inputs_unchanged로 수행했다. 새 시험에 첫 실행 실패는 없으며 앞 checkpoint의 실패 기록은 보존했다.

input-pins.json SHA256=C99035C1489DFA497BDC9E10BBBA15A58CF92099F269C689C4CEFCEAFBC0A24B, 실제 정상CLI SHA256=1DF8ADD0F6210B4E7C38CEC46943D0EA6647530086902DCB752752D4C735378D. 기존 verifier SHA256=E00C1726603EF7D0167C6D58DEB79E694071044C20A2723DE8ABE8A504F45E41을 fixture가 확인했으며 새 호스트에서 재컴파일한 verifier로 가장하지 않는다.

## 확인한 경계

- 실제 Gemma270 correction historical capture를 포함한 별도 project를 만들었다. 검증기 없는 reconnect는 Unsupported였다. 원 revision1은 남고 verifier를 공급한 정상 reconnect는 revision2로 저장·fresh inspect했다. 제안 snapshot 전체와 과거 revision1은 원 상태와 같았다.
- companion/source/candidate 각각을 격리 복사본에서 누락 또는 변조한 6상태에서 reconnect를 실행해 모두 EngineFailed로 거절했다. 각 원 상태 복구 후 proposal/project 전체 publication 바이트 inventory가 이전과 동일함을 확인했다. 원본 증거를 직접 변조하지 않았다.
- 정상 결과의 source_opened/content_identity_verified/playback_rebound=false를 확인했다. Sintel 영상 실체를 열거나 source 권한·PTS·AI 사용자 품질을 부여하지 않는다. 기술적 수정/채택 fixture는 원 모델 품질 수선이 아니다.

modoo.today `modootoday/software-qa/test-fixture-and-mock-build` 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 원본/격리 입력·실제 호출/합성 해석 분리를 적용했다. 전체190·정상bins·all-targets strict는 SOURCE268E3482의 원 실행으로 유지하며 이번 시험 변경 뒤 재실행으로 계상하지 않는다. watching 부재·episode 정상·전체 CLI 행렬과 GUI/AI 최종 인수는 남는다. 762 새 입력 snapshot은 진행 중이다.
