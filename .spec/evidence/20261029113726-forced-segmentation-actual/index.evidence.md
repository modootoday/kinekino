---
id: evidence:kinekino/20261029113726
title: 원본 nagisa·soynlp 실제 분절 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 nagisa·soynlp 실제 분절 대조

2026-10-18. [사설 환경](forced-segmentation-runtime.md)의 원본 tokenizer를 Rust C-API에서 실행했다. 실제 음성3개의 고정 transcript 중 한국어는 기존 Rust 결과와 같았고 일본어2개는 달랐다. 전체 원문을 정리해 이어 붙인 문자열은3개 모두 같았다. **분절 개수가 같아도 prompt가 같다고 판단할 수 없으며 정렬 시간 품질 승인은 아니다.**

## 실제 결과

| 입력 | 원본 tokenizer의 정리 후 단위 | 기존 Rust 단위 | 단위 목록 일치 | 이어 붙인 문자열 일치 |
|---|---:|---:|---|---|
| KO1883 | 19 | 19 | 예 | 예 |
| JA1516 | 28 | 28 | 아니오 | 예 |
| JA1634 | 32 | 31 | 아니오 | 예 |

JA1516은 원본 nagisa가 `立ち振る舞っ`을 한 단위, `ホロ`/`コースト`와 `に`/`つい`/`て`를 각각 나눴다. Rust 기준은 각각 `立ち`/`振る舞っ`, `ホロコースト`, `について`였다. `もってのほか`의 분절도 달랐다. JA1634에서는 원본 `番`/`目`와 Rust `番目`가 달랐다. `7,000`은 원본에서도 정리 후7/0/0/0으로 남았다. 이번 결과를 근거로 숫자 merge를 공식 동작이라고 정정하지 않는다.

기존 baseline 출력3개·원문3개의 SHA를 실행 전에 확인했다. 각각의 raw 단위·Unicode 정리 후 단위·기존 Rust 단위·두 prompt 전체를 [결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-functions-r1/segmentation-result.json)에 보존했다. SHA256은 `b4b09dde514f23903698ec9341b1aefcc6c21e1c779095e95ab447356fbd6089`이다. source text를 합치거나 수정한 새 정답을 만들지 않았다.

## 실행 범위와 한계

원본 `nagisa.tagging(text).words`2회, 원본 `soynlp.tokenizer.LTokenizer`1회를 실행했다. nagisa 기본 모델은 실제 초기화·tagging된 보조 참조 모델이며 제품의 HF 후보 모델군에 새 기본값으로 채택한 것이 아니다. 두 tagging 호출 내부의 개별 신경망 연산 횟수를 별도 계측한 것은 아니다. Qwen 정렬 신경망은 새로 실행하지 않았다.

한국어 scores는 고정 lumos checkout의 사전 SHA256 `9a3d6881b22b976da41359d74dbe4deaae445521997aa05d6287c540f2b4af65`에서 각 단어1.0으로 구성해 원본 LTokenizer에 전달했다. 공식 Qwen Git 사전과 이 checkout 사전의 전체 단어열 동일성은 별도 미검증이다. KO19개 일치를 전체 공식 processor 동등성으로 확대하지 않는다.

후속 [공식 사전 단어열 대조](forced-korean-dictionary-identity.md)에서21009개 단어 순서·17968개 고유 단어·uniform scores 맵 입력 동일성을 확인했다. 위 문장은 최초 실행 시점의 미검증 범위를 보존하며, 전체 processor/알고리즘·시간 품질 승인은 계속 남는다.

Unicode L/N과 ASCII apostrophe만 남기는 정리·빈 단위 제거·timestamp prompt 생성은 Rust domain에서 수행했다. Unicode 범주 회귀를 추가했지만 Python과 Rust의 Unicode 데이터 버전 전체 동등성을 증명한 것은 아니다. 원본 tokenizer 모듈은 변경하지 않았고 Qwen processor 전체를 import하거나 source를 잘라 실행·stub으로 대체하지 않았다. `official_qwen_processor_executed=false`, 모델 품질false를 유지한다.

## 사설 로드와 자원

nagisa0.3.0 배포물의 기본 데이터3개도 inventory 해시로 고정했다. `nagisa_v001.dict`는6152363B/SHA256 `968ac9e6c7a53051ef24d8561673dd31de81b2feb9b5bff01b1d3b6b2473113c`, `.hp`는519B/`6737f76b588315fe2fe05d05c99939d3142a0f1c6468f49f4c23e7003192f204`, `.model`은40274377B/`9db9abc06a927c56e18af8d485e20a14908138752be459a83f0dc7ac85368c1b`다. 배포물의 dtype·양자화 inventory를 따로 검증한 것은 아니며 모델 파일 크기를 RAM 수치로 대신하지 않는다.

