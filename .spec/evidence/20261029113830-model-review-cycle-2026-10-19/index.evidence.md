---
id: evidence:kinekino/20261029113830
title: MODEL 실제 표본 검토 r153
status: active
created: 2026-10-29
updated: 2026-10-29
---
# MODEL 실제 표본 검토 r153

2026-10-19. 기존 [MODEL 시각 계약](../../player/presentation/system/models-visual.md)과 [표본 검토 계약](../../player/presentation/system/model-review.md)의 확장을 검증했다. 기존 시안에 없는 검토 기능을 임의 이미지와 픽셀 일치했다고 주장하지 않는다. 공용 Accordion/Card/Button/Badge/Notice와 Pretendard·토큰을 재사용했다. 전체 제품 시안 일치 작업은 미완료다.

## 실행과 검사

SDK 소유 단일 NVMe 통합 검사: cargo fmt --check, cargo test 기본 병렬 138pass/9ignored, cargo clippy --all-targets -- -D warnings, cargo build --bins 최종 exit0 통지를 받았다. UI 상태 회귀4개가 포함된다. 최초 테스트 JSON 매크로 문법 오류는 소유 파일에서 수정한 뒤 통합 검사했다. root도 동일 로그를 확인했다. UI가 별도 중복 cargo를 실행하지 않았다.

실제 실행본은 `runtime/review-runs/ui-model-review-20261019-r153/player.exe`, PID16784, SHA256 `d4781af07f57323b49753e88f82092da44133c4a6e2a38ea2b4d36af23ea7e17`이다. 기존 검토 창이 없는 fresh sky 목록을 확인한 뒤 `--record-validation --validation-position=3500,100 --review-root=ABSOLUTE_COPY_STORE`로 한 개 실행했다. 가상 모니터를 사용했고 VM/DPI/모니터 구성은 변경하지 않았다. 일반 시청1180×760에서 제작 진입 뒤 제작1440×900이 됐다.

r3 정본의 packages/objects 바이트를 `runtime/review-runs/ui-model-review-20261019-r153/store`로 복사하고 별도 빈 decisions 디렉터리를 준비했다. 실제 정본 사용자 평가 저장소에는 테스트 판정을 쓰지 않았다.

## 실제 관측

- MODEL에 기존 카탈로그와 별도 실제 표본 검토 영역이 표시됐다. 64자리 SHA 입력 전 자료 열기는 비활성이고, 로딩 중 입력/자료 열기가 비활성으로 바뀌며 중복 실행 방지 안내가 보였다. 완료 후 r3 package ID·전체 SHA·무결성과 비교 자료 준비 상태가 표시됐다.
- Gemma 교정의 검토 방법/기대 결과·비시간 표본 안내, 기준선과 후보 병렬 카드·model/revision/precision/engine 조건을 확인했다. 기준선1232B와 후보516B를 명시적으로 읽어 전체 줄바꿈/선택 가능한 본문과 복사 버튼을 표시했다. 긴 기준선 내부 스크롤과 후보의 잘린 JSON 끝을 확인했다. 복사 버튼의 클립보드 실제 내용은 별도 검증하지 않았다.
- 같은 task 비교를 켠 상태에서 Gemma 교정과 Qwen 교정 카드가 같은 공용 구조로 표시됐다. 선택 메뉴에서 Llama 카드로 전환하면 비교 대상도 cards로 바뀌었다. 비교를 끄면 선택한 llama1b-cards만 표시됐다. 세 모델의 모든9개 결과를 GUI에서 읽었다는 뜻은 아니다.
- Llama 카드 원본 JSON2129B를 펼쳐 명시적으로 읽었다. 동일 SHA의 기준선에도 같은 문서 캐시가 표시됐다. 후보129B의 실제 한국어 headline/summary와 cue_ids6·7을 원문 그대로 확인했다. 입력 조건 펼침에서 decoding, request_object_sha256, source_manifest_sha256을 확인했다. 원문 읽기는 JSON 완결성·내용 품질의 성공이 아니다.
- 판정은 최초 미선택/빈 메모이며 저장 비활성이었다. 판단 보류만 선택해도 메모가 없으면 비활성이었다. 최초 이력 조회는 사용자 표본 판정 없음이었다.
- 복제 저장소에서만 `UI TEST ONLY / 복제 저장소 동작 검증 / 사용자 품질 판정 아님 / 日本語・한국어 / 저장과 이력 확인`을 입력하고 판단 보류를 한 번 저장했다. 저장 중 폼이 비활성으로 바뀌고 완료 후 메모/선택이 초기화됐다. 반환 decision ID는 `ui-1792359172733749500-4`, SHA는 `c31038da90b53edff29db96604d559a57ac75186e6b0f7409c9cd5a085ece9a6`, 표본 검토 통과는 아니요였다. 이력을 명시적으로 다시 읽어 같은 ID·판단 보류·메모·SHA를 확인했다.
- Gemma 교정에서 Llama 카드로 바꾸면 이전 저장 결과와 메모/판정 선택이 제거됐다. Llama 카드의 이력은 사용자 표본 판정 없음이었다. 테스트 저장을 실제 사용자 품질 승인으로 취급하지 않는다.

