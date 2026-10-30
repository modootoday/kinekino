---
id: sot:kinekino/meari-react-router-route
title: Meari React Router를 소비하는 제품 route 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Meari React Router를 소비하는 제품 route 계약

2026-10-19 사용자 직접 지시: Web/Native URL 기반 동일 화면·탭·페이지·데이터 전달·history/deeplink를 Meari의 React Router 호환 경로로 소비한다. 별도 onNavigate 독자 router를 제품 정본으로 확정하지 않는다. 현재 callback TSX는 삭제하지 않고 향후 Link/NavLink·route context 소비로 이관할 요구 원본으로 유지한다. 실제 공식 API/모드·지원 행렬·공통 IR/runtime과 trace 인수는 Meari 담당이 소유한다.

제품의 안정 ID/path 초안은 src/presentation/web_ui/routing/routes.json이다. 이 파일은 현재 runtime registry가 아니며 source가 지정돼도 해당 route/화면/SDK 기능이 완성됐다는 뜻이 아니다. 표시 제품명과 독립된 ID를 사용한다. 첫 runtime 소비는 Meari가 지원 계약을 확정한 뒤 같은 선언 source로 검증한다.

## 경로와 전달 데이터

VIEW는 시작/작품 탐색/선택 source 재생의 route를 분리하고 제작은 기존 workspace와 하위화면 ID에 대응시킨다. category/sourceId params와 project/source/proposal/cue/job/model selection query는 공통 Rust validation·SDK 조회로 연결한다. raw 외부 URL·파일 경로·서명 URL·draft·원생성·인증 자료를 URL에 넣거나 브라우저 입력으로 임의 프로그램을 실행하지 않는다. query가 있다는 이유로 저장/채택/재생 성공을 만들지 않는다.

현재 Rust catalog.rs와 route inventory는12개 제작 workspace/20개 하위화면으로 일치한다. 과거 r158의25는 JOB 상태6개를 탭으로 센 분류였고 [r159 정정](../../../verification/presentation/job-filter-review-2026-10-19.md)과 [JOB 정본](../system/jobs-visual.md)에서1페이지+본문 상태 필터로 바꿨으므로25−5=20이다. 별도로 승인 이미지25장은 [19개 studio+6개 viewing 시안 상태](../../../verification/presentation/design-reference-review-2026-10-18.md)이며 제작 하위25탭이 아니다. 이후 화자 화면 추가로19+1=20이며 기능 삭제·새 기능 완료를 뜻하지 않는다. fullscreen/skip/진행 상태는 실제 host/application 관측과 구분하고 URL만으로 실제 미디어 상태를 위조하지 않는다.

## 상태 보존과 뒤로가기

route 선택과 proposal/cue 선택은 안정 ID로 유지한다. 조회 실패·missing ID·다른 source/revision은 오류/미선택으로 표시하고 첫 항목으로 암묵 대체하지 않는다. 한 route의 본문/inspector scroll·선택·focus와 draft buffer를 별도 presentation state로 유지한다. raw/draft 저장소를 UI routing에 복제하지 않는다.

Back/Forward/deeplink/새로고침은 같은 선택 context를 복원하며 source identity/revision 불일치를 조용히 덮어쓰지 않는다. dirty 작업 이동은 명시적 유지/저장/버리기 상호작용을 요구하고 이동만으로 저장·채택을 호출하지 않는다. browser/native history 인덱스와 view state의 유지/폐기 규칙은 Meari 공통 runtime 계약에 연결한다. 새 경로는 제목 focus, 돌아오기는 이전 안정 focus 대상으로 복귀하는 실제 키보드 인수가 필요하다.

## 실제 인수 요구

- 같은 route trace에서 Web/Native의 path/params/query·선택 workspace/subtab·typed 전달값을 비교한다.
- correction 진입→원문 조회→draft 수정→다른 탭 시도→취소→Back/Forward에서 원문/버전/draft/선택/scroll/focus를 보존한다.
- deep link의 잘못된 proposal/job/cue·지원하지 않는 query를 정상 결과로 만들지 않는다. loader 조회·오류·cancel은 공통 SDK/application에 위임한다.
- 명시적 저장/채택은 기존 revision/CAS·source 보존·automatic_application=false를 유지한다. 기술 채택을 모델 품질 승인으로 확대하지 않는다.

route 파일 작성은 parser/runtime/실제 history 인수 증거가 아니다. 현재 이 인수와 Meari route 소비는 미실행이다. 원본 source를 flatten하거나 별도 JS router로 우회하지 않는다.
