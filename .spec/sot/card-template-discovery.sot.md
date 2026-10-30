---
id: sot:kinekino/card-template-discovery
title: 공통 카드 템플릿 조회
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공통 카드 템플릿 조회

R115 [실제 검증](../../../verification/application/card-template-discovery-20261028.md) 완료: 명시host4개 catalog/typedSDK동등성·원 카드 Jobs 여정과 기본CLI 미가용 응답,양strict/fmt·지정190/정상2bins·사후2360입력동일. 아래 구현 중/미실행은 당시 기록이며 원 R114 첫 실패도 보존한다.

첫 실패/수정: R114 snapshot2360파일 inventory86D22F1B6D55C16A0F57CD298BAE477A228949342557A133796CCD3153B40A6C,SOURCE.md07E247BC64B4A8C85CFE91BCB601719BF4F89B99FEF88633A9FC9C24DAA12D9D 보존. 2049card no-run은 E0277 두 건으로 실패했다. capabilities 보강의 기존 map 클로저가 Value를 반환하므로 새 port의 오류를 ?로 전파할 수 없었다. 실패 api_mod.rs를 preimage-r114-catalog-compile-failure에 보존하고 R115에서 and_then/Ok(value)로 수정했다. 자산 오류를 무시하거나 지원 목록으로 바꾸지 않는다. 2050fmt종료0,2051새 snapshot 진행 중. 실제 시험 통과로 아직 계상하지 않는다.

2026-10-28 R114 구현 중. 기존 capabilities 명령의 card_templates 필드와 ApplicationClient::card_template_catalog를 같은 공통 Rust 계약으로 제공한다. 새 명령을 추가하지 않아 기본103 카탈로그 수는 유지한다. DTO는 schema_version/backend_available/templates/review_artifact_only/quality_approved/platform_requirements_verified다. templates는 canvas ID/version/선언SHA/출력크기·공통 font SHA/라이선스 참조·한계를 가진 BundleTemplatePin 목록이다.

CardTemplateCatalogPort는 application에 두고 실제 CardBundleFiles adapter를 명시 카드 host가 주입한다. port가 없는 기본 SDK는 backend_available=false,빈 templates를 반환한다. 명시 host는 기존 제안1종과 새 프리셋3종의 실제 자산 admission을 거쳐 반환한다. 자산 검증 실패는 capabilities의 원 오류로 반환하며 지원 가능 목록으로 바꾸지 않는다. 저장 store 등록만으로 renderer 가용성을 추정하지 않는다.

카탈로그는 파일 저장·추론·다운로드·렌더링·품질 승인을 수행하지 않는다. quality_approved/platform_requirements_verified=false와 review_artifact_only=true를 명시한다. UI/CLI는 동일 DTO를 읽고 선택 canvas를 편집 프로젝트에 저장한다. admission은 다시 exact pin을 확인하므로 조회 이후 입력 변경을 묵인하지 않는다.

원 소스6파일은 preimage-r113-template-catalog에 보존했다. 2047cargo fmt 종료0,2048snapshot 진행 중이다. 실제 CLI capabilities와 typed SDK 조회의 동등성 검증을 기존 카드 Jobs 여정에 추가했으며 아직 실행하지 않았다. R114 컴파일/strict/실제CLI/회귀/bins/custody 미완료,최신 완료 통합gate R113. 빈 템플릿/역할별/페이지번호·GUI/사용자품질은 남는다. 원 AI9 NeedsRepair/품질승인0/9,소유권/WIP/private overlay·보류 유지.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 구체 renderer의 실제 가용성과 단순 카탈로그 선언을 구분한다.
