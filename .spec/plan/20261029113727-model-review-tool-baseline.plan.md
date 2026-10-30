---
id: plan:kinekino/20261029113727
title: 모델 검토 도구 전체 기준선 인계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 모델 검토 도구 전체 기준선 인계

PM 직렬 stage 배정 전의 명시 경로 제안이다. 현재 git ls-files의 이 tool 결과는 비어 있으며, 수리4파일만 stage해 독립 tool 전체 재현성 commit이라고 표현하지 않는다. stage/commit/push는 실행하지 않았다. 아래20개 이외 target/runtime/.spec/개인 자료는 공개 목록이 아니다.

## 명시20경로

```text
tools/model-review-bundle/Cargo.toml
tools/model-review-bundle/Cargo.lock
tools/model-review-bundle/README.md
tools/model-review-bundle/src/main.rs
tools/model-review-bundle/src/application.rs
tools/model-review-bundle/src/domain.rs
tools/model-review-bundle/src/infrastructure.rs
tools/model-review-bundle/src/presentation.rs
tools/model-review-bundle/src/export_main.rs
tools/model-review-bundle/src/application/pinned_text_sources.rs
tools/model-review-bundle/src/application/sdk_export.rs
tools/model-review-bundle/src/application/historical_companion.rs
tools/model-review-bundle/src/application/historical_companion/tests.rs
tools/model-review-bundle/src/application/historical_import.rs
tools/model-review-bundle/src/application/historical_import_acceptance.rs
tools/model-review-bundle/src/domain/historical_evidence.rs
tools/model-review-bundle/src/infrastructure/content_digest.rs
tools/model-review-bundle/src/infrastructure/review_assets.rs
tools/model-review-bundle/src/infrastructure/historical_files.rs
tools/model-review-bundle/src/infrastructure/historical_publication.rs
```

## 책임과 검증 구분

| 묶음 | 책임 | 근거·제한 |
|---|---|---|
| main/application/domain/infrastructure/presentation 5개 | 고정 과거 원문을 보존한 read-only bundle 생성과 loopback 표시 | 일반2회귀·all-target compile/Clippy 범위. 이번 public gate에서 실제 build/serve·브라우저를 실행하지 않음 |
| export_main/pinned_text_sources/sdk_export | 별도 exporter 조립·고정 source 선택·SDK review package 구성 | 기존 exporter/asset 회귀 및 compile/strict. 이번 actual package 재생성은 안 함 |
| historical_evidence/files/publication/content_digest/review_assets | 증거 상태·bounded read/hash·공통 publisher adapter·기존 객체 게시 | tool 일반 회귀를 포함한 test/strict 범위이며 별도 root SDK/package 전체 회귀는 아님 |
| 수리4: historical_companion/그 tests/historical_import/historical_import_acceptance | 합성 fixture 재사용으로 private compile include 제거, actual workspace의 명시 runtime 분리 | src의 assertion/pins 유지. 일반12pass·ignored1, fmt/strict/exportbin build 성공. 실제actual9는 이번엔 안 함 |
| Cargo manifest/lock | 두 명시 local path dependency와 고정 registry 입력 | manifest/lock 변경 없이 snapshot 통과. 후속 live dependency 등록을 자동 포함했다고 하지 않음 |
| README | 사용·역사적 검증·새 public/runtime 구분 | public gate 후 문서만 보완. 현재 README SHA와 gate 당시 Rust/manifest 핀을 별도 기록 |

snapshot의 공통 source는 **6d9f666837bac4b619c57cbd99663a42d8dcb67f**의 media-contracts/media-file-publication이다. 조회 당시 현재 root **1fd8ca1**과는 별개 pin이다. 해당 package의 live 미완성 source를 복사하지 않았다. tool20경로 기준선만 commit해도 두 package 의존의 필요성이 사라지는 것은 아니다. 현 root 또는 후속 media-contracts 등록과 함께 compile할 경우 정확 pin으로 별도 인수해야 한다.

## 최종 identity와 반환

근거는 runtime/quality-review/model-review-public-20261020-r1/evidence다. live source inventory와 snapshot source의 수리4쌍은 repaired-source-pairs.json으로 동일성을 대조했다. live-tool-before/live-tool-after/baseline-source-inventory SHA는 b6ac95c69ecf6d366c655428119414abe53786cd70ca78467bca34985fd2f60b로 같다. snapshot before/after는 경로가 다른 별도 목록이며 b98a0ffeb9b9652ddd1c56903fc94a49f8aa53ae418967e2c48404f99cd5d12b로 동일하다.

현재 Cargo.toml SHA2d67a75f24341b72c3d2369c36a658efdc6d0274ac9cb0e23a145f9d09d49ee9, Cargo.lock SHAc69689019eb2c62a2fc1567901b7817775cf8f3b6fd28ebf67bd6bde228b9dcd, README SHA4755a0a6e434bc8653b1cb3946a626a892926e3dcdad2467692a8b42728d6906이다. baseline-metadata-pins.json에 저장했다. README는 snapshot compile 입력이 아니었다.

최종 export EXE SHA0da256ef1c6656e4e60f6704d8a04c6c2d3628f71928536bf097ff338d9327a0. 최초 repo하위fmt의 outer-workspace 오류는 보존했고, 저장소밖 동일 snapshot의 fmt/test/strict/build는 모두 terminal0이다. 슬롯을 명시 반환했으며 후속 heavy는 PM이 SDK에 넘겼다. AI 소유 검사는 종료했고 다른 담당 PID는 중복 조회하지 않는다.

commit 설명 제안: `검증된 Rust 모델 근거 도구 전체 기준선 추가; 실제자료는 런타임 입력으로 분리`. 본문에는 공개12회귀/strict/bin의 정확 dependency pin, actual9 미재실행, SDK/GUI/모델품질/배포 라이선스 미완료를 적는다. 자체 license 선택은 사용자의 실제 배포 결정 잔여이며 새 license를 임의 추가하지 않았다. [실행 정본/원실패](historical-import-tool-gate.md)와 [제품 통합 잔여](ai-provenance-integration.md)를 따른다.

## 커밋과 index 반환

PM이 SDK 반환 뒤 공유 index를 명시배정해, staged 시작0과20경로 rawSHA불변·민감/비공개 패턴 경계를 재확인했다. src17 pre-stage목록은기존b6ac95c69ecf6d366c655428119414abe53786cd70ca78467bca34985fd2f60b와동일,manifest/lock/README핀도위목록과같았다. exact20만명시git add하고cached diff --check를통과한뒤2efd2f1fd8bac240f47542f33084990a0f8b93c7로커밋했다. 초기CRLF→LF정규화안내는manifest의Git text처리이며source내용수리가아니다. git show가20file/1735insertions를확인했고직후cached목록및tool status는빈결과였다.

커밋본문은고정검증dep6d9f666과SDKsnapshot1fd8ca1을분리하고,일반12검사·actual9미재실행/ignored·자체license선택/모델품질/배포/전체제품미완료를명시했다. command-local maintenance.auto=false/gc.auto=0을사용했고push/추가Cargo/추론/download/workerbuild는0이다. evidence의baseline-commit.txt/committed-exact-paths.txt/staged-tool-blobs.txt/index-return.txt가근거다. 다른root/계약/UI/ignore/Meari/자산은stage하지않았다. 공유index소유를PM에명시반환하며중복재조회하지않는다.
