---
id: evidence:kinekino/20261029114000
title: Card scoped session·콜백 계약 CPU 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Card scoped session·콜백 계약 CPU 검증

2026-10-21. root `e6a6cad7ede6e948c421f274d2a8ee63d4028621`, Meari `2e0bf259d82937b612d4753cbce0cd97569699e4` 위에서 route2 B13과 callback8r2 E883의 교집합0을 확인하고 exact10만 기존 snapshot에 적용했다. 별도 frame 후보·SDK P1b·Meari WIP는 합성하지 않았다.

## 원 실패와 수리

원 source10 `0796A7F0DA00A9E3EC5F23ECC411C42619961ED84C4ACE2C0F11D0155AE2A566`의 Native strict는 테스트 helper implFnOnce의 type_complexity1로 실패했다. r2에서는 이름 있는 RegisterParent alias와 Box로 분리했다. lint allow나 제품 행동 변경은 없다.

r2의 실제 회귀에서는 새 route2와 scoped4/local3이 실패했다. route fixture는 기존 fault adapter 설치가 원 SDK registration을 해제한다는 사실을 놓쳐 실제 attachment가 남아 있다고 잘못 가정했다. Card fixture는 PlayerSdk::default에 proposal 저장 어댑터가 있다는 잘못된 전제로 bind 이전에 Unsupported를 받았다. 이 문제는 정적 읽기와 포맷만으로 확인되지 않았다. 원 로그·receipt·BIN을 보존했다.

r3는 시험2파일만 수정했다. 새 route2는 원 SDK 어댑터를 유지하는 실제 fixture에서 원 ticket/terminal ACK/attachment 재접속을 검증한다. 기존 fake ACK8은 그대로다. Card 공통 fixture는 Rust에서 만든 고유 evidence 폴더의 기존 ProposalFiles를 동일 SDK에 명시 주입해 실제 저장과 inspect를 수행한다. 제품을 unsupported로 바꾸거나 저장을 가짜 성공으로 대체하지 않았다.

최종 source10 SHA는 `9E3B4A2B8C63168D60371B19839017078995A2D95390F8793FCB8565BACB4CE3`, 조립1496 manifest SHA는 `A25EE1DDD024E2888E8D56D7BA2F76BD25D2A99C2BECB9B58900E62A48AB9ED6`다.

## 최종 실제 관측

원 자료는 `runtime/quality-review/ui-card-route-union10-r1/revision-r3`에 있다. workspace fmt, default/native all-target strict Clippy 및 영향받는 bridge integration2 strict가 통과했다. 최종 입력의 fresh Native libtest와 관련 integration2 BIN/list를 고정해 아래 시험만 실행했다.

| 범위 | 실제 pass | 별도 ignored |
| --- | ---: | ---: |
| Viewer callback lifecycle: 기존8+새 route2 | 10 | 1 |
| Scoped Card 선택·원 parent 보존·Immediate/Native8 async | 4 | 0 |
| 같은 export의 root/editor 대상·rollback·충돌 거부 | 3 | 0 |
| 기존 document_factory integration | 20 | 0 |
| 기존 product_window_sdk integration | 10 | 0 |
| 고유 합계 | **47** | **1** |

새 회귀9는 외부 UI의 callback guard를 대체하지 않는다. 실제 두 번째 Card 편집, save 직후 acceptance 없음, 명시 accept와 동일 SDK의 inspect 재조회, 원 async execution ACK/등록 해제 후 재접속을 확인했다. local composition의 성공/실패 apply는 시험 closure이며 실제 NativeDocumentHandle frame commit의 증거가 아니다. 기존 integration30은 helper 추출에 대한 원 조립·실제 SDK/Native DOM 영향 회귀다.

Native libtest SHA `98B3526655FBFA1F8E9A2E138A297996862ABC448FF3B7D94D8B061026885AB4` 및 integration2의 원본/사본을 대조했다. 최종 receipt14는 모두 terminal/exit0, 원 실패 포함 receipt25를 보존했다. 최초 strict101과 r2의 실패9가 포함된 receipt3을 최종 통과로 소급하지 않는다.

## 보호와 범위

최종 root1491/snapshot1496의 SHA·UTC mtime 차이0, root/index 및 BIN3 사본 차이0, 기록 PID50 전부 부재를 확인했다. conservative2/CPU2/mask3/BelowNormal/descendant Job·기존 cache/offline/locked/-j1/thread1을 유지했다. 원 handle 회수 및 보호 대조 후 heavy/shared를 문서보다 먼저 반환했다. 전체868 시험·OS·GUI·브라우저·미디어 실행은 하지 않았다.

제품 route가 사용자에게 표시되는 연결, 원 frame의 keyed slot·실제 checked props/export commit, 정상 caller 및 시각 검토는 미완료다. [제품 진입 계획](../../../plans/player/presentation/proposal-product-entry.md)의 별도 frame 후보5 CC334620은 이 gate에 포함하지 않았고, 그 회귀2는 아직 작성 상태다.
