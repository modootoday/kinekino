---
id: evidence:kinekino/20261029114125
title: 프로젝트 문서와 증거 무결성 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 프로젝트 문서와 증거 무결성 검사

2026-10-19. 전체 검사 `spec-contextual-full-check-r1`은 exit0이다. 검사 시점 문서414개, 정본 로컬 링크 오류0개, 별도 분류한 upstream 링크3개를 확인했다. immutable hash105개와 build/evidence 검증도 통과했다. 이 숫자는 도구의 해당 검증 집합이며 프로젝트의 모든 파일·모델 품질·실행 파일 loaded memory를 증명하지 않는다.

[전체 검사 원문·결과·EXE·소스 스냅샷](../../../evidence/verification/spec-contextual-full-2026-10-19-r1/)을 보존했다. build-evidence.json SHA256은 `7c889179429a2b1c0f802d203cb831a96054ba9868c785a078aae2bfe0cac31a`, 사용한 EXE SHA256은 `ce4b671564eab82a5c2dc57d92cba90ebd5ac9bf5be2ae74dc2f43298b8390df`다. fmt·9tests·all-targets strict Clippy·locked offline release build를 통과했다. snapshot은 source-at-audit이며 전체 전이 compiler 입력 attestation은 아니다.

## 진단과 변경 범위

앞선 `gemma-internal-trace-integrity-r2` 전체 시도는 경로 미상 os error3/exit1로 실패했다. 원래 실패를 지우지 않았다. I/O 경로·검증 단계 context를 추가했고, `check`가 쓰지 않는 이관용 전체 파일 해시 중복 읽기를 제거했다. destination/collision 및 기존 모든 고정 증거 검증 함수는 유지했다. `plan`·`apply`의 전체 이관 inventory는 streaming hash로 유지한다. 새 전체 실행은 같은 경로 오류 없이 통과했지만, 과거 실패의 정확한 원인 경로는 당시 로그에 없어 확정하지 않는다.

## 후속 문서 링크 검사

[후속 링크 도구 EXE·소스·게이트 로그 보존](../../../evidence/verification/spec-links-2026-10-19-r1/)은 전체 검사와 분리한다. 마지막 링크 검사 결과는 이 디렉토리의 `links.json`과 `spec-links-actual-r2.stdout`에 보존한다.

전체 검사 이후 추가한 검토 패키지·실제 추적 문서는 `check-links`로 따로 확인한다. 첫 실행은420문서/정본 오류0/upstream3/exit0이었다. 이는 링크 전용이며 새 trace 원시 증거의 전체 manifest verifier 연결이나 모델 의미 품질 통과가 아니다. 링크 전용 도구 변경도 fmt·9tests·all-targets strict Clippy·locked offline release build를 통과했다.

SDK r145의96파일·4실행파일 고정·21실제 자식 기록은 전체 결과의 `native_gui_endpoint_pinned` 항목에서 검증됐다. [전용 검사 범위](../application/native-gui-pinned-central-audit-2026-10-19.md)와 기존 r137 한계는 유지한다.

## 검토 패키지 전용 검사

`check-review-pinned`를 추가해 텍스트 r3 package 고정 SHA,9개 항목의 source/baseline/candidate가 참조하는12개 고유 object의 SHA·바이트 길이, 저장된 SDK inspect 응답의 고정 SHA·package 동일성·자동 적용false, exporter 고정 SHA를 확인했다. 전체 `check`도 이 검사를 호출한다. 전체 검사를 새로 실행한 것으로 확대하지 않는다.

[검사 도구·소스 스냅샷·원문 로그](../../../evidence/verification/model-review-pinned-2026-10-19-r1/)를 보존했다. 도구 SHA256은 `002ca20ef09555d178d9f9b920af9982d4a8220aaa9c65a94e3cbea3d31e2647`, 실제 전용 응답 SHA256은 `4b0aadfa56b2fbe2d30b8fdd2586b50f8b4ef65209a52b3bdb73d7c10d483d67`이다. fmt·10tests·all-targets strict Clippy·locked offline release build·전용 실제 검사 모두 통과했다. 이 검사는 재추론·GUI 실행·사용자 판정·의미 품질·전체 전이 compiler 입력을 증명하지 않는다.

