---
id: plan:kinekino/20261029113515
title: 제품 통합 일정 추정
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 통합 일정 추정

2026-10-21 KST 사용자 중간 일정 요청 후 worker139·UI116 반환 반영. roote9aee3e/Meari0b9ea03. 날짜는 실측 개발 속도로 계산한 납기가 아니라 낮은 확신의 계획 구간이다. [실행 권한](execution-leases.md)과 [전체 완료 조건](full-product-integration-gates.md)을 따른다. 이전 추정은 [원문](../../artifacts/coordination/delivery-forecast-before-561ae92.md)에 보존했다.

## 현재 진행

| 영역 | 완료 근거 | 현재/남은 작업 |
|---|---|---|
| 공통 SDK/CLI | 저장·출처/CAS·반복 편집·명시 채택·exact retry 실제 CLI 인수 | 전체 제작/AI executor와 기본 제품 조립은 미완료 |
| UI 실행 소유·실제 IPC | e9aee3e 제품 caller/surface33 고유116 통과·반환 | Rail 반복 표시/제거·focus/a11y·drained callback/lastgood 심화, 기본 GUI 프로젝트 선택/시각 검토 별도 |
| Meari | library0b9/root561 도구 분리, Core84+Native38+CLI2=124 통과. Native integration 일반 bin0 | Web 실제 동등성·GUI/GL·설치/전체 배포 인수 미완료 |
| AI worker 제어 | normal129+경로helper2+실제 child8 고유139 통과·반환·048808a 채택 | 자연 부모 만료/실제 ORT/생산 admission은 별도 미완료 |
| 생산 AI 연결 | release/issuer/loaded-module 후보 정적 검토 | 독립 child 설정·고정 release 입력·실제 ABI/loader/모델 IO·품질 인수 미완료 |
| 시청/제작 전체 | 과거 고정 CLI 세 분류 실제 provider→Frame-host→native 관측, 일부 자막/스냅샷 계약 인수 | 최신 시청판 회귀·음성/광고 차단/실패 복구·다운로드/자막ASR화자분리/카드/숏폼/OPED/피사체·2D3D 전체 종단 연결 필요 |

시험 수는 해당 범위의 근거이며 합산 완성률이 아니다. 현재 위치는 검증된 공통 기반에서 실제 제품 흐름으로 연결하는 단계다.

## 예상 개발 구간

| 시점(10/21 기준) | 목표 | 완료 판단 |
|---|---|---|
| 다음 검증 주기 | Media 설정26 default 기능 검증·UI slot 수명 후속 → 실제 configured worker22 연결 | worker139·UI116은 반환/책임 채택됐다. 생산 권한/실제 모델 IO와 Rail 반복 수명은 별도 미완료이며 실제 소요 일수는 아직 확정하지 않음 |
| 1~2주 / 10/28~11/4 | 제한된 통합 알파 후보 | 핵심 SDK/CLI와 기본 Native 사용자 흐름 연결. 사용자 확인 가능한 후보이며 전체 제품 완료 아님 |
| 3~5주 / 11/11~11/25 | 핵심 시청판 후보 | 세 분류/회차/컨트롤/커서/오디오·실패 복구·저장/재열기·패키징의 최신 실제 경로 인수 |
| 4~10주 / 11/18~12/30 | 제작 기능·제품 AI 병렬 연결 | 다운로드/자막변환·ASR·화자분리/스냅샷/카드/숏폼/OPED의 공통 executor·CLI·UI 종단 검증 |
| 8~14주 이상 / 12/16~2027/1/27 이후 | 승인된 전체 설계 완성 후보 | 피사체/검색/2D3D·기능별 오류/취소/재시작·Native/Web 동등성·마지막 사용자 모델 품질 승인·전체 배포 |

구간은 서로 겹쳐 병행한다. 전체14주는 상한이 아니다. 최근 실제 IPC/패키지 경계는 전진했지만 전체 기능 개발 속도와 시각/미디어 품질은 미측정이므로 이전 날짜 범위를 임의로 단축하지 않았다.

## 자원·의존성과 재산정

Media 설정26 default gate가 현재 유일 heavy이며 SDK/UI/Meari는 반환했다. UI lifecycle source 작성과 독립 읽기 검토를 병행한다. 신규 추론·다운로드는 중앙 단일 worker 배정 전0이다. Pages·VM·DPI·새 장시간 시험·수동 GUI 시각 인수 보류는 유지한다. 필요한 보류 검증을 재개할 수 있어야 출시 후보를 인수할 수 있으며 해당 대기 시간은 위 기간에 추가된다.

다음 재산정은 실제 worker 제어 gate와 기본 제품 Card/프로젝트 흐름의 기능 인수 후 수행한다. 모델 품질 사용자 검토는 제품 IO 연결 뒤 마지막이며 독립 모델 실험을 제품/품질 승인으로 확대하지 않는다. 전체 scope를 줄여 날짜를 맞추지 않는다.
