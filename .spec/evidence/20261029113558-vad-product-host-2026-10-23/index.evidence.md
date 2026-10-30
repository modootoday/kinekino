---
id: evidence:kinekino/20261029113558
title: VAD 제품 host 정상 SDK·CLI gate
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VAD 제품 host 정상 SDK·CLI gate

2026-10-23 중앙 단일 heavy 배정으로 root `1897b696e936f72768ad44ef9f774d35fbcebb96`, Meari `0b9ea031ce5c08a5bf99ed9852b668cf2bfce038`, lock `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2` 위에서 실행했다. 근거는 `runtime/verification/vad-product-host-r1`이다. 원 S/T·conservative2 CPU2/BelowNormal/descendant Job·offline/locked/j1/thread1을 사용했고 시작 시 RAM19.87GiB/C123.88GiB/GPU6219MiB·공유 writer0을 확인했다.

## 확정 결과와 한계

최종 전체 fmt와 root/media-vad/protocol/executor default all-target strict Clippy가 통과했다. Cargo test --no-run으로 지정 target을 빌드하고 고정 BIN을 직접 exact 실행했다. 실제 등록819(ignored11) 중 중복 source 소비자를 제외한73개가 각각1pass·terminal·exit0이다. root44/CLI bin17/vad_jobs5/parent configuration4/일반 CLI3이며 selected ignored4다. 미선택746개와 root 등록705개 전체를 통과로 표시하지 않는다. 기존 형제 target에서 반복 편입된 동일 소스 시험은 중복 실행하지 않았다.

일반 CLI는 Cargo `test=false` artifact `05-player-cli-normal.exe`, SHA256 `B5446F895DBDA279D7AA4BE291B67F2735E511BBD06AC361CEB5EFAFC7F48A36`, 31841792bytes다. bin libtest와 별도로 고정했고 consumer07에 경로/SHA/bytes를 주입했다. 일반 CLI3은 기본 ready→EOF→job dependency receipt와 pipe/sdk joined·idle, partial/duplicate/relative 주소의 InvalidArgument, 고정 제품 record 부재의 ready 전 NeedsInput을 검증했다. 거부 사례는 job root 미생성을 검사한다. 실제 정상 CLI 하위 프로세스6개의 원 receipt는 family Empty/forced false/stdin EOF requested이며 expected child exit0/2/3과 raw stdout/stderr를 보존한다.

기존 ignored vad_jobs CLI 회귀1은 원 synthetic release fixture·로컬 SDK host·실제 normal CLI client를 소비한다. 존재하지 않는 worker 경로를 유지하며 원 readmission 실패·원 source pin·취소/재시도·host closeout을 검사한다. ORT/model/생산 admission을 실행하거나 승인하지 않는다. 새 입력 시험의 서로 다른 WAV ID/pin 검증도 실제 모델 작업 audio2 인수가 아니다.

정상 metadata 패키지356·lock 불변, root의 media-vad/protocol/executor3만 dev→normal로 바뀌었고 다른 package·resolve node·feature 변경은 없다. ORT 활성 node0이다. worker는 dev dependency로 남으며 허용 Cargo의 dev library/build-script 검사는 별도 worker 실행 파일 빌드·engine 실행과 구별한다. 이 gate의 worker executable/ORT/model/inference/download/network/GUI 실행은0이다.

## 소스와 최소 수리

최종 `runtime/development/vad-product-host-final-r1/final-source22.json` SHA256 `A69659403BB0A724EC49D7DBD7EB7CA1C31FA12C85B844F7CF7F4BF201F8988E`, `before22.json` `71657966303BF91B78D19A748A82CB52B987ED2223DC5B9EA54C05B0E1EF5F9F`다. 기존13/new9이며 root 미변경 상태의 정확 preimage를 보존한다. 원 source21 `5EE51BAD305E9018746BAC5FCD692A9328FAB1EC57DD0D23D1FD9EBFB087399F`에 fixedinputs consumer-r2 override1과 CLI-r2 new1을 명시 결합했다. source21/CLI원1/수리 전 bytes를 덮지 않았다. 제품 body 변경 없이 시험 consumer2의 필요 수리만 포함한다.

첫 strict03은 새 root test mount에서 기존 fixture public ID3과 legacy 조립 함수의 unused 진단으로 child exit101이었다. 별도 fixedinputs consumer-r2 시험1은 public lookup3의 실제 원 request 대응, legacy 조립기의 원 evidence lease 보유·중복 거부·해제 후 재개방을 검사한다. 가짜 reference·lint 억제는 없다. strict04 및 최종09는 통과했다. PM의 CLI hash 미사용 정적 지적은 shared helper의 `super::super::hash` 실제 호출과 strict 통과에 따라 철회됐으며 함수를 삭제하지 않았다.

