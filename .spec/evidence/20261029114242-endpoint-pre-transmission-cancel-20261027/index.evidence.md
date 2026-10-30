---
id: evidence:kinekino/20261029114242
title: EndpointClient 전송 전 취소와 실제 응답 보존
status: active
created: 2026-10-29
updated: 2026-10-29
---
# EndpointClient 전송 전 취소와 실제 응답 보존

2026-10-27 중앙 PM. [IPC 경계 계약](../../player/infrastructure/ipc/endpoint-execution-control.md). 현재 one-shot wire의 서버 실행 취소/ACK를 완료로 표시하지 않고, 원 SDK execution의 취소가 전송 전까지만 작용하도록 EndpointClient를 연결했다. source375 private overlay·기존 WIP/소유권·원 증거·보류는 유지한다.

## 입력과 변경

723 snapshot SOURCE7144F249CD6D48D2B50EC62D50458C3099A9F14D6538FF33FB5C7D364CB1C0A2, inventoryD34BE845A49A508165357ADCE707AE2ABAFE10FC33DC6772407E3A328BE65313/2242는 runtime/verification/source375-endpoint-cancel-input-r1이다. source375-endpoint-cancel-preimage-r1의 원3파일 SHA는 api-client81A966203D11952C6E0CA6A72C2CB0273337CD2E36C52020CDE1CFEC5374EC81/api-modB876A696093C7BE8564F0CD9B5C3C43D98457A321EB7482A785847653C092B10/endpoint-clientB197996D271533ECE7D809C02D80C64469916AE3C7641776B803E5587EB3D68B다. 새 Rust 시험은 infrastructure/local_host/client/controlled_tests.rs다.

공통 ApplicationClient controlled hook은 Sync control을 빌린다. EndpointClient의 기존 전용 thread를 scoped spawn/join으로 바꿔 호출 동안만 control을 사용한다. 원 호출자/서버 identity·요청/응답 cap·timeout·framing·기존 ACK·단일 SDK admission·server close 정책은 유지한다. preflight/기존 pipe-open retry/peer 검증 뒤 pre-write checkpoint만 추가했다. transmitted 이후 checkpoint/자동 retry·새 서버 operation registry·강제 취소·observer thread를 만들지 않았다.

721 기본 all-targets strict·722 fmt·724 Native all-targets strict는 통과했다. 실제 출력은 runtime/verification/source375-datalab-b0-r1-logs/721~에 보존한다. 과거 transport 또는 SDK 취소 성공을 새 입력의 실행으로 대체하지 않는다.

## 실제 실행 결과

725 Native lib/jobs/speaker no-run은 exit0/164004ms다. 726 LocalHost23/23(새3 포함,0.14초),727 sdk_execution19/19(0.04초),728 speaker_annotations_cli6/6(3.03초,기존 normal CLI14 포함)을 통과했다. 기존 security/DACL·서버 identity·응답 cap·중복 admission·abandoned reply·원 owner/worker join·주입 panic 경계도 같은23에 포함한다. stderr의 pipe/SDK fixture panic은 기존 기대 주입이며 suite exit0다.

새 실제 pipe 사례에서 전송 전 취소는 원 terminal Cancelled/delivery=not_sent/automatic_retry=false·server calls0으로 끝났고, 명시 다음 version 요청이 calls1로 성공했다. server dispatch 뒤 취소는 cancellation_requested=true/queued_cancelled=false와 원 실제 성공 응답을 유지했고 calls1로 끝났다. 사전 취소도 calls0이다. 두 원 host worker와 SDK execution worker를 각각 ACK/close/join한 뒤 종료했다. 로그의 eprintln 관측과 assertions를 보존하며 개별 CLI PID receipt를 기록한 시험이라고 주장하지 않는다.

화자 회귀의 원5개 root는 runtime/build-cache/tmp/speakers-cli-37884-{1793074477767317600,1793074477776639000,1793074477792468600,1793074477809312100,1793074478984148500}에 유지하고 endpoint-cancel-speaker-regression-r1/{direct-cancel,after-publication,before-publication,negative,edit-reopen}에 복사했다. 원 file lease·저장 전/후 cancellation·실제 CLI 응답을 유지했으며 복사본을 원 FileID로 계상하지 않는다.

729 Product145/145,730 Core18/18,731 Native26/26,732 자동OS1/1을 직렬 실행해 새 입력 선택190/190을 통과했다. Product는725 새 JSON, 변경 없는 Meari는622/624 기존 no-run artifact를 사용해 실제 시험을 다시 실행했다. 새 Meari 컴파일이나 전체 CLI 인수로 확대하지 않는다. 733 정상 bins2개 test=false/fresh=true·722 fmt도 통과했다. player SHA3B8F5C91136F1EC8A651314A464376A6BD6444A82448C9A1B5E53C90C7BC245A, CLI14E3D6D06FB834C73588A9E36E62A995F6E63D1DB45DF978092BF8A9AAED29B8다.

## 새 시험의 실제 대상과 한계

실제 Windows LocalHost/EndpointClient named pipe와 원 SDK ExecutionOwner를 사용한다. 전송 전 사례는 pipe open 뒤 시험 gate에서 원 handle을 취소한다. 기대 결과는 Cancelled/not_sent/자동 retryfalse, server calls0이며 명시 새 정상 요청은 calls1로 성공해야 한다. 전송 후 사례는 server SDK 진입을 관측한 뒤 원 handle을 취소한다. 실제 성공 응답·취소 요청 metadata를 보존하고 server calls1이어야 한다. 사전 취소된 direct EndpointClient도 calls0을 확인한다.

bounded gate와 RAII release·원 SDK terminal ACK/close·host pipe/SDK worker의 실제 join까지 확인한다. server API는 version을 반환하는 시험 client이므로 실제 제품의 모델/재생 작업을 취소했다고 주장하지 않는다. 파이프/identity/framing은 실제이고 gate/client counters만 시험용이다. 일반 player-cli --endpoint의 Ctrl-C·서버 실행 ID/cancel/query/ACK·유실 terminal 재조회는 미완료다.

modoo.today test-fixture-and-mock-build(본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)를 읽은 지침을 실제 pipe 자원 획득/실패 시 release·연결 복구·중복 dispatch0·join 검증에 적용했다. 검색의 PR triage는 비적합으로 미채택했다. 별도 성능 개선/비교 실측을 하지 않았고 새 개발/시험은 Rust만 사용했다. GUI·WARP·추론·다운로드·push 보류를 유지한다.

최종734 verify-current는 exit0/43351ms로 전체2242 content/identity 동일을 확인했다. stage/commit/push·canonical 승격·추가 설치0이며 이전 실패/미실행 근거를 보존했다. C NVMe target 문맥·TMP/TEMP·jobs1/CPU2/BelowNormal·heavy1과 전체 goal/보류를 유지한다.
