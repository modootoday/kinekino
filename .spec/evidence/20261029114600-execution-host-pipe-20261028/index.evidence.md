---
id: evidence:kinekino/20261029114600
title: R85 원 SDK 실행 host의 실제 Windows pipe 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R85 원 SDK 실행 host의 실제 Windows pipe 연결

2026-10-28 중앙 PM. infrastructure/local_host/execution_host를추가해공통HostExecution의짧은RPC를기존인증LocalHost에연결했다. 원작업은별도원ExecutionOwner worker에서계속실행하며기존동기제품host를교체하지않는다. 기존security::endpoint의OS CSPRNG·검증된process identity를재사용한새nonce를hostscope로사용한다. 실제pipe endpoint는LocalHost가독립소유한다.

PendingExecutionLocalHost는원HostExecution과기존PendingLocalHost를모두보유한다. poll은원SDK close진행과pipe/dispatchworker조인을별도관측하고두계층모두Closed/quiescent인경우만완료다. 미ACKterminal·원workerpending을성공정리로바꾸지않는다. trusted관리소유자가원ACK를수행할수있지만외부pipe재개방/새PID소유권획득은없다. CLI시작옵션/실제frame충돌/강제crash전수는아직미연결이다.

원local_host/mod.rs를mirror/runtime/verification/preimage-r84-execution-pipe에보존하고새execution_host.rs/tests.rs를추가했다. R85 SOURCEA0F78B9E03D6AA2F1A5E2008BF7F4E954EA5F3C3BBD9293A41F789D3CB2712D4,inventory057C592FD04C524E714D20B4B12B97D662047761BD6BED8EAEB866AEE5D21289/2329. mirror/runtime/verification/input-r85. fixed-r85의lib시험exe SHA0C5A0656D5DB7B7955E6D78BA622F3F1184640CE5BFCC734251BFE1553569D33/PDB보존.

원로그 runtime/verification/source375-datalab-b0-r1-logs:

|로그|범위|실제결과|
|---|---|---|
|1759/1761|fmt적용|exit0|
|1760/1762|Native alltargets strict·시험추가후최종|각exit0|
|1763|최종lib no-run|exit0|
|1764|실제Windows pipe선택시험|2pass/1573filtered|
|1765|snapshot|exit0|
|1766|default alltargets strict|exit0|
|1767|기존host소유권/RPC선택회귀|2pass|
|1768|fmtcheck|exit0|
|1769|최종custody|exit0·2329content/identity동일|

실제pipe시험은현재사용자/서버identity/framing경로의EndpointClient로submit→원recoverable client대기→peek/retry→원EngineFailed terminal조회→ACK→pipe/dispatch/원SDK조인을확인했다. 두번째시험은pipe종료후미ACKterminal이남으면완료false이며원ACK이후만완료true임을확인했다. 복구client는합성이고실제파일/프레임receipt저장충돌은아니다. 정상player-cli.exe를새host로실행한시험도아직없다. 세부보안거절/전송fault행렬을이번시험2개로대체하지않는다.

다음은명시CLI시작옵션·핀고정frame adapter·ready/EOF·미해결pending의원owner보유·RPCschema/typedclient및실제정상CLI인수다. 시작옵션은다른host모드와혼용하지않고미완료cleanup을성공exit로표시하지않아야한다. 기존frame_tools의핀검증/기본전역설정불변을재사용한다. 실제충돌fault시험은원파일/프로세스증거를보존하며추론/다운로드/GUI를실행하지않는다.

원R82실제MKV/190/정상bins는그입력의근거로유지한다. rootcanonical/WIP/소유권/원실패/privateoverlay·WARP/추론/다운로드/수동GUI/push보류·사용자품질0/9·goalactive유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1준수. 적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B:실제pipe/합성작업/실제CLI기능·조인관측/정리완료의범위를분리했다.