별도 [실제9개 내용 감사](../ai/text-review-content-audit.md)는 개발 담당의 원문 대조이며 이 GUI 동작 검사와 구별한다. Gemma/Llama 교정 잘림, Llama 카드 본문과 cue 근거 불일치는 품질 후속 사항이다. 모든 카드/모델에 대한 포괄 판정으로 확대하지 않는다.

## 실패와 남은 확인

Llama 카드의 원본 자료 Collapsing 영역을 펼칠 때 `request_discard has been called 11 frames in a row` 경고를 한 번 관측했고 다음 캡처에서는 사라졌다. 안정된 idle frame/접기 애니메이션과 Taffy 재측정의 회귀가 필요하다. 숨기거나 성능 정상 완료로 표시하지 않는다.

오른쪽 아래 corner와 오른쪽 edge 드래그 각1회에서 창 크기가 바뀌지 않았다. 따라서760 실제 실행 검증은 하지 못했다. 합성 compact 렌더 테스트를 실제 좁은 창 인수로 대체하지 않는다. 검토 폭 제어와 실제 크기 조절을 후속으로 확인해야 한다.

남은 범위:760/980/1180 실제 표본 비교,9개 모두 읽기,원문 전체 복사 결과,키보드 표본 선택·focus,실제 package 전환/없는 package/unsupported/손상·잘못된 UTF8 오류,저장 응답 소실·늦은 응답,재시작 후 이력 지속성,상세 픽셀·hover/pressed 일치. 자동 상태 회귀와 실제 입력 관측을 구별한다.

캡션 닫기 입력 후 get_window_state는 usable app window가 아니라고 반환했다. 이어 새 sky.list_windows의 r153 경로 필터 결과는 빈 배열이었다. 창 소멸 확인이며 별도 PID 소멸 증명으로 확대하지 않는다.

## r154 검토 폭 제어와 좁은 창 후속

r153의 소스/138 통과/고정 EXE는 이전 체크포인트로 유지한다. 후속 제품 변경은 기존 `--record-validation` 실행에서만 제작 heading에 검토 창 크기 선택기를 표시하는 것이다. `presentation/validation.rs`가 egui viewport InnerSize 명령만 소유하고 StudioShell에는 명시적 bool을 전달한다. ordinary startup에는 활성하지 않는다. SDK·main·모니터·DPI 설정을 변경하지 않았다.

