---
id: plan:kinekino/20261029114000
title: Historical import tool 예약 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Historical import tool 예약 검사

2026-10-19 실행 완료: 중앙 슬롯에서 fmt/test/strictClippy/build 모두 exit0. 일반12통과/1ignored 후 고정bin을 사용한 ignored actual9 인수1통과/실패0. 새 소스 수정 없이 첫 실행을 통과했으며 최초 source inventory 전후 SHA는 e223570e2e23ffb6d51ae2c095791063918b5fbb284fc976fb72907045482689로 동일하다. 고정 EXE SHA는 e00c1726603ef7d0167c6d58deb79e694071044c20a2723de8abe8a504f45e41. 모든 실행기 receipt에 terminal=true, logical_cpus=2, BelowNormal priority=16384를 보존했다. 실행기 소유 heavy는 종료했고 슬롯을 PM에 반환했다. 아래 명령은 실행 당시 절차이며 기존 로그/fixture를 덮어쓰는 재실행 지시가 아니다.

중앙 순서: Meari 자산 gate 반환 후 AI tool. 이 문서는 실행 명령 준비이며 슬롯 시작이나 실행 성공 근거가 아니다. 제품 `application/ai_provenance`는 SDK root 미등록이며 아래 검사에 포함되지 않는다. 신규 추론/download/worker build는 없다.

새 로그 경로: `.spec/evidence/verification/historical-import-2026-10-19-r1/`. 새 고정 실행 파일·정상 출력·scratch 경로: `runtime/quality-review/historical-import-2026-10-19-r1/`. 기존 companion 검증 로그와 binary는 덮어쓰지 않는다.

## 직렬 Rust 검사

작업 디렉터리는 repository root다. 로그 디렉터리를 먼저 생성한다. 기존 전용 실행기를 사용하며 launcher가 descendant 전체에 BelowNormal·최대 2 logical CPU를 적용한다. Cargo는 `-j1 --locked --offline`을 사용한다. 단계별 원 stdout/stderr/terminal receipt를 보존하고 실패하면 다음 단계를 시작하지 않는다. 첫 실패를 재실행 로그로 덮지 않는다.

```powershell
$taskRunner = 'runtime/quality-review/frame-document-cli-r1/final-bin/resource-runner.exe'
$taskEvidence = '.spec/evidence/verification/historical-import-2026-10-19-r1'
New-Item -ItemType Directory -Path $taskEvidence
& $taskRunner "$taskEvidence/fmt.stdout" "$taskEvidence/fmt.stderr" "$taskEvidence/fmt.receipt.json" cargo fmt --manifest-path tools/model-review-bundle/Cargo.toml --all --check
& $taskRunner "$taskEvidence/test.stdout" "$taskEvidence/test.stderr" "$taskEvidence/test.receipt.json" cargo test --manifest-path tools/model-review-bundle/Cargo.toml --all-targets --locked --offline -j1 -- --test-threads=1
& $taskRunner "$taskEvidence/clippy.stdout" "$taskEvidence/clippy.stderr" "$taskEvidence/clippy.receipt.json" cargo clippy --manifest-path tools/model-review-bundle/Cargo.toml --all-targets --locked --offline -j1 -- -D warnings
& $taskRunner "$taskEvidence/build.stdout" "$taskEvidence/build.stderr" "$taskEvidence/build.receipt.json" cargo build --manifest-path tools/model-review-bundle/Cargo.toml --bin player-review-sdk-export --locked --offline -j1
```

실행기 현재 관측 SHA256은 실행 직전 다시 확인한다. 빌드 성공 뒤 export executable을 새 runtime 경로에 복사하고 SHA/bytes 및 변경 source/Cargo.lock identity를 보존한다. 이후 검증은 고정 복사본만 사용한다.

## 고정 CLI 인수

정상 명령은 다음과 같다. `inspect-import` 출력은 source request/candidate 원문을 포함한 도구 전용 envelope이며 제품 admission 응답이 아니다.

