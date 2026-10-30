---
id: evidence:kinekino/20261029113643
title: 공식 encoder 비교용 Torch CPU archive 취득
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 encoder 비교용 Torch CPU archive 취득

2026-10-18. [원본 encoder 준비](../../player/infrastructure/inference/research-followups/forced-encoder-pytorch-readiness.md)의 다음 게이트로 공식 Windows CP313 CPU wheel을 Rust 도구로 실제 취득했다. 모델·encoder 실행과 의존성 전체 import는 아직 미검증이다.

## 고정 artifact와 실제 검사

[공식 CPU index](https://download.pytorch.org/whl/cpu/torch/)의 torch-2.9.1+cpu-cp313-cp313-win_amd64.whl, 110859971B, SHA256 `404a7ab2fffaf2ca069e662f331eb46313692b2f1630df2720094284f390ccef`가 실제 bytes와 일치했다. +cpu 없는 PyPI artifact로 대체하지 않았다. METADATA의 Name/Version과 WHEEL의 cp313-cp313-win_amd64 tag를 확인했다.

압축파일의 모든 멤버를 끝까지 읽어 CRC·크기를 확인했다. Windows 경로 별칭·경로탈출·중복·링크·암호화·특수 파일을 거부한다. 실제 파일11553개, 확장416266135B, 최대 torch/lib/torch_cpu.dll 262656512B다. archive 원본 bytes는 변경하지 않았다.

기존 취득64MiB·추출단일파일128MiB 제한을 전역 완화하지 않았다. 전용 domain pin과 고정 SHA 검사 뒤에만 추출 단일파일512MiB·전체2GiB·archive entry20000 한도를 적용한다. 기존 reference profile은 이전 한도를 유지한다. 크기가 다른 archive 및 같은110859971B의 다른 bytes를 destination 생성 전에 거부하는 회귀 테스트를 추가했다.

## 근거와 실행 경계

- [취득 manifest·전체 멤버 CRC·METADATA](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-torch-cpu-r1/result.json), SHA `a97c9e5b5ece3ae8620ea6eda70ff2c219b3bc030fa5088867fd172d469312fa`
- [취득 build](../../../evidence/ai/model-spike-builds/d5499d9caee56fabc92ee3dcb5ff9684509fbf09ed408decadfb8a086212264d/build-manifest.json), compiled inputs181+EXE1
- [추출 build](../../../evidence/ai/model-spike-builds/870967e03d80db777702d55403fb17798523f40ae14b4bd3f00897e5d984a1b8/build-manifest.json), compiled inputs181+EXE1

취득 시작 CPU31.818%·빈RAM35262251008B의 admission을 확인했다. 단일worker이며 사용자 앱·host Python 설치를 변경하지 않았다. 취득·추출 build의53tests·fmt·clippy·locked offline build가 통과했고 후속 거부회귀 추가 후54tests·fmt·clippy도 통과했다. 후속 test 추가 소스와 실제 실행 build를 구분한다.

추출은 새 runtime/ai-spike/source/encoder-torch-private-r1에서 exit0으로 종료했다. 11553파일·416266135B의 파일별 SHA를 [추출 manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-torch-private-r1/result.json)에 보존했다. manifest SHA는 `5a3edabcab03837da7e529d375fc04804964d47092d49c34cf7d1f30755ef5e0`, torch_cpu.dll SHA는 `8b95ea5c16f259d8b1eb363ae352741ab6d082cb32301b4f95355821b5ef4973`다. 기존 환경을 덮어쓰지 않았다. 이 directory만으로 완성된 interpreter 환경이 되지 않는다.

다음은 나머지 고정 의존성·원본 package tree를 별도 환경에 구성하고 DLL origin/import·strict weight load·F32 CPU/eager/eval/no_grad/threads4를 확인하는 것이다. 같은 full mel로 ko1883 최소 실행 후 세 입력 전체·stage tensor를 비교한다. 원본 framework 수치·timestamp 품질·배포 라이선스 전체 승인은 아직 주장하지 않는다.
