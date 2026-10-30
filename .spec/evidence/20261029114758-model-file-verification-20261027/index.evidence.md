---
id: evidence:kinekino/20261029114758
title: 로컬 모델 무결성 SDK/CLI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 로컬 모델 무결성 SDK/CLI 검증

2026-10-27 중앙 PM. [계약](../../player/application/analysis/model-file-verification.md). SOURCE `4D4B86FE942D9B0661AA205CC813A58E0786CD005DAA3D722C40F5E6E63F8E94`, inventory `5F233D35756505841C618D1B7C33E427974A403829563042983534A32C64F4D5`,2232파일을 source375-model-verify-input-r1에 고정했다. 원375/private overlay·이전 epoch와 구분하며 canonical 승격이 아니다. 원 편집 전6파일은 source375-model-verify-r1-preimage에 보존했다.

587 변경 strict·588/589all-targets strict·590snapshot·591no-run 통과 후592에서4시험/18.84초가 통과했다. 원 fixture4개는 runtime/build-cache/tmp/model-catalog-cli-22976-{1793064534971174200,1793064534973209400,1793064535082981100,1793064552902312300}에 유지하고 runtime/verification/model-verify-cli-datalab-r1/{invalid,cancel-timeout,catalog,matched}에 복사 보존했다. 원 WITNESS의 FileID를 복사본의 새 FileID로 가장하지 않는다. 원 정상CLI SHA `494A5793D8FD65C7EDD0238F75D4D113B6E98F5FBBD1059D6FF01B4FE0BFFE13`, 일반 catalog SHA13ECCE54…,matched catalog SHA9B0DB439AB003913D64E183BF30B3A200AB32B99952BB6DB389D2C622F10BFFB. 실제 모델이 아닌 stub/8MiB 합성 bytes다.

| 목표 상태 | 실제 확인 | 결과 |
|---|---|---|
| 불일치는 성공이 아님 | 원 verify-mismatches PROCESS와 stdout의 NeedsInput·details에 hash_mismatch/size_mismatch/missing | exit3·비강제·원 child wait, 정상 모델 검증으로 계상 안 함 |
| 전체 파일 일치 | exact catalog pin·3파일9bytes의 declared/observed SHA/size·별도 CLI repeat | matched_at_verification·지속 admission=false·runtime/quality=false |
| pin/예산/옵션 경계 | stale catalog/부족한8byte budget/범위 밖timeout | RevisionConflict/ResourceExhausted/InvalidArgument |
| 취소/오류 후 guard 정리 | Rust SDK의8MiB 파일 hash 중40번째 checkpoint에서 Cancelled 또는 주입 Timeout,20번째에 원 catalog/모든 파일 write-open 거절·nested verifier 거절 | 원 오류 보존·반환 뒤 write-open 가능/lock 해제·같은 SDK 재검증 성공 |
| 관측은 영구 승격 아님 | verify 성공 뒤 새 process inspect | integrity=not_verified 유지 |
| 명령 발견/readonly |115개 실제 CLI process receipt, 구성97 schema 일치·원 catalog/stub/CLI 불변 |10supported/87unsupported는 해당 구성의 발견 결과 |

원115개 request/stdout/stderr/PROCESS와 합성 manifest/파일을 보존했다. 실제 wall-clock 만료/hard I/O preemption·CLI 원격 취소 ACK를 위 주입 Timeout으로 입증하지 않는다. SDK checkpoint 취소/Timeout과 동일 port busy 거절만 해당 범위다. 별도 process 사이 heavy 배정은 중앙 PM1개다. 모델 포맷/engine/dtype/정밀도/권리/실제 HF artifact·설치/삭제·실제 추론/품질·GUI는 남는다. 원 파일 hash 일치가 실행/품질 권위를 부여하지 않는다.

모델 및 startup 설정 file adapter에 Windows module gating을 명시했다. 전체 non-Windows/Web build와 다른 기존 Windows module 경계는 별도 미검증이다. 아래 통합은 같은 입력에서 실행했으며 이전2E9의 성공을 승계하지 않았다.

| 후속 로그 | 결과 |
|---|---|
|593/594|Product no-run·고정145/145|
|595/596|독립 Meari no-run·Core18/18|
|597/598/599|Native no-run·26/26·자동OS1/1|
|600|정상 player/player-cli build exit0·두 artifact profile.test=false/fresh=true|
|601|기존 저장/CAS/replay·combined·readonly CLI3/3,25.58초|
|602|fmt --check exit0|
|603|전후2232파일 content/identity 동일·exit0/39331ms·stage 없음|

source375-model-verify-{product,core,native,os}-exact-r1의 네 group-return.txt는 같은 SOURCE4D4…/execution_result=Ok/producer_integrity=Ok를 보유한다. frozen190 계획 SHA082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464·등록/ignored·원 EXE/PDB를 대조했다.600 정상 player SHA `24B825A658C0B4B691C5437C53C5A621413D5C9846322206C7E187164A6BFE56`,CLI `494A5793D8FD65C7EDD0238F75D4D113B6E98F5FBBD1059D6FF01B4FE0BFFE13`.

root HEADd13cbfca140e41863b4d0bd27930be3f57a1123f/Meari0844f6c24caf224617ca8abbb2dd75695a971b99·두 lock·Cargo/rustc/custody SHA가 기존 핀과 일치한다. C native/meari-native/tmp·locked/offline/jobs1·CPU2/BelowNormal·heavy1 유지. 새 의존성 설치·전역 설정 변경·실제 모델 다운로드/추론·push 없음.

기존에 읽은 modoo.today software-qa/code-change-review-verification·test-fixture-and-mock-build 및 ai-quality-plus/agent-run-postcondition-and-replay-check(body SHA61843099B7D54C3D7AC226218A5EE1B6CCDBE36EF84268762DBC33A68FBB58B5,worked-example SHA80C9FE8A8A79DD8E08D84B36483D7742F273E55FB2769D2E6156A1111FDD5C11)를 적용했다. 모델명/명령명만으로 실제 입력을 알 수 없으므로 원 JSON bytes·catalog SHA·코드/SOURCE·CLI와 원 process 결과를 연결했다. 이 시험은 제품 모델 prompt/추론을 전송하지 않았으며 prompt hash를 만들어 채우지 않는다. 재열기는 부작용 없는 같은 stub/선언에서 비교했고 PID/시각/FileID는 실행별 값으로 유지했다. WARP/추론/다운로드/수동GUI/push 보류·기존WIP/소유권/원 실패를 유지한다.
