---
id: plan:kinekino/20261029113955
title: 정상 제품 사용자 검토 실행 계약 — 미실행
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 정상 제품 사용자 검토 실행 계약 — 미실행

선택 소스는211r2 `0C361974F8C6544C58A99DA11E2DD0C43029B9800506354D7450DE7FF421F1BE`다. UI 보고상 fmt/type 통과이나, 정상 제품 바이너리 빌드·실행·실제9 표시·사용자 판정 근거는 아니다. 정확 파서11 pins는 `runtime/coordination/normal-product-user-review-contract-r1/parser-pins11.json`(43325D7E257D752BC25F60212224CE539087DEA9ACCAE762E65B070ABA0BDEB9), 기존 증거 참조는 같은 디렉터리 evidence-references.json에 기록했다. 이번 작업은 소스/작은 인계 문서만 읽었으며 Cargo·실행·새 바이너리 해시/복사·다운로드0이다.

## 실제 허용 입력

Windows 정상 GUI `player.exe`는 `native-proposal-window` feature를 포함해야 한다. Cargo의 정상 bin 이름은 player/player-cli다. 해당 빌드에서 별도 실행 모드가 없으면 NativeViewer가 기본이며, 아래는 `--native-viewer`로 명시한다. 값/중복 flag는 거부된다. `--ui-demo`, `--smoke-test`, `--start-url`, native-proposal 요청·validation-position과 혼합하지 않는다. preview/NoMedia/ignored libtest는 정상 제품 실행이 아니다.

| 책임 | 확인한 정확 argv / 조건 |
| --- | --- |
| 라이브 재생 | `--catalog-root ABS_DIR --mpv-executable ABS_EXE --mpv-sha256 LOWERCASE_SHA256` 전부 필요; 일부/중복/상대 경로 불가 |
| 실제 브라우저 작업 | `--job-root ABS_DIR --browser-profile-root ABS_DIR`와 위 catalog root 필요; 원 어댑터가 canonical 경로 분리/profile 소유권 검사. 이 roots는 사용자 실제9 저장소와 구분된 중앙 지정 저장소 |
| 실제9 읽기·사용자 저장 | `--review-root ABS_DIR --proposal-root ABS_DIR`; 두 정확 userroots는 아래 단일 인계 문서를 따름 |
| 기존 historical 증거 읽기 | `--historical-verifier ABS_EXE --historical-verifier-sha256 LOWERCASE_SHA256 --historical-workspace ABS_DIR --historical-package sdk-text-results-r3` 네 개 전부와 proposal-root 필요. 이것은 모델 추론 worker가 아님 |
| 관측 | `--record-validation`으로 원 LocalHost/ReadyV1/FinalV2 기록 요청; endpoint 존재나 snapshot만으로 재생·정리 성공 판정하지 않음 |
| 선택 프로젝트 | 필요할 때만 `--project-root ABS_DIR --viewer-project VALID_ID [--viewer-project-revision POSITIVE_CANONICAL_DECIMAL]`; 이 예시는 미선택 시작. 디렉터리 자동 스캔/프로젝트 생성 없음 |

저장 옵션 및 native 재생/profile/historical 값은 각각 분리형 또는 `--name=value`를 지원한다. 아래 예시는 확인된 분리형만 사용한다. 전체 argv의 임의 미지 옵션을 승인하는 의미가 아니며 `--worker-executable`, `--cli-sha256`, `--review-package`, 자동 actual9 선택 옵션을 만들지 않는다.

## 정상 GUI 예시 한 개 — 아직 실행하지 않음

`<NORMAL_PLAYER_EXE_ABSOLUTE>`는 중앙이 새로 빌드·고정하여 provenance/hash를 인수할 정상 player.exe다. 현재211r2의 그 바이너리는 미확정이며, 기존 libtest/예전 player를 대신 지정하지 않는다. 아래 normal-product-review-r1의 세 라이브 roots는 향후 중앙 배정 예시일 뿐 생성/사용한 경로가 아니다. mpv 경로도 과거 pin의 위치이므로 최종 보호/custody가 유지되는지 실행 gate에서 확인해야 한다.

```text
"<NORMAL_PLAYER_EXE_ABSOLUTE>" --native-viewer --record-validation
  --job-root "%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/normal-product-review-r1/jobs"
  --catalog-root "%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/normal-product-review-r1/catalog"
  --browser-profile-root "%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/normal-product-review-r1/browser-profile"
  --mpv-executable "D:/workspace/@myso-kr/com.kinekino.player/target/debug/mpv.exe"
  --mpv-sha256 f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790
  --review-root "%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/product-gui-real-ai9-input-r1/review-root"
  --proposal-root "%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/actual9-historical-import-r1/proposals"
  --historical-verifier "D:/workspace/@myso-kr/com.kinekino.player/runtime/quality-review/historical-import-2026-10-19-r1/player-review-sdk-export.exe"
  --historical-verifier-sha256 e00c1726603ef7d0167c6d58deb79e694071044c20a2723de8abe8a504f45e41
  --historical-workspace "D:/workspace/@myso-kr/com.kinekino.player"
  --historical-package sdk-text-results-r3
```