```powershell
$taskBin = 'runtime/quality-review/historical-import-2026-10-19-r1/player-review-sdk-export.exe'
& $taskRunner "$taskEvidence/verify.stdout" "$taskEvidence/verify.stderr" "$taskEvidence/verify.receipt.json" $taskBin verify-companion sdk-text-results-r3
& $taskRunner "$taskEvidence/import.stdout" "$taskEvidence/import.stderr" "$taskEvidence/import.receipt.json" $taskBin inspect-import sdk-text-results-r3 gemma270-correction
& $taskRunner "$taskEvidence/missing-item.stdout" "$taskEvidence/missing-item.stderr" "$taskEvidence/missing-item.receipt.json" $taskBin inspect-import sdk-text-results-r3 unknown-item
```

정상은 exit0, item 부재는 exit1이며 대체 항목을 반환하지 않아야 한다. 추가 원본불변 목록은 original source manifests·package·objects·companion의 SHA/bytes를 전후 보존한다. SDK 정규 CLI의 기능 검증으로 표현하지 않는다.

actual9 scratch 부정 인수는 기존 export binary test target의 Rust ignored test `historical_import::acceptance::fixed_binary_actual_nine_negative_acceptance`가 소유한다. 소스는 `tools/model-review-bundle/src/application/historical_import_acceptance.rs`다. 쉘은 변조·복원·기대 exit 정책을 구현하지 않는다.

빌드 후 고정 bin SHA를 관측한 값으로 지정하고 다음 한 테스트를 기존 실행기로 호출한다. SHA 환경변수는 자동 추론값이 아니라 보존할 고정 실행 파일의 실제 관측값이다.

```powershell
$env:HISTORICAL_IMPORT_BIN = (Resolve-Path $taskBin).Path
$env:HISTORICAL_IMPORT_BIN_SHA256 = (Get-FileHash $taskBin -Algorithm SHA256).Hash.ToLowerInvariant()
& $taskRunner "$taskEvidence/acceptance.stdout" "$taskEvidence/acceptance.stderr" "$taskEvidence/acceptance.receipt.json" cargo test --manifest-path tools/model-review-bundle/Cargo.toml --bin player-review-sdk-export --locked --offline -j1 historical_import::acceptance::fixed_binary_actual_nine_negative_acceptance -- --ignored --exact --test-threads=1
```

Rust harness는 고정 binary SHA를 확인하고 기존 read_verified로 actual9를 검증한 뒤 고유 scratch `ai-import-negative-20261019-r1`와 로그 하위 `acceptance/`를 새로 만든다. 기존 디렉터리가 있으면 원로그/자료를 덮어쓰지 않고 실패한다. 원본 package·companion·source/candidate bytes를 복사하고 baseline JSON의 source/candidate 원문을 직접 대조한다. source/candidate 각각 누락·동일 크기 1byte 변조, companion declared→observed 승격·누락의 각 기대 exit1/오류/빈stdout을 검사한다. 매 단계 scratch만 복원하고 마지막 정상 재조회 exit0 및 원본 bytes 동일성과 원manifest pins를 재검증한다. 단계별 stdout/stderr/receipt를 create_new로 보존한다. SDK admission·사람 품질 승인은 항상 false다.

새 source는 fmtcheck만 통과했다. 첫 native gate가 harness의 컴파일/일반 회귀를 검증하며, ignored actual9 인수는 고정 export binary 빌드 이후에 실행한다. 기존 회귀나 문서 준비만으로 이 새 인수를 통과 표시하지 않는다.
## 별도 제품 gate

새 application 모듈 등록·infra reader·proposal/reference 원자 저장·제품 CLI create/inspect/edit/accept/fresh process/project capture는 SDK checkpoint 뒤 별도 소유 검사다. MemoryStore 회귀와 tool 검사 통과로 이를 대체하지 않는다. 모델 정밀도 실측·license admission·사람 품질 승인·apply/render도 별도다. [통합 계획](ai-provenance-integration.md)을 따른다.