사설4355파일과 manifest 해시를 재검증하고 isolated/no_site/no_user_site/ignore_environment/dont_write_bytecode/safe_path 및 실제 private CPython DLL을 확인했다. NumPy·SciPy·sklearn·psutil·dynet·_dynet·nagisa·nagisa_utils·soynlp의 실제 module file이 사설 root 안임을 확인해 경로·해시를 기록했다. 원본 nagisa가 sys.path에 자체 패키지 경로를 추가하는 것을 허용하되 나머지 검색 경로는 명시한 사설 표준 라이브러리/python/site-packages로 제한했다.

원본 dynet_config API에 mem256·seed42·requested_gpus0을 지정했다. 이는 초기 메모리 요청이며 프로세스 RAM 상한이 아니다. 배포물 DyNet38 2.2의 실제 dynet.__version__은2.1로 관측됐고 원문과 일치한다. soynlp의 기존 SyntaxWarning은 stderr에 보존했고 원문을 수정하지 않았다.

preflight CPU9.737098344693285%·free RAM37129154560B에서 한 worker를 실행했다. [supervision 관측](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-functions-r1/observation.json)은52.2221379초·exit0·timeout=false·출력 초과false였다. BelowNormal, BLAS/OMP/MKL4threads, private kill-on-close Windows Job, 배정 후 stdin admission,60초 기한·각 stdout/stderr1MiB 상한을 사용했다.

20ms 명목 간격2576개 표본에서 최대 private712806400B·working set539615232B·query failure0을 관측했다. 전체 파일 해시·import·초기화·tagging·출력 저장을 포함한 root child process 표본 최대이며 job 전체 메모리·hard cap·모델만의 시간/메모리·반복 성능이 아니다. 사용자 앱·캐시·VM·DPI는 변경하지 않았다.

## 재현과 다음 판단

같은 새 EXE에서 `segmentation-jer-regression-r1` 기본 명령의 원본6개 대조를 다시 실행했다.38.9035098초·exit0·timeout=false였고 counts·frames·교차 행렬이 모두 같았다. [worker 출력](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-jer-regression-r1/worker.stdout) SHA256 `8b516b1ce73187162497b011789a0ee9e4b05b814b06485e7dc766e541b80fba`는 앞선 실행기 결과와도 byte 일치한다. 신경망 실행0이며 전체 dscore CLI/모델 품질 승인은 아니다.

새 mel 분기도 `segmentation-mel-regression-r1`으로 실제 재검증했다.43.8152838초·exit0·timeout=false였고3개529152원소 모두 기존1e-5 이내·최대1.1920928955078125e-7이었다. [결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-mel-regression-r1/mel-result.json)는 최초 NumPy 실행과 별도 보존한다. 검사/로드를 포함한 시간의 차이를 tokenizer 추가나 메모리 관측의 성능 효과로 귀속하지 않는다.

실행 명령은 `jer-function-reference --segmentation segmentation-functions-r1`이다. 실행 EXE SHA256 `cf81d3dc685c14290a2606be63e943d9d8796f9958191b2aba704d73f93ab9c1`의 컴파일 입력55개·EXE1개를 [선보존](../../../evidence/player/jer-function-reference-builds/cf81d3dc685c14290a2606be63e943d9d8796f9958191b2aba704d73f93ab9c1/build-manifest.json)했다. domain 정리/prompt, application 입력 검증, 외부 tokenizer adapter, OS 메모리 관측의 책임을 분리했다. Rust3tests·fmt check·clippy all-targets -D warnings·locked offline build를 통과했다.

다음은 공식 한국어 사전 단어열 대조와 고정 nagisa 단위 목록을 사용하는 Rust 정렬 단일 요인 A/B다. fixture에 맞춘 임의 merge/split을 기본값으로 넣지 않는다. 실제 분절 차이가 기존 raw equal/repair collapse를 해결하거나 원인이라는 결론은 새 정렬 실행 전에는 내리지 않는다. 공식 encoder 수치와 사람 word 경계 검토도 남는다. soynlp GPL v3 및 보조 모델·사전·native 배포 고지는 참조 실행 성공과 별도 게이트다.
