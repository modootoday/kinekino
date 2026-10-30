---
id: plan:kinekino/20261029113558
title: 설계 문서 책임별 재배치 계획
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [specification]
---
# 설계 문서 책임별 재배치 계획

2026-10-18 사용자 명시 승인: “.spec 재배치는 승인합니다.” 아래 준비 계획에 따라 책임별 재배치를 실행했다. UI 상세 시안·모델 선정 승인과 구별한다. 후속 지시로 Pages 갱신은 중단하고 로컬 문서에만 반영했다.

## 실행 결과와 복구

초기 `.spec`159개 파일을 목록화하고109개를 이동했다. 기존20개 SOT의 ID는 유지했다. UI는 presentation, 상태 전이는 application/jobs, 실제 검증은 sots/verification, 원시 증거는 evidence, 과거 생성 시안은 artifacts/design/archive에 배치했다. 기존 ADR/Plan은 각 단일 결정·작업 범위를 담당해 경로를 유지했다.

sot012/013의9/10개 절을 기능별 문서로 분리하고 원래 문서는 진입점으로 유지했다. [절 대응표](../../evidence/maintenance/reorganization-2026-10-18/sections.json)는 원문 절 해시·대상·판단 근거를 기록한다. [전체 대응표](../../evidence/maintenance/reorganization-2026-10-18/mapping.json)는 기존→신규·원본 해시·백업 경로다. [검증 결과](../../evidence/maintenance/reorganization-2026-10-18/verification.json)는 최신 링크·불변 자료 검사 결과다.

백업 `runtime/spec-reorganization/backup-1792286332`에 전체 원본 `.spec`와 관련 외부3개 파일, inventory.json을 보존했다. 절 분리 직전 본문은 `runtime/spec-reorganization/section-backup`에 추가 보존했다. 복구는 편집을 멈추고 대응표/해시를 대조한 뒤 해당 원본만 복사한다. 현재 초안·새 문서를 자동 덮어쓰는 일괄 rollback은 제공하지 않는다. 백업은 삭제하지 않았다.

최초 검사에서 외부 도구·로그·폴더 링크9개를 찾아 수정했다. README·cleanup 도구 provenance·기존 요약 진입점·디자인 manifest contract를 갱신했다.105개 원시 자료/PNG의 해시와 디자인 manifest32개 파일을 검증한다. 외부 웹·원격 앵커·GUI/CLI 동작은 이관 검사 범위 밖이다. 현재 로컬 앵커 참조는 없다.

새 SDK/GUI·CLI/제품 식별·화자 분리 계약을 새 구조에 적재했다. 구현은 [SDK 계획](plan-006-sdk-gui-cli.md)으로 분리한다. DPI/VM/연속 재생 시험은 재개하지 않았다.

## 목표 책임 구조 제안

- sots/player/presentation: 공통 frame/tokens/components/inputs, viewing, studio 기능별 계약.
- sots/player/domain: 미디어·구간·자막·편집·투영 등 정책과 값 계약.
- sots/player/application: 작업 수명·revision·모델·미리보기·내보내기 유스케이스와 포트.
- sots/player/infrastructure: WebView2/광고필터/mpv/HF/추론/파일/Windows 어댑터 계약.
- sots/verification: 실제 재생·DPI·자원·정리 결과와 evidence 링크.
- adrs/player: 결정 근거와 승인 상태. plans/player: 우선순위와 미완료 게이트.
- artifacts/design/<version>/<viewing|studio|system>: 화면 ID별 이미지와 manifest.
- evidence: 도메인/시험별 원시 관측 자료. 디자인 시안과 검증 스크린샷을 구별.

아래는 준비·실행 계획 이력이다. 최종 경로는 대응표와 README를 따른다. 기존 검증 문서는 당시 시험 단위의 관측/수정/한계를 보존하며 새 기능의 규범 정책을 대신하지 않는다.

## 승인 전 준비

현재 준비 결과: 독립 검토 화면25개·13개 화면 계약·189개 요소 ID, 공통 필드/상태/일관성 문서, 디자인 PNG32개 manifest를 적재했다. 주요 생성 오류를 수정했으나 공통 Rust 렌더의 픽셀·DPI·접근성 확인은 구현 후 인수 게이트다. 사용자 승인이나 기존 문서 전수 경로 매핑 완료를 의미하지 않는다.

1. 현재 탭/하위 탭 시각화 완료, 요소 계약과 일관성 검토 기록.
2. 기존 모든 문서와 인용/상대 경로/증거 파일을 읽기 전용 목록으로 수집.
3. 문서별 단일 책임 분류와 기존→신규 경로·분할 대상 매핑 준비.
4. 중복 공통 규격·상충 값·관측/설계 혼합 식별, 정본 우선순위 제안.
5. 사용자에게 결과와 재배치안을 제시하고 명시적 승인 기록.

## 승인 후 실행

1. 대상 파일 목록·내용 해시·연결 관계를 보존하고 복구 가능한 스냅샷 생성.
2. 검토된 대응표에 따라 이동/분리. 루트 지침과 UI 정본 링크를 갱신.
3. Markdown 상대 링크·앵커·이미지 참조·Pages 링크를 Rust 검증 도구로 검사.
4. 파일 수/해시와 내용 손실·중복·순환 참조를 확인. 관측 증거는 내용 변경 금지.
5. README를 새 책임 구조의 진입점으로 갱신하고 제거/이동된 옛 경로 안내 방식을 확정.
6. 승인 기록·결과·미해결 링크를 보고. 실패 시 매핑/스냅샷으로 복구.
