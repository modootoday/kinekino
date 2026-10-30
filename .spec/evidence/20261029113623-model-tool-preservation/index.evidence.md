---
id: evidence:kinekino/20261029113623
title: 측정 도구의 실행 전 보존
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 측정 도구의 실행 전 보존

2026-10-18. [과거 증거 감사](provenance-audit.md)의 누락과 최신 mutable source 한계를 소급 해소했다고 주장하지 않는다. 이 계약은 새로 고정한 tool build와 그 이후 실행에 적용한다.

## 보존 계약과 실제 실행

AMI 후속: 취득/가공 build `3f6e2479d8b37e1fdcbc58e61ca286ce3ab61649f50a3562f520249c1dab3378`와 실제 관측 build `43bedaa70bb822f0e51f0a44336bdaa8dc326bd346ef7bde5e7b4846f2061b41`는 각각77파일을 선보존하고19tests/fmt/Clippy/locked offline build를 통과했다. 같은60초 crop의 FP32/INT8 actual2회를 보존했다. 별도 Rust 평가기5202d56c의 EXE/manifest SHA와 verify-build23파일을 root가 확인한 뒤 실제 DER/JER2결과를 새 경로에 저장했다. 평가기 준비의 합성19tests와 actual 모델2회·평가2회를 구별한다. 그 full dependency archive나 공식 scorer 수치 parity를 완료했다고 하지 않는다.

원본 참조 관측 gate 후속: 실행 orchestrator `b0ef04dd38617d68be5eb4b0708e950a154a0c2bc9a98a6ecb67d8fe6714356b`와 사후 대조 build `65d0c28ce86f3ee7ec67a7737d24886772cfc26b165c74c3d0bfd8b85410d28f`는 각각75파일을 선보존했다.18tests/fmt/Clippy-Dwarnings/locked build 통과 후 원본 R2 실행·자원 admission 검사·독립 수치 대조를 수행했다. 후속 report는647개 snapshot이다. spec-maintenance의 최종30/pre-final29개 참조 manifest 파일 검사와 immutable105개/정본203문서 링크0·보존upstream1 검사가 통과했다. 해당 full check는 동일 source의 release 빌드로 수행했고, 최적화된 빌드 사용을 모델 성능 결과로 표현하지 않는다.

E5 후속: 준비·취득 build `cf3c76c71afb49d3c55d533b3ca5dfa208a4a7313c5a3de6b16c3de1b6fcb5cd`와 실제 추론 build `e984bd3cf688091c143b6a4e0366f835bc77d0e5c27bea57749fffb4817db6a6`는 각각73파일을 선보존했다. 실제 label e5-fp32-observed-r1은 후자의 worker/orchestrator/manifest SHA를 기록했고33입력 추론과 평가를 끝냈다. 두 시점 모두17tests/fmt/Clippy-Dwarnings/locked build가 통과했다. 후속 report는 고정 raw catalog와 acquisitions를 포함한631개 snapshot을 기록했으며 가중치는 증거 폴더에 중복 복사하지 않았다. 이전 MiniLM/과거 실행을 이 새 build로 소급 연결하지 않는다.

후속 취득 기록 보완: Rust `fetch-record`를 추가한 build `d8d6a157a56edc739b5b1f03e30ecc5d57c92e340c0361654a3f128046ae4e22`가 원본 MiniLM3파일을 취득했고 source71파일을 보존했다. 이어 acquisitions JSON의 report 수집을 추가한 build `507d90cf752f13d0164281eca7b2520746aee9c73e55d56eef2cd9e64b9edefa`도71파일을 보존했다. 두 build 모두 해당 시점 fmt/test16개/Clippy-Dwarnings/build를 통과했다. 후자는 신경망 실행 없이 report582개 snapshot을 생성했다.

기존 `minilm-original-r1` label로 fetch-record를 실제 재호출했을 때 파일 존재 오류183으로 취득 전에 거부됐다. result.json의 전후 SHA256 `6a2019040c0ef1c8b46b2d90cf2970f37a1e17aeb4fb7b8f08e7ab721ae899f5`가 동일했다. 기존 download-embedding.json은 새 명령의 출력 경로가 아니다. label 재사용 거부는 원자적 디렉토리 생성으로 보장하며 일반 cached model file은 별도의 hash 검사를 유지한다.