source disclosure의 표시 위치를 실제 렌더 결과에서 찾고 pointer press/release 뒤30frame의 연속 discard와 마지막 idle1pass를 검사하는 Rust 회귀를 추가했다.760/1440에서 기본 표본, 긴 원문을 source/기준선이 공유하는 캐시, 실제 Pretendard 설치 조건 모두 통과했다. 그러나 r153의11frame 경고 원인은 아직 재현·확정하지 못했다. 경고 숨김/애니메이션 제거/문제 해결 완료 표시는 하지 않았다. 더 정확한 표본 전환·타이밍 재현은 남는다.

session34357 terminal exit0: cargo fmt --check, 전체 기본 병렬 test139pass/9ignored(lib94pass/1ignored), clippy all-targets-Dwarnings7.40s, build bins0.59s. 테스트 compile1m10s. 이 후속 검사를 r153의138 통과 기록과 혼합하지 않는다.

실행본 `runtime/review-runs/ui-model-review-20261019-r154/player.exe`, PID118632, SHA256 `c6220467fe20c80cd8206bc4c0083d1f35e42417941a5949bffadfa4029aae0e`. 같은 r153 복제 store를 사용해 재시작 지속성을 확인했다.3500,100의 가상 모니터에서 한 개 실행했고 검토 크기 선택기로 실제 창760×760을 만들었다.

- 760에서 SHA 입력/자료 열기·r3 ID/전체 SHA·자료 무결성/비교 준비·검토 방법/기대 결과가 폭 안에서 표시됐다. 기준선과 후보는 같은 순서로 적층됐다. 단독 비교 토글을 바꿔 선택 Gemma 교정만 표시했다.
- 기준선1232B와 후보516B를 명시적으로 읽어 줄바꿈 문서·내부 스크롤·잘린 후보의 끝을 확인했다. 긴 revision/engine/hash는 폭 안에서 줄바꿈됐다. 캐시된 문서 상태의 원본 펼침도 실제 입력했으며 이번 캡처들에서는11frame 경고가 재현되지 않았다. 이를 이전 경고 원인 해소로 확대하지 않는다.
- 설정 drawer를 열고 Escape로 닫은 뒤 선택 표본·원본 펼침과 스크롤 위치가 유지됐다. 미선택/빈 메모의 저장 비활성과 판정 폼·하단 이력 동작이760 폭 안에 표시됐다.
- 재시작 후 이력을 명시적으로 읽어 r153의 동일 decision ID·판단 보류·테스트 메모·동일 SHA를 확인했다. 새 판정은 저장하지 않았다.
- 입력을 유효 형식의 미존재 SHA(64자리 f)로 바꾸고 자료 열기를 실행했다. NotFound 오류와 실제 SDK 메시지가 표시되고 이전 r3 package/item/본문/이력·저장 폼은 제거됐다. 없는 자료를 승인 대기로 표현하지 않았다. 서로 다른 두 유효 package 사이의 전환까지 확인한 것은 아니다.

r154 캡션 닫기 뒤 새 sky 목록의 r153/r154 경로 필터는 빈 배열이었다. 현재 소유 검토 창은 남아 있지 않다. 별도 PID 소멸 증명은 아니다.

후속 남은 범위는 위 목록에서760 기본 비교/폼·오류 및 재시작 이력 관측을 제외한 항목이다. 특히11frame 경고의 정확한 재현/원인·최소수정,9개 모두 읽기,유효 package 간 전환,실제 clipboard 내용,키보드 표본 선택,980/1180은 미완료다.

실행 검토 뒤 회귀 테스트만 추가 보강했다. 원본 본문에 고유한 fixture 문구를 두고 클릭 전 숨김·클릭 후 실제 표시도 단언하므로 입력이 무시된 검사를 안정성 성공으로 취급하지 않는다. 단독session33804 exit0와 최종session3258의 fmt--check/전체139pass9ignored/clippy2.14s exit0를 확인했다. 제품 렌더 코드는 r154와 같으며 이 테스트 보강을 새 실행본 GUI 검증으로 주장하지 않는다.