## 2026-10-20 공개 재현성 소스 수리

PM 독립 배정으로 일반 unit의 비공개 compile 입력을 제거했다. 변경은 도구 안의 Rust 4파일뿐이다.

- src/application/historical_import.rs: private companion include_str 대신 기존 synthetic_sample 호출. 원문 보존·재hash·다른 source/candidate 거부 assertion을 유지했다.
- src/application/historical_companion.rs: cfg(test) synthetic_sample만 추가하여 기존 합성 fixture를 같은 derive 경로로 재사용한다. production admission/pins/명령은 변경하지 않았다.
- src/application/historical_companion/tests.rs: 기존 fixture 가시성만 pub(super)로 조정했다. 자료·회귀 assertion은 변경하지 않았다.
- src/application/historical_import_acceptance.rs: repository 위치의 암묵 추론 대신 명시 HISTORICAL_IMPORT_WORKSPACE 절대경로를 runtime에 받는다. 실제자료 고정 package/source pins·고정 EXE SHA·ignored 여부·부정/복원/원문불변 assertion은 유지했다. 실제자료를 공개 fixture로 복사하지 않았다.

정적 검토: 도구 src 전체에 include_str!/include_bytes!/CARGO_MANIFEST_DIR가 남아 있지 않음을 rg로 확인했고, 직접 rustfmt --edition 2024 및 --check는 exit0이다. Cargo/build/추론/download/stage/commit은 실행하지 않았다. 기존 2026-10-19 통과는 수리 전 source 근거이며 이번 변경의 compile/test/strictClippy 통과로 사용하지 않는다. 공유 root/SDK/UI/Meari/manifest/ignore/index는 변경하지 않았다.

다음 중앙 슬롯의 최소 검증: private .spec 파일이 없는 임시 공개 source snapshot에서 locked offline -j1 test --all-targets와 strictClippy, exportbin build를 수행한다. 실제자료가 없는 일반 회귀는 통과하고 실제 actual9 인수는 계속 ignored여야 한다. 별도 실제 인수에는 HISTORICAL_IMPORT_WORKSPACE/HISTORICAL_IMPORT_BIN/HISTORICAL_IMPORT_BIN_SHA256를 명시한다. 원로그와 scratch를 덮지 않도록 새로운 소유 workspace에서 실행하며 원본 자료·source pins는 유지한다. 실제관측 gate와 공개 합성 회귀의 범위를 합치지 않는다.

### 공개 snapshot 구성과 실행 문맥 점검

필수 source closure는 tools/model-review-bundle의 Cargo.toml/Cargo.lock/src 전체와 packages/media-contracts 및 packages/media-file-publication 각각의 Cargo.toml/src 전체다. tool 자체 workspace와 두 명시 path dependency만 소비한다. root Cargo·Meari·UI·.spec·AGENTS·실제 모델/미디어를 snapshot의 compile 입력으로 복사하지 않는다. 도구 src 및 두 package src의 정적 검색에 include_str/include_bytes/env! 외부 compile 입력은 없었다. README는 사용 설명으로 복사할 수 있지만 compile 필수는 아니다. snapshot에 별도 root workspace manifest를 합성하거나 기존 manifest/lock을 고치지 않는다.

현 tool manifest SHA는 2d67a75f24341b72c3d2369c36a658efdc6d0274ac9cb0e23a145f9d09d49ee9, tool lock은 c69689019eb2c62a2fc1567901b7817775cf8f3b6fd28ebf67bd6bde228b9dcd다. media-contracts manifest는7aeeb6412eaca8d6301762c047f5d43dd47333be0825b9ccd422165494dbd047, publication manifest는02c78bda4e3401ec89b9d8093145cef7a6dff48231f90949e039e926f4be0e4f다. snapshot 생성 시 src inventory와 함께 다시 기록한다.