Rust model-spike build.rs가 자체 src/Cargo.toml/Cargo.lock/build.rs와 외부 경로로 공유하는 validation processes.rs의 byte 크기/SHA를 컴파일 시점에 기록해 실행 파일에 포함한다. `preserve-build`는 현재 파일을 그 지문과 대조하며 변경됐으면 재빌드를 요구한다. 완성본은 전체 EXE SHA를 디렉토리 이름으로 사용하고 기존 경로를 덮어쓰지 않는다. source·lock·EXE 및 파일별 SHA manifest를 보존하며 가중치는 넣지 않는다. 결정적 재빌드나 전체 dependency cache·native 패키징의 증거는 아니다.

최초 `0e0d3375545402fb6b13d97807f11968279eee4a55738b3c41d96606420f0b1b` 시도는 EXE 옆 onnxruntime.dll을 필수로 가정해 실패했다. [불완전 표시](../../../evidence/ai/model-spike-builds/0e0d3375545402fb6b13d97807f11968279eee4a55738b3c41d96606420f0b1b/INCOMPLETE.md)를 남겼고 완성 manifest가 없으므로 성공 건수에 포함하지 않는다. 실제 로컬 ort-sys build output에서 static=onnxruntime 링크 선언을 확인했다. DLL 부재만으로 링크 방식이나 외부 의존성이 전혀 없다고 추론하지 않도록 helper의 표현도 제한했다.

수정 후 EXE `342202cad0c869fd7ab07ab7bbf4d467aee7fc1237387fa1c04ab555f62d6da7`의 [완성 manifest](../../../evidence/ai/model-spike-builds/342202cad0c869fd7ab07ab7bbf4d467aee7fc1237387fa1c04ab555f62d6da7/build-manifest.json)가66개 항목을 보존했다. companion DLL은 false이며 source 지문 일치는 true다. 해당 코드의14시험·fmt·Clippy·실행 파일 빌드와 실제 보존 명령이 통과했다. 이 EXE로 새 모델 추론이 이미 실행됐다는 뜻은 아니다.

## 검사와 다음 실행

별도 spec-maintenance가 완성 directory 이름·EXE hash·각 manifest 파일 크기/SHA·경로 경계를 검사한다. 불완전 directory는 INCOMPLETE.md가 있어야 하며 성공 manifest와 분리한다. 완성 manifest의 파일 무결성 검사가 컴파일러/환경/모델 출력의 재현성을 증명하지는 않는다.

실제 root 검사에서 새 보존본66개 파일이 통과했고 manifest SHA256은 `8e3b85eb0c7ff95b36598fbc6e28e6174057ea559b98f1786f111cd54307d8ce`였다. 실패한0e0d directory는 `incomplete_not_counted`로 구분했다. [검사 결과](../../../evidence/ai/model-spike-build-integrity-2026-10-18.json)를 별도 보존했다. 검사 도구4시험/fmt/Clippy와 문서175개 링크 검사(깨진 링크0)가 통과했다.

다음 자체 tool 추론은 실행 EXE hash가 이 보존본과 같은지 확인한다. source 변경 후에는 다시 빌드·보존해 새 identity를 사용한다. 과거 binary/source가 없는 실행을 최신 소스로 대체해 동일 실행이라고 표시하지 않는다. Git 도입 이후에도 로컬 evidence는 추적 제외 상태로 유지되며 별도 백업/보존 대상이다.

## PCM 배율 후속 build

2026-10-18의 WeSpeaker 전처리 단일 요인 실험 build는 `9ef54944aa203d5b08dc5945baf0ac15a3e5e731060f7accfd10ccc203db623e`다.78파일·manifest SHA256 `c75e8440fee8a5b83c1d420c112c9efa455a260f90cbc6b5bf1ce5b6d7b3fca6`를 실제 추론 전에 보존했다.24tests/fmt/Clippy all-targets-Dwarnings/locked offline build가 통과했다. [실제2회 관측과 품질 개선 미관측](diarization-pcm-scale.md)을 별도로 기록했다. 보존 경로의 직접 worker 실행은 gate가 거부했고 허용 target 경로의 동일SHA로만 실행했다. 이전54d2 local-anchor build나 과거 실행 증거는 덮어쓰지 않았다.
