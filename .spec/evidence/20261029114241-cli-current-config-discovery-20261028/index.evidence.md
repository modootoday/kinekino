---
id: evidence:kinekino/20261029114241
title: 새 R40 CLI의 구성별97명령 관측·비교표
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 새 R40 CLI의 구성별97명령 관측·비교표

## 실제 범위

새 ED1D normal CLI를 기본 구성과 명시 project/proposal/read-only job 저장소 구성에서 각각 실행했다. capabilities1+명령97의 schema97,각 CLI98회(총196)이며 광고된 실제 작업은 실행하지 않았다. [명령별 비교표](../../../../runtime/verification/source-sdk-r40-cli-status-report-r1/commands.md)와 [CSV](../../../../runtime/verification/source-sdk-r40-cli-status-report-r1/commands.csv)는 같은 CLI 해시의 같은97명령 집합을 결합한다. 명령 발견 분모97을 확인한 것이며 기능 인수/GUI 완성률의 분모를 고정한 것은 아니다.

|구성|supported 표시|unsupported 표시|schema 대조|
|---|---|---|---|
|기본 in-process|9|88|97/97|
|project/proposal/read-only jobs|28|69|97/97;초기화 후 저장소 path/content 동일|

미지원은 해당 구성에서 port/worker/verifier 등이 없다는 의미도 포함하며 전체 미구현 판정이 아니다. metadata 발견으로 정상·오류·취소·기한·저장/재열기 기능 인수나 사용자 품질을 추정하지 않는다. 기존 실제 기능 근거를 아직 명령별 표에 연결하지 않은 행은 미매핑으로 남기고 미구현과 구분했다.

## 입력·producer·원 오류

제품 소스 변경0. SOURCE R40 DB7C9807F9891A80BDF105992E760796D0DA1F71179E63DCE8F296767218E97B,inventory D0428705C89204706EC4E63CE2A9223E83BCFFE254ACAD7E5D22D21C51A9CFC3/2297.1293 Native cli_discovery_inventory no-run은 actualreceipt terminal exit0/13090ms다. Cargo JSON test target/native-proposal-window/executable/freshfalse를 확인하고 fixed-r40/cli-discovery-inventory.exe로 보존했다:5AE90B8AE9938D3640FBDFDD24B3AE06822B905D4AFDA5E63EA3C73C3A52DC61/1923584bytes.

내부 query의 CARGO_BIN_EXE는 debug/player-cli.exe다.1293 이후 SHA를 재확인해1285의 ED1D4899A4825626F25B058CF47BB74856ABB2D54C082021EB0ABAC11DEBBE94/36718592bytes와 같았다. 독립 C2 driver가 이 원 alias·test·index를 guard했고 내부 Rust 시험도 CLI 전후hash를 대조했다. 새 CLI가 아니라 과거 R14 실행파일을 소비한 것이 아니다.

1294 첫 C2 admission은 total120000ms가 원 실행기 상한60초를 넘어서 exit2/invalid input으로 거절됐다. test 실행0·outcome 미생성이며 raw stderr/receipt와 빈 외부 evidence 디렉터리를 보존했다. 원 driver나 child별15초 기한은 수정하지 않고 허용60초 total/5초cleanup으로 새1295를 배정했다. 모델/네트워크/기능 시험의 실패로 바꾸지 않는다.

1295 기본 actual1pass/2filtered·15.53초,1296 저장소 actual1pass/2filtered·14.89초. 각 resource-runner receipt terminal/exit0,각 C2 outer selected_one=true/ownership complete/physical_closed=true/first_error=null/outer exit0·Empty·nonforced·raw EOF다. 원 stdout/stderr/외부receipt는 runtime/verification/source-sdk-r40-discovery-{default-r2,stores}-os1/outer 및 source375-datalab-b0-r1-logs/1293~1296-*에 있다. operator discovery index SHA B8558AC169C4C6A54A18E283CC46D5E751C1F8C8CE83505A28FD65DC018DB10A,proof=false.

원 결과 파일:

