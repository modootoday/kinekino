---
id: sot:kinekino/ai
title: AI 후속 연구 색인
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# AI 후속 연구 색인

최초 후보 조사 기준일은 2026-10-18이다. 이 디렉토리는 후속 후보·실행 전 게이트와 이후의 근거 감사·SDK 인계를 담당한다. 최초 후보 조사에서는 HF JSON 메타데이터·공식 모델 카드·소스 문서만 읽었고 다운로드·추론·빌드는 하지 않았다. 이후 실행·검사 근거는 각 문서가 연결하는 verification/evidence 정본의 시점과 범위로 구분한다. 이 색인 전체를 ‘모든 후보가 아직 미실행’ 또는 ‘모든 기능이 완료’로 읽지 않는다.

기존 관측은 [스파이크 범위](../spikes/coverage.md), [도구](../spikes/tools.md), [프로토콜](../spikes/protocol.md)가 정본이다. 기존 실패·품질 미달은 여기의 최신 후보로 덮어쓰지 않는다. 아래 표는 최초 연구 우선순위이며 현재 실행 여부는 연결된 후속 정본을 확인한다. 실행 성공·품질 승인·실제 정밀도·메모리/성능 비교 적격을 서로 대신하지 않는다.

## 현재 SDK·CLI 인계와 근거 감사

2026-10-19 최신 AGENTS.md를 따른다. 기능 정책은 공통 Rust application/domain에 두고 SDK·CLI로 검증한다. 승인된 Meari React/TSX 선언의 병행 제작은 UI 소유 세션 범위이며 모델 기능·품질의 GUI 선행 gate가 아니다. 신규 모델 추론·다운로드·worker 빌드는 제품 CLI 준비와 중앙 자원 배정 전까지 보류한다.

| 책임 | 현재 진입점 | 범위 |
|---|---|---|
| 모델 근거·평가 조건 | [modoo.today 스킬 적용 감사](modoo-existing-evidence-protocol-review.md), [비텍스트 역할 근거](cross-role-output-evidence-audit.md) | 실제 읽은 스킬·참고자료, R3/R4/text9 및 다른 역할의 관측/미검증·분모·정밀도·품질/메모리/시간 구분 |
| 모델 job와 공통 CLI | [현행 jobs 접합](ai-executor-current-jobs-integration.md), [엔진 CLI 인계](model-engine-cli-contract-handoff.md) | 기존 JobExecutor/ExecutionControl 재사용, raw/artifact·취소·불변 provenance·저장 출력의 미실행 인수 항목 |
| 실제9 결과와 제안 | [source 매핑](text-r3-source-mapping.md), [가져오기 오류·후속 적용 경계](text-r3-import-error-contract.md) | 원문 불변, needs_repair·명시 edit/accept·기술 fixture와 사용자 품질 결정 분리 |
| 제품 준비 후 사람 품질 검토 | [최종 검토 준비 상태](model-quality-final-review-readiness.md) | 미준비 항목·실패 표본·판정 근거. 지금 품질 승인이나 새 실행을 요청하는 절차가 아님 |

각 인계 문서의 정적 source 사실·기존 담당 실행 근거·새 설계 제안·미실행 acceptance를 구분한다. 신규 구현과 source freeze·Cargo/검사 슬롯은 PM 및 SDK 소유 세션과 조율한다. 같은 감사·같은 checkpoint 검사를 반복해 새 진척으로 세지 않는다.

| 실행 순서 | 먼저 해결할 문제 | 연구 문서 |
|---|---|---|
| P0 | 원래 세션이 확보한 일본어 3개 transcript ID의 ASR·시간 검토 | [평가 에셋](speech-assets.md) |
| P1 | ASR 시간 범위 오류와 정렬 근거 분리 | [정렬·화자](speech-timing-diarization.md) |
| P2 | 기존 군집의 화자 수/겹침/ID 오류 수작업 평가 | [정렬·화자](speech-timing-diarization.md) |
| P3 | MobileSAM 마스크 정답·연속 프레임 추적 | [추적·에셋](tracking-assets.md) |
| P4 | 한국어 JSON과 문장 의미 품질 개선 | [텍스트](structured-text.md) |
| P5 | 2.5D GLB 먼저; 완전 3D runtime 별도 게이트 | [추적·에셋](tracking-assets.md) |

고정 SHA는 [revision 목록](revisions.md)을 따른다. SHA는 조사 시점 스냅샷이며 파일을 확보했다는 뜻이 아니다. 다운로드 전에 개별 파일 byte 수·LFS SHA256·라이선스 원문·config dtype·변환 계보를 manifest에 고정한다. HF tag나 main만 저장하지 않는다. gated 동의나 라이선스 범위는 임의 mirror로 우회하지 않는다.

Rust의 domain 시간/화자/근거 값 → application 공통 유스케이스 → infrastructure 모델·미디어 어댑터 구조를 유지한다. GUI/CLI는 같은 계약을 사용한다. Pure Rust 계산, Rust FFI, Python 공식 참조 구현을 구별한다. 새 전처리·평가·다운로드 코드는 Rust로 작성한다.

실행 담당 인계는 [plan-008](../../../../../plans/player/plan-008-ai-followup-research.md)에 있다. 실제 실행 전에는 현재 도구의 인자를 다시 확인하며 여기서 제안한 미구현 CLI를 실행 가능한 명령으로 취급하지 않는다.

재검증에서 발견한 실제 명령·파일·전처리·라이선스 원문과 잔여 차단점은 [실행 준비 계약](execution-readiness.md)에 모았다. 일본어 archive 확보는 원래 세션의 관측이며 이 조사 세션의 실행 결과가 아니다. 기존 검증 문서 갱신 시점과 원시 manifest 시점이 다를 수 있으므로 둘을 함께 참조한다.

별도 후속 목표로 실행된6개 모델군의 [원본·변환 provenance와 출시 라이선스 게이트](provenance-licenses.md)를 조사했다. 코드·가중치·사전/tokenizer/class map·학습데이터 권리를 분리하며 HF unknown·원문401·변환 기록 부재를 명시한다. 조사 완료와 출시 재배포 권리 승인은 별개다.
