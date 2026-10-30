---
id: evidence:kinekino/20261029114253
title: 자산 Jobs의 취소 경계·정상 재기동·명시 재시도
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자산 Jobs의 취소 경계·정상 재기동·명시 재시도

2026-10-29 R230 private overlay. [원 CPU Jobs](asset-build-jobs-20261029.md) 후속이며 Core 소유 파일·기존 WIP/실패/보류를 유지했다. AGENTS SHA `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C`. preimage-r229-asset-cancel에 제품/시험 변경 전 파일을 보존했다. 추론·다운로드·수동 GUI·WARP·commit/push 없음.

## 구현과 입력

원 CPU control의 CAS 거절 등 primary error에도 artifact ID·예상 publication SHA·committed/may_exist·partial_cleanup_supported를 보존한다. primary code/message/details를 바꾸지 않고 secondary release error를 별도로 추가할 수 있게 했다. source header의 과거 no-legacy 문구도 현재 선택 CPU 계약과 일치시켰다. process-owned prototype은 계속 미인수다.

R230 inventory2494개 SHA `A8F85AC50F178615E6F7511E608342708D82132A751D2826EC9A9B8F04A21A7F`, SOURCE.md SHA `368245DDD4C5B93AE2EAE2192B35A8AC6139C42BF848C7CF02C6F17B1C7B195C`.3011fmt/3012snapshot/3013native strict/3014default strict/3015lib no-run/3020fmt check 종료0. fixed-r230/native-core의 원 exe SHA `0957AE59C8E00FA40C7DF6D554A7832A599C830D3C3B0A5A7C0C47EB493FBA8A`. 기존 cache PDB/원 producer를 보존했다. 의존성 변경 없음. 기존 Meari deprecation 경고는 소유 범위와 함께 유지했다.

## 실제 시험

3016 strict host path/다른 operation 거절1통과·명시 실행용3ignored. 시험용 wrapper는 실제 identity/deadline/cancellation/prepare/normalize를 원 adapter/control에 위임한다. 원 reserve_publication 직전 또는 원 marker commit 뒤 begin_verification 직전에만 pause하며 자체 CAS·가짜 process exit/EOF를 만들지 않는다.

3017 원 Jobs CAS 전 취소/정상 재기동/수동 attempt2 실제1통과(20.54초). root `runtime/verification/source-sdk-r230-asset-cancel-r1`. pause 중 project/image/mask 쓰기 열기가 거절됐고 data8개가 완성됐으나 publication marker는 없었다. 실제 client.cancel은 Requested, 원 CAS 위임1회는 Cancelled를 반환했다. adapter-error.json에 원 `cancelled`/`Cancellation won before publication`과 artifact/예상 marker SHA, committed=false/may_exist=false/partial_cleanup_supported=false가 남았다. worker 종료 뒤 source 쓰기 열기가 허용됐고8개 부분 파일의 전후 SHA가 같았다.

같은 프로세스에서 원 host를 정상 종료/재기동한 뒤 job은 Cancelled/attempt1 그대로였으며 자동 재실행은 없었다. stale revision 재시도는 전후 모두 거절됐고 명시적 retry만 같은 input/source SHA의 attempt2를 완료했다. attempt1의8파일과 marker 부재는 보존됐다. attempt2 publication SHA `eed0fd8f49159e85e07de8fec08e5a3ccbc6d1e7ecf6e4c081eb11c7ba4086f6`, read-only 저장소 재열기 output도 같았다.

3018 원 발행 후 취소 실제1통과(3.65초). root `runtime/verification/source-sdk-r230-asset-late-cancel-r1`. marker까지9파일이 존재하는 pause에서 cancel은 PublicationInProgress였다. 같은 원 CAS 위임은1회였고 verification/release 후 Completed였다. publication SHA `448937fdb10eb97deafa0adc4317264f059be9ff0b4b7351527c7968139a965a`. 파일 hash/byte·원 input/source/attempt와 재열기/worker 종료가 확인됐다. 늦은 취소로 발행을 되돌리거나 두 번째 CAS를 하지 않았다.

3019 같은 입력 버전의 정상 원 Jobs 실제1통과(3.60초). root `runtime/verification/source-sdk-r230-asset-jobs-r1`, publication SHA `d0a51b699d777dd20f68d3510645d4bd9e4382fd039445b41e3e4ef6d2b99a09`. 선택한 과거 revision 유지·9파일 해시/크기·동일 key 재조회·저장소 재열기·원 worker 종료와 source/project bytes 보존을 다시 확인했다. 세 실제 시험은 서로 다른 fresh root이며 이전 입력의 성공을 합치지 않았다.

## 남은 범위

3021 사후 custody 종료0: 실제2494개 source content/identity unchanged. 검증 후 입력은 R230 snapshot과 동일하다.

이 결과는 기존 CPU Jobs의 선택 취소 경계와 정상 host 재기동/명시 retry다. PNG decode/encode 중간 취소·실제 ACK 유실·비정상 프로세스 종료/OS 재부팅·CloseHandle 실패·권한/disk-full/tamper·모든 CAS fault·다중 frame/batch/한도 및 process-owned CPU 인수로 확대하지 않는다. source는 기존321×137 static PNG와 수동 mask이며 PTS null·품질/색 승인 false다. SDKCLI host 연결/실제 CLI 렌더·artifact inspector/reconciliation·source bundle/편집/undo/projection·GUI·사용자 AI 품질도 남는다. 최신 전체190/정상 bins R221·제작 인수0/5·AI품질0/9·전체 goal을 유지한다.

기존 modoo.today QA fixture/변경 검증/verdict의 실제 원 control과 pause/fixture 구별·첫 오류 보존 원칙을 적용했다. 출처·SHA/적용 판단은 [원 입력 검증](asset-input-admission-20261029.md)에 연결한다.
