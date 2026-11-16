# 루틴 공통 운영 (항상) [사용자 결정 2026-11-03 전달, 근거 문서 없음; 기록 형식은 각 로그 파일]
루틴 3개(천천히 완성, 버그 점검·수정, 공개 표면 점검)가 공통으로 따른다. 루틴 프롬프트는 이 파일을 가리키기만 한다. **이 파일이 정본**이다. 루틴 프롬프트·메모리에 예전 숫자(예: 락 25분)가 남아 있어도 여기 규정이 우선이며, 차이가 보이면 보고에 한 줄만 적고 이 규칙을 따른다.
- 시작·락: `.local-status/.routine-lock`이 있고 **mtime 기준 45분 이내**면 종료한다(status=`skipped-lock`). 없으면 만들고, 끝나면 지운다. 락 stale 기준 45분·작업 중 갱신은 2026-11-05 원 님 승인. 빌드는 공유 빌드 락(build-and-test.md, 경로는 `.agents/local/build-env.md`).
- 락 갱신: 작업이 이어지는 동안 **단계마다**(준비→테스트→구현→검토→게이트→병합·push), 그리고 어떤 단계든 **최소 10분마다** 락 파일에 `touch`로 mtime을 갱신한다. 갱신하지 않아 stale로 잡히면 다른 루틴이 들어와 겹칠 수 있다.
- 하트비트: 모든 루틴은 시작·종료 때 `.local-status/routine-heartbeat.json`을 갱신한다. 키는 루틴명(`천천히 완성` / `버그 점검·수정` / `공개 표면 점검`). 값: `lastStart`, `lastEnd`(ISO8601 KST 또는 null), `status`(`ok` / `skipped-lock` / `failed` / `merged` / `queue-empty` / null), `note`(짧은 문자열). 시작 직후 `lastStart`와 임시 status를 쓰고, 끝날 때 `lastEnd`·최종 status·note를 쓴다. 2026-11-05 원 님 승인.
- 조용한 실패 금지: 락 스킵·실패·병합 없음 종료도 `.local-status/routine-log.md`에 **한 줄**(시각 KST, 루틴명, 결과)을 남긴다. 공개 표면은 `.local-status/public-surface-log.md`에도 같은 형식으로 남긴다. 2026-11-05 원 님 승인.
- 24시간 침묵 알림: 시작 시 heartbeat의 해당(또는 전체) 루틴 `lastStart`/`lastEnd` 중 더 최근 시각이 **24시간을 넘기면** 원 님에게 **한 번만** 보고하고, 같은 침묵에 대해 반복 보고하지 않는다. 2026-11-05 원 님 승인.
- 기록: 진행 `.local-status/routine-log.md`, 버그 `.local-status/bug-log.md`(KB### 번호, 상태 OPEN/QUEUED/FIXED), 공개 표면 `.local-status/public-surface-log.md`, 하트비트 `.local-status/routine-heartbeat.json`, 작업 큐 `.spec.local/kinekino/plan/20261101003500-next-slice-queue.plan.md`.
- 일요일 실행은 테스트를 3회 반복해 불안정 테스트를 찾는다.
- 낮은 심각도는 기록만. 새 기능 제안 금지.
- 보류 항목(원 님 지시 전 손대지 않음): 릴리즈, SDK 태그, allowlist 2~4단계, UI 목업 재개, 아이콘 시트 겹침 수정, windows-triage 번들 삭제, meari 릴리즈, 미디어 다운로드.
- 빌드·저장소 문제로 진행 불가: 원인과 남긴 상태만 한 번 보고(코딩은 루틴 본체가 직접 한다). 같은 막힘 2회면 지시를 바꿔 1회 재시도, 그래도 막히면 브랜치를 남기고 보고. [.local-status/WORK-METHOD.md]
- 루틴별 역할·스킬·지시문 템플릿: `.local-status/WORK-METHOD.md`.
- 로컬 값: 추적 규칙이 가리키는 개발 호스트 경로·명령·용어·운영 기록은 `.agents/local/*.md`(추적 안 함)에 있다. 루틴은 항상 로드 규칙과 함께 읽는다. [2026-11-16 원 님 지시]
- 큐가 바닥나면(대기·진행 후보가 결정 열 때문에 모두 건너뛰어져도 집을 조각이 없으면): 로드맵에서 새 행을 임의로 만들지 말고, 원 님에게 **한 번만** 보고한 뒤 heartbeat `status=queue-empty`로 조용히 종료한다. [.spec.local 큐 사용법, 2026-11-05 원 님 승인]