일반 CLI 첫 시도는 uppercase 외부 SHA와 lowercase Rust digest 비교에서 spawn 전에 실패했다. 같은 digest의 lowercase 주입으로 정규화했으며 소스/조건은 바꾸지 않았다. 두 번째는 기본 fixture가 기존 job root를 준비하지 않아 실제 CLI가 NotFound4로 종료했다. 원 raw/terminal/EOF receipt를 보존하고 별도 CLI-r2에서 기본 긍정 사례에만 ordinary job root를 준비했다. 거부 사례의 root 미생성 assertion은 유지했다. 이는 기존 JobFiles/immutable path 전제를 따르는 시험 준비 수리이며 CLI 정책 완화가 아니다. 소비자07 새 BIN·실제11 목록·최종 fmt/strict·일반 CLI3을 다시 회수했다.

normal05/root 및 sibling06은 해당 source closure의 고정 결과이며 CLI test-only leaf 수리로 불필요한 재빌드를 하지 않았다. consumer07은 새 leaf를 실제 빌드했다. 각 package/target/kind/features/profile/fresh/source-build/path/SHA/bytes는 fixed manifest에 있다. runner receipt96개는 전부 terminal, nonzero3은 위 첫 실패이며 삭제·통과 승계하지 않는다.

## 보호와 후속

root1267/library479/원 raw77/SDK·download root+S28/index·AGENTS·library docs6 및 S snapshot3174의 비입력3163은 재해시 차이0이다. final22와 실행 S의 bytes도 일치했다. fixed7와 normal05 고정 복사본은 불변이다. root index/commit은 PM 소유, push0이며 외부 untracked WIP를 삭제·채택하거나 ignore 경계를 바꾸지 않았다.

BIN731 재해시에서 변경7/new4를 관측했다. root libtest1·normal CLI/GUI의 top/deps 별칭4는 Cargo producer와 일치하며 CLI/GUI 쌍은 같은 C volume·FileID·SHA다. 신규4는 고정 test artifact 원본이다. 그러나 `native/debug/build/media-vad-worker-470928e8aa35bb24/`의 `build_script_build-470928e8aa35bb24.exe`와 `build-script-build.exe` 2개는 before `0C80D1E42CE9D613F4FF0E1E095661A3EDA973AB34430E3E369D2DD709BF95D3`→after `2D2094C73DC9FA25162BFA1B2466F7C5C5E0390ED79E637914F6E86D6BC6CCAA`/140800bytes로 바뀌었고 원 producer 귀속이 미해결이다. 두 경로끼리는 같은 FileID지만 Cargo06/07의 cc2f artifact는 FileID/SHA가 달라 해당 producer로 승계하지 않는다. `cache-unresolved2.json`, `alias-identity.json`, raw fingerprint·시간 및 현재 bytes `artifacts/observed-check-buildscript.exe`를 보존했다. pre-gate bytes의 별도 백업은 없으며 원 SHA snapshot만 있으므로 복구 가능을 주장하지 않는다.

따라서 `protection-final-binaries.json`의 all_binaries_protected는 false이고 sourceactual73와 공유 캐시 미해결2를 분리한다. 중앙 지시로 추가 JSON strict 관측 제안은 실행하지 않았고 재빌드·복구·삭제0이다. 해당 캐시는 PM 판단·새 명시 admission 전 재사용 가능으로 표시하지 않는다. 원 runner96은 모두 terminal이고 일반 CLI6 원 family Empty/EOF receipt를 보존했다. 새 앱/WSL 종료 없이 Windows native gate를 마쳤으며 명시 반환은 slot-return.json을 따른다.

modoo.today `modootoday/software-qa/code-change-review-verification` 본문 SHA256 `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`, `references/verdict-table.md` `9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9`를 실제 읽고 source/타입/관측/권한 verdict를 구별했다. 정적 미발견과 작성 수를 actual로 승계하지 않았고 첫 실패·원 owner·진단을 보존한 뒤 최소 수리를 별도 동결했다.

독립 승인 release/native record와 실제 설치 bundle, 서로 다른 audio/destination2의 normal CLI submit/watch/get·실제 결과, native ABI/loaded closure·Windows 기준, no-overwrite·취소·재시도·owner/EOF 실제 인수는 필수 후속이다. 현재 PRODUCT_RELEASE=None 거부를 제품 활성화 완료로 대체하지 않는다. 전체 Media/AI 범위와 최종 사용자 모델 품질 검토는 미완료다. [현재 목표](../../../plans/player/ai/media-current-goal.md)와 [제품 계약](../../player/application/analysis/vad-product-host.md)을 따른다.
