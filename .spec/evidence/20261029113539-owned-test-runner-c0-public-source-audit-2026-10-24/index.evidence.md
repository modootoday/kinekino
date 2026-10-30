---
id: evidence:kinekino/20261029113539
title: owned-test-runner C0 출처·공개 경계 읽기 감사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# owned-test-runner C0 출처·공개 경계 읽기 감사

2026-10-24 PM source-only 배정. SDK sole-heavy에 간섭하지 않고 후보 정확8·기존 standalone lock·그 lock의 로컬 registry manifest/LICENSE/.crate만 읽었다. 코드 추가/수정/이동/Cargo/빌드/OS/공개 예외/stage/commit/push0. 프로젝트 라이선스 부여나 공개 승인 결과가 아니다.

## 고정 입력과 파일별 판정

정확 후보는 `runtime/development/owned-test-runner-forced-success-r1/effective8.json` SHA256 `310535659BF454959B9306BB5437E65A66E6EA6DB63D4DCBEE16E5805C451DF7`의 논리8이다. source2 1080을 유지한다. 아래 SHA 앞8은 원 manifest의 전체 SHA로 연결된다. 향후 timeout 병합·책임분리로 새 bytes가 생기면 이 감사 범위 밖이다.

| 논리 경로 `tools/owned-test-runner/` 기준 | SHA 앞8 / bytes | 로컬 출처와 민감 경계 판정 |
| --- | --- | --- |
| Cargo.toml | 5099C801 /349 | r2 raw. owned-process 상대 path 및 registry 버전만; license 없음, 작성 권리 확인 미완료 |
| src/main.rs | 6CD544FB /2169 | r2 raw. CLI args·표준출력/오류·원owner 유지. 사용자 경로는 실행시 입력/JSON에 들어가며 baked 개인경로 아님 |
| src/application.rs | EECF626E /14493 | forced-success raw. r3 후계 exact2의 app. artifact guard/원 SDK/selection/receipt. 출력 JSON은 원 executable/evidence 경로·PID/creation을 포함할 수 있으므로 실제 JSON은 private |
| src/receipt.rs | 7686B67A /2515 | r2 raw. SDK receipt를 schema owned-test.process/1로 표현. 자체 license/copy attribution 표시 없음; 외부 SDK 호출을 소스 복사로 단정하지 않음 |
| src/request.rs | E4D18D0D /5025 | r2 raw. local absolute path/pin/예산 입력 계약. 실제 개인경로 literal 없음. operator pin이 compilation proof가 아니라는 주석 유지 |
| src/guarded_artifact.rs | 3A003A66 /7315 | r2 raw. sha2·공식 windows API 호출/handle guard. source origin attribution 표시 없음; Win32 API 이름 사용 자체가 외부 코드 복사 증거는 아님 |
| src/test_result.rs | 168CF4F6 /1914 | r2 raw. libtest 출력 parsing. 모델/미디어/외부 소스 포함 지시 없음 |
| src/tests.rs | A6311C14 /22427 | forced-success raw. fixture 문자열 `C:\\fixture\\…`는 합성 경로. TEMP/TMP와 OWNED_TEST_BASELINE_CHILD/SHA256/BYTES는 실행 환경에서 소비, 고정 실제 사용자 경로/비밀값 아님. Command는 고정 baseline 자극이며 실제 binary/미디어를 소스에 포함하지 않음 |

standalone lock은 S의 `tools/owned-test-runner/Cargo.lock`, SHA256 `0B2562E8D295AEE5602407228C3A5A21DBD5E478299A685CE94780FADE49226B`다. 읽기만 했고 S 코드/lock 수정0이다. lock에는 registry URL/checksum/name/version/의존성 및 local owned-process/owned-test-runner 식별자가 있으며 사용자 절대경로·credential URL은 관측하지 않았다. registry checksum은 인증 토큰이 아니다.

원r3 source8 7336E34F→후계 exact2 1080/delta2 A4D7C180의 로컬 lineage는 보존돼 있다. timeout source8 003D22E2·red0F7A는 별도이며 이 감사가 그 모든 변형의 전체 소스를 다시 승인한 것은 아니다. runtime manifest의 `source`에 든 사용자 절대경로 자체는 private provenance라 공개 source inventory에 복사하지 않는다. 기존 fixed BIN/Cargo JSON/raw 출력/FileID/PID/개인 경로/registry 사본도 runtime에 유지한다.

탐색은 위 manifest가 지시한 파일8만 rg 인수로 전달했다. URL/Windows·Unix 사용자 경로/token/password/secret/API key/copyright/license/copied/adapted/include_str/include_bytes/env 매크로와 알려진 key prefix/PEM/Bearer 등을 확인했다. 탐색된 절대경로 literal은 tests의 합성 C:\\fixture뿐이며, 탐색한 credential·개인계정 패턴과 외부 소스 포함 매크로는 결과0이다. 이는 선택 패턴 탐색과 관련 코드 읽기의 관측이며 고엔트로피/인코딩된 비밀·복사 출처·권리 귀속 부재의 완전 증명은 아니다. Git history·외부 코드 유사도·개별 기여자의 권리 문서·자동 secret scanner 전체감사는 미실행이다. 신규 실행 도구를 만들거나 다운로드하지 않았다.

## 정확 lock과 캐시 의존성