같은 도구의 후속 링크 전용 실행은429문서/정본 링크 오류0/별도 upstream3/exit0이었다. 위 보존 폴더에 `review-pinned-links-r1.stdout`과 stderr를 추가했다. 이는 새 전체 hash 검사 통과를 뜻하지 않는다.

## IPC 진단 봉인 중앙 검사

`check-host-diagnostics-pinned`는 [r2 봉인](../../../evidence/player/local-host-diagnostics/2026-10-19-r2/manifest.json)의 SHA256 `4883bc15bfd3e4accbbf9af12fe8d55399ffde5b2a4fbb0e28c1e32979389006`과60개 엔트리의 경로·중복·바이트 길이·SHA를 확인했고 exit0이다. 전체 `check`에도 연결했지만 새 전체 실행은 아니다. 최초 두 실패의 원인 미확정과 선택 source-at-audit/전체 compiler 입력 미증명 상태를 고정 manifest에서 함께 검사한다.

[중앙 도구·소스 사본·게이트·실제 응답](../../../evidence/verification/local-host-central-2026-10-19-r1/)을 보존했다. 도구 SHA256은 `8bfdbbc0f294e5983f5875459e8b47e4c132f5c3fec781096f8a072c6931afb4`, 실제 응답 SHA256은 `3ac64dd865ea4d49cafdb912d9762eb993582c11979163dc5ef24b023595ef40`이다. fmt·10tests·strict all-targets Clippy·locked offline release build도 통과했다. 이 파일 무결성은 새 제품 실행·사용자 품질 승인을 뜻하지 않는다. r1 봉인의 상대 링크 실패와 가역 격리는 [원래 진단 기록](../../../evidence/player/local-host-parallel-diagnostics-2026-10-19.md)에 남긴다.

## 내부 추적 증거의 중앙 검사

`check-trace-diagnostics-pinned`는 R2 source(D61)의기존18쌍/새3파일과 startup source(FEA352)의기존19쌍/새5파일,각설정4개·patch·고정manifest를검사했다. R2 실제실패의고정manifest/rawjournal/측정,3count파일SHA,3generation파일과manifest동일성,planSHA,engine97a3/workerD5의EXE바이트도확인했다. 전용실행exit0이며 rawschema2/invalid2/completefalse/full_boundary_gatefalse를그대로확인하는검사다.

[도구·소스사본·gate·실제응답](../../../evidence/verification/gemma-trace-central-2026-10-19-r3/)을보존했다. EXESHA256 `8f743caa401527da27ff54eecd9a3039ee6b702643e96d517178a9f8ac7d0d5c`,응답SHA256 `8eb5f499e2dba346e51d383445b137b7391454f08dd556175d8caa23e4d1c668`이다. fmt·10tests·strictall-targetsClippy·lockedofflinereleasebuild통과후실행했다. 전체check에도연결했지만새전체검사를실행한것은아니다. source무결성통과는새startup실제실행·capture완결·비간섭·의미품질을증명하지않는다.

## 저장장치 관측의 한계

06시 KST 읽기 전용 Get-Disk/Get-Volume 관측에서 D:는ST2000DM008 SATA HDD, C:는Samsung970EVO Plus NVMe였고 C: 여유는137,104,289,792B였다. 별도 PerfDisk 표본에서 D: queue6/read1,369,851B/s/write4,030,828B/s, C: queue0를 관측했다. 복수 target 빌드의 I/O 경합 가능성을 보여 주지만 local_host 실패의 원인이나 반복 성능 벤치마크로 확정하지 않는다.

진행 중인 검사·사용자 앱을 중단하지 않고, 다음 새 native 통합 빌드는 명시적 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native` target과j1을 공유하도록 조율했다. 전역 Cargo/OS 설정·기존 target 삭제·VM/DPI 변경은 하지 않았다. 실제 속도 개선은 아직 비교하지 않았다.