이것은 한 argv의 줄바꿈 표기이며 실행용 PowerShell 스크립트/새 launcher가 아니다. userroots를 붙여 실행하는 것은 실제 사용자 검토 준비 단계다. 개발자 기능 시험에는 별도 scratch roots를 사용한다.

## CLI·외부 실행 파일의 기존 근거

정상 CLI는 `player-cli capabilities --json --endpoint "<원 GUI LocalHost endpoint>"`처럼 같은 SDK에 한 요청을 전달할 수 있다. 실행 파일은 최종 고정된 절대 경로로 대체한다. endpoint forwarding에 local review/proposal roots를 혼합하지 않는다. 로컬 읽기는 `--json --input ABS_REQUEST_JSON --review-root ABS_DIR --proposal-root ABS_DIR`와 위 historical4를 사용하며 positional 명령/context flag를 JSON 입력과 중복하지 않는다. 이 읽기/전달 예시는 이번에 실행하지 않았다. 별도 `--serve-playback` CLI host는 GUI의 같은 SDK가 아니므로 정상 GUI 인수의 대용으로 시작하지 않는다.

기존 CLI 증거는 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/native-playback-factory-cpu-r2/bin/normal-cli/player-cli.exe`, 32487424B, SHA7AA175D179DEBE64E73E8D1B44A35D8E502DF8A8AD290C8EC2B89F5AD030FD89다. **211r2 동일 바이너리 근거가 아니다.** historical verifier는 위 예시 경로/1127424B/SHAE00C1726…F45E41의 기존 실제 inspect/import 근거를 재사용한다. mpv는121051136B/위 SHA와 과거 actual-environment.json 경로를 재사용한다. 새 대형 파일 해시/복사 없음. 현재 정상 player/CLI211r2와 모델 추론 worker의 고정 실행·배포 packet은 없다. 모델 실행 worker argv/pin이나 새 추론을 이 문서에서 추가하지 않는다.

## 사용자 namespace와 남은 제품 단계

실제9의 단일 기준은 `runtime/coordination/ai-review-input-r1/USER-REVIEW-HANDOFF.md` SHA9C545A4F20858C1F39EF63A41C9D9DD95637B72FC0E5755FBFD11F5C825F899A다. 오래된 CLI-NEXT의 다른 proposal-root 예시 대신 이 최신 인계의 actual9-historical-import-r1/proposals를 사용한다. package/objects는 원 입력으로 보존하고 review-root/decisions 및 proposal 수정·저장·채택 이력은 사용자 명시 동작만 기록한다. 개발자 테스트 verdict/자동 수리/초기화/판정 복제0. 저장·채택과 AI 품질 승인은 별도이며 품질 승인 주체는 사용자다.

현재 argv는 저장소 주입까지 확인된 소스 계약이다. 기존 소스 진입점은 ProductFrame/choicesNone → configure_default_chooser → ListProposals::default → onChooserList/Document/Item → onOpenLiveProposal이며, draftNone은 원 ManualRepair 경로로 간다. 이4개 선택 소스는 menu-source-pins4.json(4B86B5F0E718EE6BDBF6B14B0EEC83D094E7097FFA50C2E85325215A2508A234)에 대조했다. 실제9 목록/원본·후보 비교/수정·저장/명시 판정·이력으로 가는 실제 표시/E2E 검토는 완료로 간주하지 않는다. 최초 프로젝트/패키지를 자동 선택하는 옵션도 없다. range/next·previous/timer와 정상 player 실행은 별도 미완료다. 선택 main은 보고 writer의 반환을 `_diagnostic_failures`로만 받으므로 정상 제품에서 최종 stdout 실패를 사용자에게 확실히 표시·보존하는 연결도 UI 검토 대상으로 인계한다; 기능 exit0만으로 관측 성공을 주장하지 않는다.

Source Player는 가제이며 표시명/출력명은 domain::product::PRODUCT 책임이다. 정상 바이너리·mpv/historical verifier·브라우저 런타임·FFmpeg/폰트·공통 아이콘 등 최종 배포 자산의 패키징/권리/고정 버전·경로 검증은 미완료다. 기존 개발 경로를 사용자 배포본으로 표시하지 않는다. 기존 핵심 SDK/CLI 검증 뒤 정상 GUI 표시와 사용자 실제9 품질 시험을 중앙 packet으로 배정한다. 이번 문서는 실행/품질 승인이나 배포 완료가 아니다.

적용 스킬: 실제 읽은 modoo.today code-change-review-verification+필수 verdict-table, test-fixture-and-mock-build. 원 source/pin·실행 사실·기술 성공·사용자 품질 판정 및 namespace를 구분했다. 정확 skill 본문 hashes는 SDK211 HANDOFF를 참조한다.