[고정 의존성 inventory](owned-test-runner-c0-dependencies-2026-10-24.json) SHA256 `0D7F9DC3F8BFCD918F3220D3FA0B51C210E7AA3D187B7B77EE1E6A35D2532149`에 registry33의 정확 name/version/lock checksum/edge block/manifest SHA/license 선언과 원 LICENSE 계열 파일66의 경로/SHA/bytes를 기록했다. JSON은 로컬 절대경로가 있으므로 private 근거다.

기존 cache `.crate`33개 SHA가 lock checksum33개와 모두 일치했다(불일치0/다운로드0). 이것은 archive pin 관측이다. cache source의 sha2 디렉터리에는 `.cargo-checksum.json`이 없음을 확인했으며, 모든 추출 파일을 archive 내용과 전수 byte 대조한 것은 아니다. 따라서 로컬 manifest/license 각 hash를 별도로 보존했고 archive 체크섬 일치만으로 추출 소스 무변조를 인증하지 않는다.

33개는 **lock inventory**이며 현재 Windows target의 실제 linked/runtime dependency33을 뜻하지 않는다. libc나 proc macro·서로 다른 syn2.0.117/3.0.9 등도 포함된다. Cargo metadata/build를 실행하지 않았으며 실제 바이너리의 포함 범위·배포 묶음은 별도 판단이다. local owned-process와 runner2개는 registry33에 포함하지 않는다.

관측한 선언은 다음과 같다. generic-array0.14.7/zmij1.0.23은 MIT, memchr2.8.3은 Unlicense OR MIT, unicode-ident1.0.27은 `(MIT OR Apache-2.0) AND Unicode-3.0`, version_check0.9.5는 manifest에 `MIT/Apache-2.0`, 나머지는 `MIT OR Apache-2.0`다. 전체 정확 버전은 inventory에 있으며 MIT 계열을 선택할 수 있다는 사실을 프로젝트 자체의 MIT license 결정으로 확대하지 않는다.

캐시 원문에서 확인한 재배포 관련 조건을 기록한다(법적 적합성 승인 아님).

- 읽은 MIT 원문들은 저작권·permission notice를 copies 또는 substantial portions에 포함하는 조건을 둔다. copyright 소유자가 다른 RustCrypto 각 crate, Microsoft, generic-array, libc, memchr, typenum, version_check의 원문을 구별해 hash로 보존했다. itoa와 같은 `23F18E03…` 원문을 공유하는 serde/syn/proc-macro/quote/zmij군은 동일 SHA 원문을 참조한다.
- Apache-2.0 원문은 license 사본 제공, 수정 파일의 변경 고지, 관련 저작권·특허·상표·attribution 보존, 배포 원문에 NOTICE가 있을 때 그 관련 내용의 readable copy 조건을 명시한다. Windows 원문 `C16F8DCF…`의 전체 조건을 읽었다. 다른 crate Apache 원문도 hash inventory는 있으나 전체 변형 비교·그 하위 source NOTICE 전수탐색은 수행하지 않았다. 외부 dependency 파일 변경0이며 MIT 선택/Apache 선택은 아직 배포 결정으로 확정하지 않았다.
- unicode-ident의 별도 LICENSE-UNICODE 원문 `F7DB8105…`를 읽었다. Unicode copyright/permission notice를 copies 또는 관련 documentation에 두는 조건과 이름의 광고 사용 제한이 명시돼 있다. MIT/Apache 중 하나만 보존하면 충분하다고 판정하지 않는다.
- memchr COPYING은 두 license 중 하나를 사용할 수 있다고 적고 있다. 이번에는 MIT 원문 `0F96A838…`을 읽었으며 Unlicense 선택을 요구하거나 결정하지 않았다. typenum LICENSE는 dual label이고 별도 MIT 원문을 읽었다.

license root66 파일 목록에는 별도 NOTICE 이름이 관측되지 않았으나 dependency source 전체 하위 NOTICE·vendor bundle의 부재까지 증명하지 않는다. 이번 대상에 제3자 모델·폰트·이미지·JS·운영 script는 포함되지 않았다. 새 package에 dependency 원 소스를 복사하거나 라이선스를 삭제하는 작업0이다.

## 미결정과 인계

프로젝트 root·runner·owned-process의 license field/프로젝트 LICENSE·권리자 승인에 관한 기존 확인 공백은 남는다. 본 C0는 관측된 외부 조건을 기록했으며 프로젝트 license/저작권자를 창작하지 않는다. PM/권리자가 프로젝트 자체 권리와 배포 license·notice 책임을 결정하고 C2 최종 source를 다시 pin해야 공개 source 채택을 판단할 수 있다. `publish=false`나 새 .gitignore 책임 root만으로 재배포 허가를 대체하지 않는다. 공개 push 요청은 없고 실행0이다.

PM은 단일 packages lib+bin/owned-process·resource-runner 비흡수 방향을 추천안으로 인수했고, 중간 tools C1은 기본 생략하며 forced actual gate 뒤 직접 C2 책임분리 후보를 준비하도록 조율했다. 그 actual gate는 여전히 pure/type redgreen 이후 OS/직접CLI별도 배정이다. C0 감사로 빌드/타입/OS 승인을 승격하지 않는다.

스킬 적용: 앞서 읽은 modoo.today `modootoday/software-qa/code-change-review-verification`의 supplied claim/소스 경로·관측·미검증 분리와 `modootoday/spec-authoring/plan-authoring`의 확인되지 않은 권한/측정값을 채우지 않는 원칙을 적용했다. 정확 body/ref hash는 기존 HANDOFF와 통합 계획에 보존한다. 이번에는 cached 원문에 관한 사실만 기록하고 타 언어 policy/검증 도구를 추가하지 않았다.
