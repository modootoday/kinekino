---
id: evidence:kinekino/20261029114542
title: 기존 파일·실제 ffprobe 회귀 — DATALAB / SDK R37
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기존 파일·실제 ffprobe 회귀 — DATALAB / SDK R37

## 범위와 입력

요청 준비를 원 core run 안으로 옮긴 R34/R37 이후 기존 media_download_owned와 real_probe의 원 SDK/JobsCore/저장·취소 계약을 새 호스트에서 실행했다. source 코드 변경은 없다. 원 D 자료/과거 실패/과거 FileID는 보존했다. 원본 핸들·실제 ffprobe 기술 검증이며 HTTP/provider,전체 CLI·GUI·AI 품질 인수가 아니다.

사용 소스는 private SDK R37 SOURCE.md A00BDA38F2D285593E12F0A5D339629F9847FFCAD2C83832353862F262C10C0D,inventory97A425FA3C624B898E6C4C29A286C1CEC0CB4A56457A9923683AD6CAC3660C0C/2296이다. 사전 verify-current session57757 terminal exit0/2296 content·identity 동일을 확인했다. root/source375 canonical 채택이 아니다.

원 portrait는0FD9B2DE74774B4D2DCEA9F9B66466899B44AFF1300B121D548E10AF4C386483/932830bytes. 원 manifest runtime/development/download-real-probe-r1/input-portrait-r1.json C90B2C168EA25C78889531008D89DFED79C7CD60CB3EF6AC733B23B8B1275C25는 D 경로 그대로 보존했다. 새 runtime/verification/source-sdk-legacy-file-probe-r1/input-portrait-c-only.json SHA0C93F32871262CE107DFF8A504C42D6CCCF32939DED4B4766685D03EA063E888은 실제 C 경로와 원 provenance/권리 경계를 명시한 별도 입력이다. 원 미디어를 생성/수정/다운로드하지 않았다.

## 최소 의존성과 실제 producer

옛 Chocolatey ffprobe와 과거 synthetic exe는 현 호스트 경로에 없었다. 이미 보존된 FFmpeg6.1 archive SHA1DA2AA73766C9B1A04E7A7D694470C0AC2B6D9BA99E1A7B879780CE177AA07CE에서 ffprobe.exe 한 파일만 프로젝트 toolchains에 추출했다.1235 실제receipt terminal exit0. 원 tool pin EFB014C8F39091C90F3616154FE20AB8470F5F943F6A48EC4FD30C479C7DD697/82620928bytes와 일치한다. 원 DEPENDENCY-RECEIPT.json은 유지하고 FFPROBE-LOCAL-RECEIPT.json을 별도 추가했다. 사용자 승인 최소 의존성 범위이며 network download0/global PATH 변경0이다.

원 tests/support/download_probe_child.rs SHA AFFC9C32A0EEDA4A99C38756B8C787D36BFD05716183F2403D49B09B4AEE0CEA를 현 stable MSVC rustc로1236 한 번 빌드했다. fixed-legacy-probe-r1/download-probe-normal.exe를 만들고 별도 held 이름으로 복사했다. 각 D492E6F9AACA0EB1370B0AC8C21F35B60745646CD2F4A55EC75458372DB551A5/209408bytes다. 과거8A45/0A3A producer로 표시하지 않는다. 합성 원 stdin/seek/EOF·수명 시험용이며 실제 decoder가 아니다.

1237 cargo --locked --offline -j1 -p kinekino-player --test media_download_owned --features source-adapter-test-support --no-run의 실제receipt terminal exit0. compiler JSON target/test media_download_owned,feature와 executable debug/deps/media_download_owned-7a31306841757d12.exe를 확인한 뒤 fixed-legacy-probe-r1/media-download-owned.exe로 보존했다. SHA C22FFF99FB8973BC05FCA434192ED7AFD76D30174AD6294ABCFF212EE00E2306/3600896bytes. 이 명령에서 생성된 부수 bin alias는 정상 제품 bins gate나 기존 fixed CLI를 대체하지 않는다.

operator index runtime/verification/source-sdk-legacy-file-probe-r1/index.json SHA2C0DCACEF448F74F6B94C7AAC22B8206FFD93C2F27FD0E906BC2C088F88AC718,proof=false. 실제 C2 driver는 기존 fixed-r2 7E7AD25832F177143A3D0D3376EEFC317AE1EF38356E1DC6606B5AECCD5F5F44/924672bytes다. 각 고정 test/child/index를 guard하고45s total/5s cleanup으로 exact case를 독립 실행했다.

## 실제 결과