lock의 외부22개 crate version/license 선언을 로컬 registry Cargo.toml에서 읽었다. 대부분 MIT OR Apache-2.0, generic-array/zmij는MIT, memchr는Unlicense OR MIT, version_check는원문MIT/Apache-2.0, unicode-ident는(MIT OR Apache-2.0) AND Unicode-3.0이다. unicode-ident 로컬 원본에는 LICENSE-APACHE/LICENSE-MIT/LICENSE-UNICODE가 존재한다. lock/checksum과 registry cache는 기존 offline 입력이지 공개 fixture에 vendor 복사한 자료가 아니다. 배포 시 고지/동봉 범위 및 자체 소스 license 선택은 PM 공개정책의 별도 확인 항목이다. 현재 root/tool/두 local package에서 별도 LICENSE 파일이나 manifest license 선언을 발견하지 못했으며 이를 승인으로 보충하지 않았다. 모델 license나HF원문을 공개 fixture로 가져오지 않는다.

재사용 후보 target은 기존 tools/model-review-bundle/target이다. 저장된 .rustc_info는rustc1.99.0(b940084d7, x86_64-pc-windows-msvc) 관측이고 현재 compiler를 새로 실행해 확인한 값은 아니다. 이 cache의 export EXE는수리전파일이므로이번검증통과bin으로쓰지않는다. 배정슬롯에서snapshot source를입력으로 Cargo가변경/fingerprint를판단하게하고,--target-dir의절대경로로기존cache만재사용한다. root/native/Meari target/cache를섞지않고동시cargo실행도금지한다. cache재사용성은nativegate전에는미검증이다. 최초test/strict/build는공개snapshot에대해수행하고,실제actual9/새추론/download는추가하지않는다.

### 2026-10-20 공개 snapshot 실제 gate 완료

PM이 Core 반환 후 단일 실행슬롯을 명시배정했다. 공통 source는 committed6d9f666837bac4b619c57cbd99663a42d8dcb67f의media-contracts/publication만 git archive로 추출하고 live 공통package는 복사하지 않았다. 수리후 toolmanifest/lock/src를 더했다. 최초 repository 하위 snapshot의 fmt는 바깥 root workspace 자동 발견으로exit1/220ms였다. 원로그와 snapshot을 보존했고 manifest/root소스수정 없이동일내용을저장소밖%USERPROFILE%/AppData/Local/Temp/source-ai-public-20261020-r1로복사했다.

바깥 snapshot에서fmt327ms/test16170ms(기본2+export10=12pass/실패0/ignored1)/strictClippy11256ms/exportbinbuild6132ms가모두exit0이다. 기존tooltarget를절대--target-dir로재사용했고root/Mearicache는사용하지않았다. 모든receipt는terminal=true,logical_cpus=2,priority16384,descendants=true다. snapshot직하에는packages/tools만있고.spec/runtime/AGENTS는없다. ignoredactual9는실행하지않았다. 실제자료나license를새공개fixture로복사하지않았다.

증거root는runtime/quality-review/model-review-public-20261020-r1이다. 최초실패fmt.*와재검증fmt-outside.*를분리보존했다. live src전후inventory SHA b6ac95c69ecf6d366c655428119414abe53786cd70ca78467bca34985fd2f60b,바깥snapshot전후inventory SHA b98a0ffeb9b9652ddd1c56903fc94a49f8aa53ae418967e2c48404f99cd5d12b로각각동일하다. repaired-source-pairs.json이수리4파일과snapshot4파일의동일해시를별도보존한다. path가다른두inventory해시를같은목록이라고표현하지않는다.

고정bin은final-bin/player-review-sdk-export.exe이며SHA0da256ef1c6656e4e60f6704d8a04c6c2d3628f71928536bf097ff338d9327a0이다. 이번bin으로actual9/제품SDK/OS실행을추가하지않았다. source/manifest/lock의내용수리없이배치위치만수정후통과했다. 전체목표/실제정밀도/라이선스선택/배포/사람품질은미완료이며tool공개소스일반회귀성공과구별한다. 슬롯은PM에즉시반환했고stage/commit/push는0이다.
