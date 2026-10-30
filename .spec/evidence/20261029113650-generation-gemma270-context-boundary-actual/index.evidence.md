---
id: evidence:kinekino/20261029113650
title: Gemma270 전체 context 경계 API 실제 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 전체 context 경계 API 실제 관측

후속 [수정판 동일 입력 재검증](generation-gemma270-context-boundary-fixed.md)은2047→total2048,2048·2049→HTTP400이다. 아래는 수정 전 원본 관측이며 보존한다. 내부trace/full gate와 모델 제작품질 승인은 별도로 남는다.

2026-10-19. [준비 후보](generation-gemma270-boundary-preparation.md)109개를 실제 count하고 정확한2047/2048/2049 입력으로 같은 message의 chat 요청을 수행했다. **프로젝트 전체 sequence<=2048 계약상 초과를 API usage에서 관측했다.** 처리 position 자체나 내부 IDs/forward의 초과를 관측한 결과는 아니다.

| count API 실제 값 | 후보 ID | chat HTTP | 보고 prompt/completion/total | 결과 |
|---:|---:|---:|---|---|
| 2047 | 106 | 200 | 2047/2/2049 | length 종료지만 전체1token 초과 |
| 2048 | 107 | 200 | 2048/1/2049 | 가득 찬 prompt에서도 추가1token |
| 2049 | 108 | 400 | 없음 | invalid_request_error, prompt greater than2048 |

경계 arm은 max2/temp0/seed42/ignore_eos=true/truncate=false이며 품질 arm256과 별도다. count/generation의 저장 messages는 같다. length 문자열로 Length/ModelLength를 구별하지 않았다. 원문 rendered UTF8·U32LE IDs·forward offset/position·상세 stop counter가 없어 full_boundary_gate_passed는 false다. 후보 추가 전 is_done이 현재 길이만 검사하고 이후 add_token이 push하는 source와 결과가 일치한다. upstream 전체 의미를 단정하지 않고 [프로젝트 정책 ADR](../../../adrs/player/adr-generation-context-boundary.md)을 적용한다.

## 자원과 증거

같은 고정253115424B GGUF와 [e862 CLI](generation-gemma270-cli-build.md), BelowNormal/CPU4/prefixcache0/maxseq1/paged off/offline 조건이다. preflight CPU29.98046875%/free35594211328B,113.5291725초/ready5.5529552초, peak private1292873728B/working1528918016B, exit0/timeout=false다. count/hash/기동/생성을 포함하며 반복 성능·노트북 전력 실측이 아니다. 소유 server79960은 종료했고 사용자 앱·VM·DPI는 변경하지 않았다.

[manifest](../../../evidence/ai/model-spikes-2026-10-18/results/gemma270-boundary-api-r1/manifest.json) SHA256은 `23c153d4acc6e23d709470a1125232622c29343554a3653fe8147f8313f78e4b`, [측정](../../../evidence/ai/model-spikes-2026-10-18/results/gemma270-boundary-api-r1.json)은 `8fcbd54f0c5179b8bae3c1897f43c96e13ae140fc5c2a235395a56deacc98fdf`다.109개 요청/응답·개별SHA·선택 후보·3생성 응답을 보존했다. [worker compiler inputs201개/EXE1개](../../../evidence/ai/model-spike-builds/b36e46ef09a822aab2a516823cd31e756fb09692166853e796b5d2aa1d55017a/build-manifest.json)를 선보존했다.

## 수정과 미검증

후보를 더한 길이로 ModelLength를 검사하고 text/multimodal의 가득 찬 prompt를 거절/명시 truncation 분기로 보냈다. embedding의 정확한 최대 입력은 계속 허용하며 generation budget·EOS/stop 우선순위를 유지했다. Sequence fixture2개/admission category1개 회귀를 추가했다.

[수정 snapshot](../../../evidence/ai/engine-context-cap/gemma270-context-boundary-fix-r1/manifest.json) SHA256 `91f1c1148302fc3716c7d5b261a66f44811e1f41827428b5f72d919592f29fbf`, patch `3dd7d4894a958da611519a7548508a113825269f057778bcadeabd864a8f6d99`에 프로젝트11쌍과 설정/license4개를 보존했다. fmt/core check와 전체1374tests/0failed/5ignored, CLI/server all-targets strict clippy가 통과했다. CLI unit을 실행한 결과로 확대하지 않는다. 새 CPU CLI build는 별도 진행 중이다. 수정 EXE의 같은3경계·render/IDs/stop/forward trace는 아직 남았다. 원본 결과를 덮어쓰지 않는다.

[전체 시험 원문](../../../evidence/verification/gemma-context-boundary-fix-2026-10-19-r1/gemma-context-boundary-fix-tests-r1.stdout) SHA256은 `d7e1597d0b70b42bdc88958ac997aee69a047ef7259b3af3c8d12b214ee61152`다. test-profile build13분54초/시험10.44초이며 같은 폴더의 [CLI strict clippy](../../../evidence/verification/gemma-context-boundary-fix-2026-10-19-r1/gemma-context-boundary-fix-clippy-r1.stderr) SHA256은 `a231e2824154e5fab8432779f67322a6d69ece71fcd3dc976f54410e5aa2706f`다. clippy3분05초였다. 원본 EXE와 다른 소스 단계로 분리한다.
