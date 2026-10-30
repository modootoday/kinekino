---
id: evidence:kinekino/20261029114425
title: VAD 원 입력 영속화와 실제 JobService admission 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VAD 원 입력 영속화와 실제 JobService admission 검증

2026-10-21 root `c7bed5ea9aba3f15ae46f7dae71205696d75f326`, Meari `4331ff3ea5cad71a7159bdb9e3f86fbfccb16220`의 기존 캐시에서 검증했다. 원 증거는 `runtime/verification/vad-job-admission-union18-r1/`이다. 공통 application 영속 envelope와 구체 infrastructure executor를 실제 Rust JobService/파일 저장소로 소비했으며, 기본 제품 operation 등록과 실제 CLI/AI worker 실행은 하지 않았다.

## 고정 입력과 최소 수리

Media r5 delta9, SDK envelope5, SDK consumer4를 중복·override 없이 고유18(기존9/신규9)로 조립했다. 원 source18 SHA256 `FFF78C9A242AAA510D7233AE9FF3363905DFE7227F3CAA3A58595CF5C191DD40`과 부모 원문을 보존했다. 원 음원 SHA·공개 요청6개를 내부 version1 envelope로 저장하고, 공개 요청·release manifest·descriptor blob pin을 구별한다. 실제 attempt가 발급받은 원 scope로 재입력을 검증하고 준비 입력 전체 SHA를 비교한다. 새 scope나 deadline을 만들지 않는다.

첫 root consumer strict는17611ms/101이었다. 시험에 sibling으로 포함한 기존 `assemble_attempt_executor`가 해당 target에서 사용되지 않는 경고였다. contract fixture에 그 기존 함수의 참조 한 줄만 추가했으며 제품 visibility·suppression·정책은 변경하지 않았다. 원 로그와 source18, 수정본을 분리 보존했다. 최종 source18은 `161E5A8E1DEC5ACA1F0643220F22A9D4887B2915AAE744D12194D05795A0F55C`, Cargo 생성 lock 포함 인계19는 `44E4CCCF0E4432E75433E10F9C7C460A80FD08C4F28D454002114CC1D06791F4`다. 채택은 중앙 PM 책임이다.

원 lock51275322를 보존하고 offline metadata로 dev dependency 로컬2의 참조를 해소했다. 생성 lock `721586F9C83ADA7CF504E5A5AE0CB168F66B017C3956CD84387D794E4EAE2528`; registry681의 name/version/source/checksum 차이0, 다운로드·수동 lock 편집·pruned lock 이식0이다. 후속 Cargo 명령은 locked로 실행했다.

## 실제 결과

선택 Media 패키지 all-target strict3055ms/0, root lib/player-cli/vad_jobs strict 수리 후2272ms/0, 최종 fmt2246ms/0이다. Media 시험 빌드와 root 시험 빌드가 통과했다. root 빌드는382747ms/0이며 Cargo가 normal GUI/CLI bin도 함께 생성했지만 실행하지 않았다. 고정 시험 바이너리4의 실제 목록을 기대43과 대조해 누락0을 확인한 뒤 full name별 exact/thread1로 실행했다.

| 대상 | 고유 실제 결과 | 범위 |
|---|---:|---|
| media-vad contract |13pass|기존 계약 및 EOF에서 취소된 실제 reader의 최종 control 확인|
| media-vad-executor product_guards |22pass|실제 ordinary-file guard·controlled lookup·공유 fixture|
| root lib prepared envelope |4pass|원 SHA/영속 schema·idempotency·실저장소 재개방·retry 보존|
| root vad_jobs 직접 caller |4pass|구체 adapter Send/Sync 계약·원 scope 취소·원 입력 변경·만료|

합계43pass/0fail/0ignored다. root lib의 나머지643개와 integration inherited23개는 각각 필터에서 제외했으며 중복 집계하지 않았다. `registration43.json`, `test-results.json`, 원 case01~43 stdout/stderr/receipt를 보존했다.

취소는 실제 두 번째 admission 중 원 JobService에 요청해 `state=failed/failure=cancelled`를 관측했다. 이는 cooperative cleanup ACK인 `cancelled` 상태가 아니다. 원 음원을 submit 뒤 변경한 경우 `needs_input/source_expired`였다. 같은 키는 재입력하지 않았고, 원 host 종료→read-only 재개방→새 host의 명시 retry는 attempt2에서도 원 prepared SHA·원 음원 pin을 유지한 채 다시 `needs_input/source_expired`가 됐다. 실제 최종 event6과 descriptor/diagnostic blob을 `evidence/case-42/`에 보존했다. output=null/publication_started=false다.

1ms 만료는 authorize_calls=1로 두 번째 admission 전에 `failed/timeout`이 됐다. admission 중간 만료를 실증했다고 주장하지 않는다. 각 filesystem caller는 기존 retaining helper의 원 host begin/poll을 통해 receipt 존재·첫 오류 없음이 확인돼야 통과한다. 이번 실행은 의도적으로 panic이나 closeout 실패를 유발하지 않았다. 실패 시 원 panic/owner 보존 코드는 별도 실증 범위로 남긴다.

## 종료와 보호

원55개 receipt는 모두 terminal(54개0, 최초 strict101 1개)이며 도구 원 세션도 종료 회수했다. 기록 고유PID109개는 마감 시 부재였다. PID 부재만으로 cleanup을 증명한 것은 아니다. root1149·Meari456(기존 docs2 포함)·양 index·부모 raw·S 비입력·Meari 고정 BIN 원본/사본14쌍 보호 차이0이다. 새 시험/normal bin 원본·사본 SHA도 대조했다. heavy/shared를 명시 반환한 뒤 문서를 마감했다.

Cargo no-run이 normal CLI 출력을 기존3E13에서 `5DB7B549361B55056E06CA68B9BAA1DE4C61C063103F19877A7A923A95AEA133`으로 재생성한 의도 변경1은 보호 차이와 분리 기록했다. 원3E13은 기존 immutable artifact와 추가 사본으로 보존했다. 새5DB는 실제 CLI 실행0이며 과거3E13의 실제 소비 근거를 승계하지 않는다.

## 남은 인수

실제 CLI injected-host submit/get/dedup→명시 종료→재개방→retry/get은 별도 후속이다. 기본 operation 등록·제품 지원, 실제 AI worker/runtime load·모델 추론·라이선스/품질 승인·GUI 인수는 이번 결과에 포함하지 않는다. 동기 ProfileAdmission/SourceBlobStore 호출은 hard deadline으로 선점할 수 없고 호출 후 원 scope 확인을 유지한다. 전체 제품 goal은 계속 진행 중이다.
