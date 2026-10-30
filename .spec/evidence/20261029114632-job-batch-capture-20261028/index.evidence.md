---
id: evidence:kinekino/20261029114632
title: 일괄 입력 capture 검증 — R126
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 일괄 입력 capture 검증 — R126

R127 입력 고정:2167snapshot 종료0,2373파일/inventory F8952C69A6A826DFC1869CBE1ADE099FEBADE19B4F43B710765B17A2B5DDBE71,SOURCE.md B60D35EBB14FF1AE6157321FBBAD3DE7A9980D065C4E8517A6A3983496256C95.2168 카드 lib/Jobs/CLI no-run 진행 중이다. 원 실패는 R126 증거로 유지하며 R127 실제 시험/strict/190/bins/custody는 미완료다.

R127 원인 후보 수정 진행:2155/2156 capabilities timeout 이후, 카탈로그가13종 각각에서 동일한 prepare_product_styles/font_metadata를 반복하는 코드를 확인했다. preimage-r126-catalog-assets/bundle.rs 보존 후 조회당 한 번 검증한 폰트 메타데이터를 공유하도록 변경했다. 전역 캐시·검증 생략·deadline 증가는 없다. 개별 템플릿 검증은 같은 helper를 사용하며 선언/핀은 변경하지 않았다.2166fmt 종료0,2167snapshot 진행 중. 실제 원인 확정 및 CLI 회복은 아직 미검증이다. 기존 modootoday code-change-review-verification을 적용해 원 실패·후속 입력·검증 범위를 분리한다.

기본 gate 최종:2164 정상2bins build 종료0,exe/PDB는 fixed-r126/native-bins에 보존했다. GUI SHA F037711A6E555A587F62EC51879EAC373D2EF4910E48A1310AEEF3212A4CC60D,CLI SHA 4CE2A54063B17D84C3C52EECA14E317BB0442219250E5334BFF9AA5CD1659C77이다.2165 사후 custody 종료0. 카드CLI의 capabilities 시간초과 2회는 미해결이며 다음 원인 조사 대상이다. 전체 일괄 기능·전체CLI/GUI·사용자품질 승인 완료가 아니다. 아래 진행 상태는 각 실행 당시 기록이다.

지정 회귀:2160 Product145/2161 Core18/2162 Native26/2163 OS1 종료0,190/190 통과. 네 group-return의 execution_result/producer_integrity 모두 Ok이며 source_receipt_sha256은 R126 SOURCE.md와 같다. Meari는 기존622/624 producer manifest로 실제 시험을 다시 실행했으며 새 컴파일로 계상하지 않는다.2164 정상 player/player-cli build 진행 중,사후 custody 미실행. 카드CLI2155/2156 실패는 이 지정 회귀의 통과 범위에 포함되지 않는다.

후속 gate:2157 기본 strict/2158 fmt 종료0.2159 지정회귀 실행은 증거 상위 디렉터리 누락으로 시험 시작 전 os error3이며 원 로그를 보존했다. 누락 디렉터리를 생성한 뒤2160 Product145 실행을 시작했다. 이는 카드CLI 시간초과 실패를 해소하거나 대체하지 않는다. 전체190/bins/custody는 아직 완료로 계상하지 않는다.

첫 실패 보존:2154 카드 strict는 종료0이며 외부 blitz-paint 기존 경고1개가 남는다.2155 추가 카드 CLI 회귀는 첫 capabilities 응답의 client_deadline/read_response timeout(exit10)으로 실패했다. 요청은 실행 여부 불명이며 자동 재시도하지 않았다. 원 host는 completed_total=0·dispatch/pipe/sdk 종료·process exit0·family Empty지만 시험의 정상 완료 기대와 달라 실패로 보존했다.2156은 제한시간·소스·고정 exe를 바꾸지 않고 새 evidence root에서 수행한 독립 진단 실행이며 같은 첫 capabilities timeout이 재현됐다. 정상 경로 통과로 계상하지 않으며 원인 미확정이다. evidence roots source-sdk-r126-card-cli-r1/r2와 원 stdout/stderr/receipts를 보존한다.2157 기본 strict 진행 중이다.

추가 확인:2151 카드 no-run 종료0(223.645초),2152 실제 카드 capture→원 submit/묶음 발행/동일 key replay/재열기 시험1개 종료0(2.80초),2153 journal 계약7시험 종료0. 카드 원 host shutdown은 completed_total=1,error=null,secondary_count=0이다. fixed-r126/card의 lib SHA DEC3303DDDBFA348C25A271E011B15E5D567B6FB3CC3247BEC3FEE4FE13DBF82,Jobs SHA F1A66DA36C8BFF776E231BF8AD2FF3C520BB93B0E1334BFF1102662CAC923A6D,CLI SHA B044FDA073E7E65A5E0715BE42F74C8C7E6E268FA65B961A942A59C61A37878C를 보존했다. 기본 lib SHA는 7E7CCDC69D3CFA148CB9CA03EE047932096AF7B68B4991C8B225FDC1718720B9다.2154 카드 strict 진행 중이며 아래 이전 진행 상태는 당시 기록이다.

진행 중이다. 전체 일괄 지원이나 전체 제품 인수 완료를 뜻하지 않는다.

입력은 private overlay input-r126의 2,373파일이며 inventory SHA는 77D3C44E8304999EA8782989C0F45CF01CB6D1037516D6AEFFF0E5F3BD90D1AC, SOURCE.md SHA는 14720964620CFC3A5D5C4998837CA2CC2298E0F8103E50A2B869A656CB3783B8이다. preimage-r125-batch-capture와 기존 원 증거를 보존했다.

2145 fmt, 2147 media-jobs no-run, 2148 기존 공통 Jobs 80시험, 2149 기본 제품 lib/Jobs/CLI no-run이 종료 0이다. 2150 고정 실행파일의 capture 시험 2개도 종료 0이다. 실제 자막 source 고정·작업 미생성·출력 부재·원본 변경 거절·원 submit 및 출력·같은 키 replay·read-only 거절을 확인했다. 별도 합성 adapter는 미허용 및 normalization 비멱등 입력을 거절하고 작업을 만들지 않았다.

실제 증거는 runtime/verification/source-sdk-r126-batch-capture-r1에 보존했다. 원 retaining host shutdown은 subtitle completed_total=1, 두 거절 사례 completed_total=0이며 세 사례 모두 error=null, secondary_count=0이다. typed document의 합성 scope는 실제 BatchStore custody 증거가 아니다.

2151 카드 구성 no-run 진행 중이다. 카드 capture 실제 시험·양 구성 strict·전체190·정상 bins·사후 custody는 아직 통과로 계상하지 않는다. 실제 event store·BatchLease orchestration·SDK/CLI·부분 실패 인수도 남는다. 최신 완료 기본 기준선은 R125, 카드 실제 CLI 기준선은 R123이다.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 실제 자막 원 소유자 시험과 합성 거절 시험, 아직 없는 실제 일괄 저장을 구분했다. AI 사용자 승인0/9·WIP·소유권·보류를 유지한다.