- tmp/source-sdk-r40-discovery-default-r2-os1/cli-discovery-32392/inventory.json SHA40C476BF63553E28F53603D07093ACE0FFB213AB8FC612F6853127C4B25D6D8B.
- tmp/source-sdk-r40-discovery-stores-os1/cli-store-discovery-35672/configured-store-inventory.json SHA F1D323BC0D944D1C3857C92C7B0D59F7EBA0D1E6EAE75E29ADBBB9972CB73C91.

각 구성의 raw98 stdout/stderr/process JSON과 초기화 후 저장소 inventory 비교를 보존했다. readonly job 구성의 첫 capability 호출은 event/source/lock 파일을 초기화하므로 무생성이라고 표시하지 않는다. 그 이후97schema 조회에서 저장소 path/content가 동일함을 원 시험이 확인했다.

## Rust 비교 도구

runtime/development/cli-status-report-r1에 std-only Rust 도구를 책임별 inputs/render/main으로 분리했다. 기존 custody json parser를 읽기 재사용하며 원 parser SHA B44583E0E11C344EB2D87C9648B9B7B29B5E882FECF9EC3A28179095C0467A12를 따로 고정했다. 원 custody source/binary를 수정하지 않았다. 새 도구는 CLI SHA·profile·중복/count·같은97명령 집합·허용 status·functional percent=null을 검사하고 functional_evidence_mapping=unmapped를 출력한다. supported 수를 완료율로 바꾸지 않는다.

1297 fmt/1298 all-targets strict/1299 test5/1300 normal producer/1301 실제 생성 모두 actualreceipt terminal exit0. test5는 기존 parser3와 서로 다른 CLI/중복 명령·기능 완료율 오인 거절2다. fixed-r1/cli-status-report.exe ADF284BD315402DE28EA7B6FAE618EB56B9EFB605D319C2FD37C10CB50E0F95F/345600bytes. 독립 native/cli-status-report-r1 target·locked/offline/-j1이며 새 의존성/전역 설정 변경0이다.

도구 자체 source snapshot 시 최초에는 receipt parent,다음에는 tests/assets/packages 디렉터리 전제 미충족으로 capture가 거절됐다. 그 빈 input-r1을 보존하고 빈 디렉터리 전제를 충족한 새 input-r2에5파일을 고정·verify했다. 도구 SOURCE.md B70C7AE49C74645EEE704B6DEE573906CADABB2006969C3BFBD07E8AD7872E6B,inventory1393B5A25B38EE457FE2AFB49284B93D35EE4AFCE7B80910C028CCDCBFC44AB9/5. 외부 재사용 parser는 이5파일에 포함되지 않아 위 별도 pin을 함께 사용한다. snapshot generic canonical 제목은 제품 채택이 아니다.

최종 commands.md SHA00BAE37D8019EA802E9BAA2EC94F3C583B93685114EE612D777E119182C51780,commands.csv SHA5920D61E9898BE278158F66B0E1E26881B0AEE1DE6B5B7E8F48FCE6C6BAA6EEC.1301 stdout은 commands97/default9/store28/functional percent unassessed를 보고했다.

## 마감과 다음 범위

1302 final custody 실제receipt terminal exit0/41151ms,제품2297 content/identity unchanged. 기존 R40 fmt/strict/지정190/정상bins·새CLI 선택3의 원 실행을 유지하며 제품 source 변경 없이 반복했다는 주장은 하지 않는다. C-only cache/tmp·conservative2/CPU2/BelowNormal·heavy1·같은handle terminal 후 직렬을 유지했다.

다음은 기존 실제 기능 정본을 이97행의 host/정상/오류/취소/기한/저장·비적용에 연결하고 실제 executor 공백을 닫는 것이다. 모델/미디어 다운로드·추론·수동GUI·WARP·push0·배포 제외. source375 private overlay/원 실패·WIP/담당 소유권·AI 사용자0/9와 NeedsRepair·GUI 시각 미인수·production source 정책/factory/실제HTTP 미완료는 유지한다.

기독 modootoday/software-qa/test-fixture-and-mock-build SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 실제 fixture/초기화/정리와 발견·기능 구분,code-change-review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 producer/입장 거절·실제 결과·미완료 분리를 적용했다.
