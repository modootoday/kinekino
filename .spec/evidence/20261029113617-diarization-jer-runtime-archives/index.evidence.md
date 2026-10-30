---
id: evidence:kinekino/20261029113617
title: JER 참조 런타임 원본 배포물 확보
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JER 참조 런타임 원본 배포물 확보

2026-10-18. Rust 어댑터로 CPython embeddable과 외부 수치 라이브러리 원본 배포물5개를 한 worker에서 순서대로 확보했다. 다운로드 파일의 SHA256은 모두 고정 기대값과 일치했다. **압축 해제·설치·인터프리터 실행·외부 JER 함수 대조는 아직 수행하지 않았다.** 모델 추론0회, 사용자 Python/환경 변경0건이다.

## 확보한 파일

| 원본 | 바이트 | 실제 대조한 SHA256 |
|---|---:|---|
| CPython3.13.3 Windows AMD64 embed | 10885425 | `59ff76e16e6597de47474fb22be69e7191a89116910d728ab735079b078e52db` |
| NumPy2.3.2 cp313 Windows AMD64 | 12782876 | `c63d95dc9d67b676e9108fe0d2182987ccb0f11933c1e8959f42fa0da8d4fa56` |
| SciPy1.16.0 cp313 Windows AMD64 | 38403710 | `79a3c13d43c95aa80b87328a46031cf52508cf5f4df2767602c984ed1d3c6bbe` |
| intervaltree3.2.1 | 25929 | `a8a8381bbd35d48ceebee932c77ffc988492d22fb1d27d0ba1d74a7694eb8f0b` |
| sortedcontainers2.4.0 | 29575 | `a163dcaede0f1c021485e957a39245190e74249897e2ae4b2aa38595db237ee0` |

기대값은 [CPython 공식 Sigstore 배포 메타데이터](https://www.python.org/ftp/python/3.13.3/python-3.13.3-embed-amd64.zip.sigstore)의 message digest와 PyPI 공식 release JSON의 SHA256에서 확인했다: [NumPy](https://pypi.org/pypi/numpy/2.3.2/json), [SciPy](https://pypi.org/pypi/scipy/1.16.0/json), [intervaltree](https://pypi.org/pypi/intervaltree/3.2.1/json), [sortedcontainers](https://pypi.org/pypi/sortedcontainers/2.4.0/json). Sigstore 서명 체인 검증을 수행한 결과가 아니며 `signature_verified=false`를 기록한다. 라이선스 metadata는 취득 어댑터에 기록했으나 archive 내부 원본 LICENSE와 bundled notices 검토·보존은 추출 단계의 잔여 작업이다.

## 실행과 보존

실행 EXE SHA256은 `8911ac2eedaa53e1e3e3034a3ea5eaf7be9b6e532f2035021af823ee477d8b69`. 실행 전에 [build manifest](../../../evidence/ai/model-spike-builds/8911ac2eedaa53e1e3e3034a3ea5eaf7be9b6e532f2035021af823ee477d8b69/build-manifest.json)를 보존했으며 컴파일 입력151개와 EXE1개, 총152개다. 전체 transitive native 재현 빌드 인증은 아니다.

```text
player-model-spike.exe jer-runtime-archives jer-runtime-archives-r1
```

실제 종료코드0. 취득 전 CPU19.23828125%, 가용 RAM37037481984B로 기존 resource headroom 검사를 통과했다. 다운로드는64MiB/file 상한·순차 streaming·create_new·고정 hash/가능한 exact bytes 검사를 사용한다. 부분 파일은 실패 시 보존하며 다른 앱을 종료하지 않는다. 단독 직접 취득 실행이며 측정 wrapper의 BelowNormal·100ms peak memory·process tree timeout 관측 결과를 보유한 것으로 표현하지 않는다.

로컬 원본은 `runtime/ai-spike/source/jer-runtime-jer-runtime-archives-r1/`에 있다. [취득 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/jer-runtime-archives-r1/result.json)의 SHA256은 `f7e79f9277b52ed8d11b8dd616f3ed925df7d5df0e9196202a3b1fcb3e8449fd`, [stdout](../../../evidence/ai/model-spikes-2026-10-18/results/jer-runtime-archives-r1.stdout)은 `4be4046e53f7e45cf1765ab095fefc922148028008f7f6cbf4b083b6e9dd895b`. 보고서 snapshot1075개에 이 manifest/stream을 포함했다. 배포물 binary 자체는 `.spec`에 복제하지 않고 runtime 원본을 유지한다.

fmt check·Rust test42·clippy all-targets -D warnings·locked offline build 통과. 취득 구현은 application orchestration과 infrastructure fixed-archive adapter를 분리했으며 제품 의존성은 바꾸지 않았다.

## 남은 실제 대조

[6개 실제 입력](diarization-jer-source-inputs.md)을 사용한다. 경로 탈출·symlink·압축폭탄을 거부하는 Rust 추출, 원본 라이선스 보존, 사설 isolated CPython C-API 초기화와 실제 flags/sys.path/module hash 확인, 무수정 원본 함수 실행 및 frame counts/contingency/JER 대조가 남아 있다. [결정 경계](../../../adrs/player/adr-jer-function-reference-boundary.md)의 허용치와 full CLI 별도 판정을 유지한다. 독립 JER와 모델 품질 승인은 false다.
