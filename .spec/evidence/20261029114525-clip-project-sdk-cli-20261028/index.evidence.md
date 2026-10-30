---
id: evidence:kinekino/20261029114525
title: 숏폼 프로젝트 SDK·CLI 연결 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 숏폼 프로젝트 SDK·CLI 연결 검증

## 최신 완료 체크포인트 — R145

R145는 2324 fmt check,2326 기본 all-targets strict,2327 lib/Jobs/clip_project_cli/CLI no-run,2328 실제시험1개/21CLI(12.82초),2330 Product145/2331 Core18/2332 Native26/2333 OS1,2334 정상2bins,2335 사후2399입력내용·FileID동일을 모두 종료0으로 확인했다. 전체190의 group-return.txt 4개에서 execution_result/producer_integrity=Ok(())와 동일 R145 SOURCE hash를 직접 확인했다. Core/Native/OS는 기존622/624고정producer의 실제 재실행이며 새 컴파일로 표시하지 않는다. R144 첫 strict 실패와 2329 exact runner의 출력 상위디렉터리 누락 경로오류(exit1,시험 전)를 보존했다. 2330은 상위디렉터리를 준비한 별도 재실행이다.

R145 input-r145는2399파일/inventory A239F296F2B3461CE6D6EF34FEE807904CD3B328F6EA88CC2CE5DB8B94ACA8E2,SOURCE.md E596476A5BBC74E3A68A6428BEB69629A6D2286901C29CEF0116A5980573BBB6이다. CLI는R144와동일bytes A66D7F2A6555924179B4E402970EE86320FE41762363B1D788B33D980594D210. 정상GUI는5EA69100A56C932980EACA35F656D8C99E8E5408D9BB1BF874C83A76162045EC. exe/PDB를 mirror/runtime/verification/fixed-r145/clip-cli 및 native-bins에 보존했다. 실제여정증거는 C:/workspaces/com.kinekino.player/runtime/verification/source-sdk-r145-clip-cli-r1/actual 이다.

21CLI는 기본/구성 capability·3명령schema·생성·저장·head/과거조회·중복/stale/foreign source/root주입·endpoint/serve혼합거절을 포함한다. 별도typed저장소에서3메서드의결과를일치시켰으며 최초발행bytes의사후동일성을확인했다. 매CLI가새프로세스에서저장소를재열었다. 이 성공은 합성 미디어 선언의 프로젝트 저장 지원이며 media custody/실제렌더/encoder/전체111명령discovery재인수/전체제품GUI/AI품질승인이 아니다. 다음은 실제 source preflight·도구chain pin·encoder 및 원Jobs renderer 연결이다. 아래 WIP/미실행 표시는 당시 관측으로 보존한다.

R144 실제 CLI 통과/첫 strict 실패:2321 no-run 종료0(189979ms),고정CLI SHA256 A66D7F2A6555924179B4E402970EE86320FE41762363B1D788B33D980594D210/fixed-r144/clip-cli와시험exe/PDB보존.2322 실제시험1개/21CLI호출 통과12.53초,3typed메서드동등·발행1/edit1→발행2/edit3·매CLI재열기·과거조회·중복/stale/foreign/root주입/endpoint/serve혼합거절·원발행bytes보존. evidence C:/workspaces/com.kinekino.player/runtime/verification/source-sdk-r144-clip-cli-r1/actual/verification.json. 원발행SHA62e3333f2f85d18bcf492cffa2b6c500e80484e767a505dc180e51763eebb1d3.2323 strict는새시험의private owned_process_evidence가공개미사용항목live/elapsed_ms를dead_code로만들어2오류실패했다. 외부blitz-paint deprecation1도별도보존. 제품실행실패와구분한다. preimage-r144-clip-test-visibility에시험보존후R145에서기존card_project_cli와동일한pub mod경계로수정했으며allow는추가하지않았다.2324fmt종료0/2325snapshot진행중,재검증전이다.

R144 입력 고정:2320 snapshot 종료0,2399파일/inventory SHA256 85ABBF3F0ECA35F883D940A990C82358781CAB39821676D06076371A922A5469,SOURCE.md SHA256 2AEC5967D427666667D840C240F2132BABC9CA8FEC6FA64315DD2D63A901B998.2321 기본 lib/Jobs/새 clip_project_cli/CLI no-run 실행 중이다. 새 입력을 R143 typed 선언의 컴파일 성공으로 대체하지 않는다.