|실행|원 case·결과|입증 경계|
|---|---|---|
|1238|real_ffprobe_original_file_single_publication 1pass/30filtered|원 regular-file fd→capabilities/inspect→동일 SHA·932830bytes 단일 발행·Completed1|
|1239|real_ffprobe_truncated_original_is_not_published 1pass/30filtered|원 첫16bytes를 실제 ffprobe가거절;moov atom not found/invalid fd input;미발행|
|1240|real_original_source_cancel_closes_without_publication 1pass/30filtered|원 source 협력취소→Cancelled/미발행;probe 시작 전|
|1241|external_cancelled_error_with_user_cancel_remains_failed 1pass/30filtered|외부 Cancelled-code와 사용자취소 경합에도 원 Failed/Original source cancelled 보존;probe 시작 전|
|1242|local_cancel_cleanup_failure_cannot_acknowledge_cancel 1pass/30filtered|원 close 오류를 Failed/Original source close failed로 보존;취소 ACK로 승격하지 않음;probe 시작 전|
|1243|original_stdin_seek_eof_and_single_core_publication 1pass/30filtered|새 synthetic normal 원stdin/seek/EOF와 단일 발행|
|1244|rejected_candidate_preserves_existing_destination 1pass/30filtered|synthetic 검증 거절 시 기존 목적지 보존|
|1245|dropped_client_retains_original_child_lease_until_cancel_close 1pass/30filtered|원 child/lease 보유·취소·물리정리|
|1246|run_unwind_retains_original_child_lease_until_same_owner_close 1pass/30filtered|원 unwind 뒤 같은 owner/child 정리|

각 실제resource-runner receipt terminal exit0와 outer selected_one=true/ownership complete/physical_closed=true/first_error=null을 확인했다. outer process는 exit0·family Empty·forcedfalse다. raw stdout의 exact1pass도 확인했으며 9/9는 이 선택9만의 분모다. 등록31개 전체 실행이나 전체 제품 인수로 확대하지 않는다.1238/1239는 ffprobe 실제 실행,1240~1242에서는 ffprobe guard-only다. 실제 ffprobe 실행 중 취소/drop/unwind의 증거가 아니다.

원 내부 capabilities/inspect close receipts도 정상은exit0/familyEmpty/forcedfalse,잘린입력 inspect는 nonzero로 보존했다.1245의 held 합성 child27304/creation134375829715146532는 child_exit124/familyEmpty/forcedtrue/stdin_eof_requestedtrue다. 외부 test 정상 종료와 내부 의도된 강제 종료를 구별한다. client-drop을 child 자연 종료나 실제 decoder 인수로 표시하지 않는다.

원 outer는 runtime/verification/source-sdk-legacy-{real-normal,real-truncated,source-cancel,external-cancel,cleanup-failure,synthetic-normal,synthetic-existing,synthetic-drop,synthetic-unwind}-os1-r37/outer,원 내부는 같은 tag의 runtime/build-cache/tmp 아래 cases/{real-normal,real-truncated,real-source-cancel,external-cancelled-user-race,local-cancel-close-failure,normal,existing-target,client-drop,run-unwind}에 보존했다. raw/Jobs journal/phase receipt/prepare trace를 유지한다.

## 보존과 다음 목표

사후 verify-current session91295 terminal exit0으로2296파일 content/identity 동일을 확인했다. 사전/사후 전체 입력 확인을 구별해 보존했다.

마감 시 tool·media·원 manifest·C-only manifest·held fixture SHA를 재확인해 위 핀과 같았다. source 수정이 없어 fmt/default/support/parent strict는 R37 원 실행 근거로 유지하며 이번에 재실행했다고 쓰지 않는다. 지정190/정상제품bins도 원 DFB0 입력의 근거다. C-only cache/tmp·conservative2/CPU2/BelowNormal·heavy1·같은 exec handle terminal 확인 후 직렬을 유지했다. 이번 gate의 첫 실행 실패는 없다. 과거 정상시험 observer/취소/rename 실패와 R37 compile·배정 오류는 원 기록대로 보존한다.

다음 공백은 receipt 실패 중 origin 취소/정리의 독립 진행,production provider policy/원 source factory와제품 등록,실제 HTTP/probe와 전체 CLI 행렬이다. GUI 원본/후보 비교·재생·수정·명시판정·이력과 실행 시안 비교도 남는다. AI 사용자0/9/원NeedsRepair·WIP/담당 소유권·source375 private overlay 유지. 실제 추론/다운로드/수동GUI/WARP/push0·배포 제외.

기독 modootoday/software-qa/test-fixture-and-mock-build SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 원 fixture/실제decoder·정상/취소/teardown 분리와 code-change-review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 producer·원오류·주장/근거 분리를 적용했다.
