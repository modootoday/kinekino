---
id: evidence:kinekino/20261029113631
title: Gemma270 startup 엔진·worker 인계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 startup 엔진·worker 인계

2026-10-19. [생명주기 구현](generation-gemma270-startup-lifecycle-implementation.md)의 CPU 엔진을 별도 Stage/Revision으로 보존한다. 이 문서는 엔진·worker 식별과 실행 인계만 담당하며 모델 품질 승인을 뜻하지 않는다.

## 엔진 고정

- engine ID: `gemma270-startup-lifecycle-cpu-r3`
- [보존 manifest](../../../evidence/ai/engine-cli-builds/gemma270-startup-lifecycle-cli-r3/manifest.json): SHA256 `23df8b5052db2f33b285a6dc7839310afd8324a86ca50b349cfa5e7e6bd4ce47`
- 같은 폴더의 `mistralrs.exe`: 99,084,288 bytes, SHA256 `b7ece1736cf49d85523fe514cd09fb65bb1e3912c8f634677e1264196462a7f8`
- source manifest: `fea352797c0a162f22407fdfc6323cd52ff2967306ca7c5ec420e4f326c5bd87`

Rust snapshot은 source manifest 전체와 build-inputs의 project_patch 일치, build 후 source 검증 결과, EXE 크기·SHA, 복사 후 원본 재검증을 요구한다. R1/R2와 기본 Revision은 유지하며 startup Revision은 Gemma270 외 모델이나 다른 엔진으로 대체하지 않는다. 이는 기록된 입력의 검증이며 전체 전이 의존성 compiler attestation은 아니다.

## 최종 worker와 gate

worker SHA256은 `f763a4cf9800c6492b91b73219cc7c785d5e4050a9797b96be4cb004f7f9ee41`, 크기는41,413,120 bytes다. [보존 manifest](../../../evidence/ai/model-spike-builds/f763a4cf9800c6492b91b73219cc7c785d5e4050a9797b96be4cb004f7f9ee41/build-manifest.json)의 SHA256은 `8ed86c28c409dc483bb93edc9680a5b78c507ebb97f80d44c65f7064e58281b9`다. 컴파일 당시 입력206개를 현재 파일과 대조한 뒤 EXE1개와 함께207개를 보존했다. 동반 ONNX DLL은 없으며 이 사실로 링크 모드나 완전한 패키징을 추정하지 않는다.

[검증 원문](../../../evidence/verification/gemma-startup-pin-2026-10-19-r3/)에서 engine-lab의 fmt check·3tests·all-targets strict Clippy·locked offline build와 최종 model-spike의 fmt check·70tests·all-targets strict Clippy·locked offline build가 모두exit0이다. model-spike-final 접두 로그가 마지막 wrapper 허용 목록 수정 이후 결과다. 비추론 `verify-generation-startup-engine`과 `preserve-build`도exit0이며 모델 forward0이다. vendor 최종 strict Clippy 원문도 같은 폴더에 보존했다.

PM은 프로젝트 루트에서 다음 명령을 실행한다. 양쪽 worker 경로와 label은 동일해야 한다. 이미 사용한 label은 새 `gemma270-boundary-` 접두 label로 양쪽 모두 교체한다. worker를 재빌드하면 이 pin을 재사용하지 않는다.

```powershell
& 'D:\workspace\@myso-kr\com.kinekino.player\tools\model-spike\target\debug\player-model-spike.exe' observe-generation-boundary gemma270-boundary-startup-lifecycle-r1 'D:\workspace\@myso-kr\com.kinekino.player\tools\model-spike\target\debug\player-model-spike.exe' generation-boundary-startup-lifecycle gemma270-boundary-startup-lifecycle-r1
```

## 고정 입력과 실행 예산

[동일3case fixture](../../../evidence/ai/generation-boundary-fixtures/gemma270-boundary-trace-input-r3/manifest.json)의 SHA256은 `85009c8ea7f6267daca32b3e25b39463443c6f2c0219b31ed366e9e185296b95`다. trace-plan은 `e3408fbfd0f10736856459875a499215f6170231b5aa8f7cce353718decc5e9e`, requests는 `1aa40075fc6eceab49e075ed8c56f554c8fd6b24b97820e6cb9faf84ddd4d5e4`로 검사한다. 새 실행의 run_id와 workspace_root만 재지정한다. 기대 raw schema는3이다.

PM이 단일 worker로 실행한다. outer admission은 가용 RAM 12GiB 이상, 유한 CPU 사용률 0 이상 60% 미만을 요구한다. 외부 worker 제한은600초이고 HTTP 요청 제한은90초다. BelowNormal과 수치 라이브러리 thread 환경4를 사용하며 OS 전체 thread cap4를 보장하지 않는다. 새 다운로드·모델 변경은 포함하지 않는다.

## 판정 경계

위 pin 준비는 비추론 검증과 보존으로 완료했다. 이후 PM의 단일 실행을 [startup 실제 관측](generation-gemma270-startup-lifecycle-actual.md)으로 별도 보존했다. terminal witness, off/on 비간섭, resolved controls·flush ACK·공통 DTO strict join은 해당 실행·검증 근거가 있어야 완료할 수 있다. raw complete 또는 worker exit0만으로 이들을 통과시키지 않는다. 모델 품질 검토는 별도 후속이며 이 인계에 혼합하지 않는다.

## 통합 무결성 검사 범위

`tools/spec-maintenance`의 `check-trace-diagnostics-pinned`는 기존 R2 actual의 incomplete/invalid2/품질 미승인 상태와 R2/R3 source 검증을 유지하고, 새 R3 CLI archive11파일과 최종 worker207파일 검증을 추가한다. 보존 manifest의 고정 SHA를 먼저 확인하고 각 파일의 경로·크기·SHA와 worker identity를 검사한다. 현재 소스로 보존본을 대체하지 않는다. 해당 검사 자체도 추론을 실행하지 않는다.

이 확장 후 fmt check·11tests·all-targets strict Clippy·locked offline build와 실제 `check-trace-diagnostics-pinned`가 모두exit0이다. [같은 검증 폴더](../../../evidence/verification/gemma-startup-pin-2026-10-19-r3/)의 spec-maintenance 접두 원문이 근거다. 검사 EXE SHA256은 `724eb3ce428c307a73159a477c4d7e8b7cfe0a03ea71c967f54f71932da4937e`/4,893,696 bytes다. 출력은 R2 실패 보존과 R3 engine11/worker207 파일 무결성을 각각 구분한다.
