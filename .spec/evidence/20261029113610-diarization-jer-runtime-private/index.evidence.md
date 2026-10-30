---
id: evidence:kinekino/20261029113610
title: JER 참조용 사설 런타임 추출
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JER 참조용 사설 런타임 추출

후속으로 [실제 격리 초기화와 원본 JER 함수 대조](diarization-jer-functions-actual.md)를 완료했다. 아래 interpreter 미실행 문구는 이 추출 단계의 범위이며 후속 실행 결과와 구분한다.

2026-10-18. [해시 검증한 배포물5개](diarization-jer-runtime-archives.md)를 대상으로 Rust 추출 어댑터를 추가했다. 제품 SDK 의존성이 아닌 root 검증 도구의 외부 reference 준비 단계다. 원본 archive나 사용자 Python을 변경하지 않으며 새 workspace 폴더에 `python/`과 `site-packages/`를 분리한다. **인터프리터·JER 함수 실행은 아직 수행하지 않았다.**

## 책임과 제한

application은 label 검증·자원 검사·고정 archive 재검증·새 destination 및 성공/실패 보고서를 조립한다. domain의 archive path 정책은 parent/current components, 절대경로, Windows device/ADS/역슬래시/끝점·공백 별칭을 거부한다. infrastructure는 고정 `zip=9.0.5`의 실제 ZIP reader를 통해 encrypted/link/special entry를 거부하고 CRC/명시 크기와 실제 읽기 바이트를 확인한다. ZIP 경로를 임의로 정규화해 다른 원본처럼 만들지 않는다.

파일128MiB, 총 해제768MiB, archive당10000 entries, 전체20000 files로 제한했다. 대소문자 중복을 거부하고 create_new로 기존 파일을 덮지 않는다. 새 destination에만 추출하며 canonical parent가 destination 내부인지 검사한다. 별도의 hostile local process와 TOCTOU 방어를 증명한 결과는 아니다. 오류 시 부분 폴더와 failure.json을 남기고 같은 label을 재사용하지 않는다.

## 검사와 실행 식별

fmt check·test46·clippy all-targets -D warnings·locked offline build 통과. 새 검사는 Windows 경로 탈출/별칭, 실제 ZIP 원본 byte 복원, 기존 destination 거부, 대소문자 충돌 시 최초 파일 보존, 실제 member CRC 변조 거부를 포함한다. 초기 checked_add 정수 타입 컴파일 오류를 u64 명시로 수정했다. 성공하지 못한 빌드는 실행 증거로 취급하지 않는다.

실행 전 보존 EXE SHA256 `7eb7a24523d7ab39c4eec711784d70627e0f40cc651928e7774c52d8af315bee`, [build manifest](../../../evidence/ai/model-spike-builds/7eb7a24523d7ab39c4eec711784d70627e0f40cc651928e7774c52d8af315bee/build-manifest.json)의 컴파일 입력154개와 EXE1개, 총155개다. Cargo.lock에는 ZIP 관련 의존성 변경을 기록했다. 모든 transitive crate/native source의 재현 빌드 인증은 아니다.

```text
player-model-spike.exe jer-runtime-prepare jer-runtime-private-r1
```

## 다음 실행 게이트

실제 준비 실행은 종료코드0으로 완료됐다. 파일2500개·해제179809611B이며 각 파일의 archive 연결·상대경로·크기·SHA256을 [결과 manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/jer-runtime-private-r1/result.json)에 보존했다. manifest SHA256은 `fdc420936e19f732fe1b11e3a5a28a028a69ab19fb45409e0ebf8c8f2b070d7b`. 자원 검사 CPU20.8984375%·free RAM35401125888B, 실제 interpreter_executed=false다. 보고서 snapshot1082개에 manifest/stdout/stderr를 포함한다. 해제한 전체2500개 binary/source는 runtime 원본에 있고 `.spec` 복제 증거로 주장하지 않는다.

원본 notice11개 경로/hash를 inventory에서 확인했다. CPython LICENSE.txt, NumPy/SciPy dist-info LICENSE.txt, intervaltree의 licenses/LICENSE.txt와 sortedcontainers LICENSE를 수정 없이 추출했다. 라이선스 본문 일부와 배포 metadata를 읽은 준비 확인이며 법률 감사나 제품 재배포 패키지 완성은 아니다. 실제 runtime DLL 적재와 dependency 호환성은 아래 게이트로 남는다.

원본 `python313._pth`는 archive 내용대로 보존했다. 파일의 `#import site`나 선언한 directory layout만으로 실제 CPython 격리를 승인하지 않는다. Rust C-API에서 isolated init, site/user site/environment/CWD 차단을 설정하고 실제 flags·sys.path·loaded module/DLL hash를 관측해야 한다. 고정 dscore 원문 함수 호출, 실제6개 입력의 frame/contingency/JER 비교, process-tree60초 및 stream1MiB 제한은 [결정 경계](../../../adrs/player/adr-jer-function-reference-boundary.md)의 후속이다. 전체 dscore CLI와 독립 JER parity, 모델 품질은 여전히 미완료다.