R144 연결 구현 중: optional ClipProjectStore를 전체 SDK builder에 유지하고 create/inspect/save_revision dispatch·엄격 payload·CAS/지원 context·상세 schema·가용성 표시를 추가했다. 명시 --clip-project-root local 조립을 연결하고 endpoint/serve host 혼합을 거절한다. 발견 명령은111개로 늘었으며 기존8곳의 명령 개수 기대만 갱신했다(episode_capture의108ms는 그대로). 새 Rust actual CLI 시험은 3개 typed 메서드·별도 저장소 동등성·매 호출 재열기·head/edit revision 분리·과거조회·중복/stale/foreign source/root주입 거절·원발행bytes보존을 확인하도록 작성했다. 실제 렌더는 하지 않는다. preimage-r143-clip-dispatch에 기존API/runner/관련시험을 보존했다.2318fmt/2319fmt check 종료0,2320snapshot 진행 중. 컴파일·새실제시험·strict/전체gate는 아직 통과 전이다.

R143 체크포인트 완료:2315 기본 lib/Jobs/CLI no-run 종료0(192820ms),2316 기본 all-targets strict 종료0,2317 사후 custody 종료0. 고정2396 입력의 내용·파일 식별 동일성 확인이다. 새 typed 메서드는 컴파일됐지만 dispatch/schema/root/실제 CLI 연결 전으로 지원은 아직 미완료다. R143 실행시험·전체190·정상bins는 실행하지 않았다. 아래 실행 중·미실행 표시는 해당 당시 기록으로 보존한다.

2026-10-28 중앙 PM · R143 WIP. 소유권과 private overlay를 유지한다. 실제 저장소 기준선은 [R142](clip-project-store-20261028.md)이며 이 문서는 연결 계층의 별도 근거다.

## 현재 변경

ApplicationClient에 create_clip_project/save_clip_project/inspect_clip_project를 추가했다. 공통 RequestContext와 ClipProjectInput/InspectClipProject/StoredClipProject를 사용한다. 저장 expected_revision은 편집 revision이 아닌 발행 head다. 임의 root를 요청 payload에 추가하지 않는다.

기존 client.rs는 mirror/runtime/verification/preimage-r142-clip-sdk-cli/client.rs에 보존했다(SHA256 D3317E8E6EFFC73D09822D8604348C5CBCEFD3C301B28971751A02E1E50D0180). root AGENTS SHA256 40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C를 재확인했다. 별도 프로젝트 celir-sql Cargo 프로세스는 변경하지 않았다.

## 입력과 실행

- 2313-source-sdk-r143-typed-fmt: fmt --all -- --check, 원 receipt terminal/exit_code=0.
- 2314-source-sdk-r143-typed-snapshot: 종료0, 2396파일. inventory SHA256 A012C55AB5643B91F5D1A6142A230F184149DB593643BFF9AE9B44439CF40477.
- input-r143/SOURCE.md SHA256 3A174A84DE10A66E85AD38F89058D5FA044912F63E286B8616AA2DF01C52AE90.
- 2315-source-sdk-r143-typed-no-run: 기본 lib·Jobs·CLI 컴파일 실행 중. 종료 결과 전에는 통과로 계상하지 않는다.

로그 root는 C:/workspaces/com.kinekino.player/runtime/verification/source375-datalab-b0-r1-logs이며 mirror는 C:/workspaces/com.kinekino.player/runtime/development/source-sdk-integration-r1이다. C-only native 캐시/tmp, conservative2 producer를 유지한다.

## 남은 연결과 인수

dispatch와 엄격 payload/CAS context 검증, 모든 generic builder의 optional ClipProjectStore 소유권 이동, schema/capabilities, 명시 --clip-project-root 조립을 연결해야 한다. 기본 미설정은 Unsupported여야 하며 --endpoint와 local root 혼합을 거절한다. CLI 명령의 실제 인수는 생성·조회·저장·과거 revision·재열기·중복/stale CAS/unknown field/경로 주입 거절·원 발행 bytes 보존·typed 동등성을 확인한다. SDK 메서드의 존재만으로 지원을 표시하지 않는다.

render_short·실제 source custody/preflight·encoder·렌더 Jobs·미디어 결과 검증은 이 저장 API와 별도다. R143 strict/실행/전체190/정상bins/사후custody는 아직 미실행이다. 최신 전체 완료 gate는 R142다.

기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 typed API 선언·컴파일·실제 CLI 지원·실제 미디어 성공을 구분한다. WARP·추론·다운로드·수동 GUI·push 보류와 AI 사용자 품질 0/9를 유지한다.
